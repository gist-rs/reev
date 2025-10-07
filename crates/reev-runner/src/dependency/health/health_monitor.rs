//! Health monitor for continuous monitoring of dependency services
//!
//! This module provides continuous health monitoring capabilities for external services
//! with background tasks, statistics tracking, and event notifications.

use super::{
    HealthCheckConfig, HealthCheckResult, HealthChecker, HealthError, HealthStats, ServiceHealth,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::interval;
use tracing::{info, warn};

/// Health monitor for continuous monitoring of multiple services
pub struct HealthMonitor {
    /// Health checker instance
    checker: HealthChecker,

    /// Services being monitored
    services: Arc<RwLock<HashMap<String, MonitoredService>>>,

    /// Background task handle
    monitor_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,

    /// Event sender for health status changes
    event_sender: mpsc::UnboundedSender<HealthEvent>,

    /// Event receiver for health status changes
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<HealthEvent>>>,

    /// Whether monitoring is currently active
    is_monitoring: Arc<RwLock<bool>>,
}

/// A service being monitored
#[derive(Debug, Clone)]
pub struct MonitoredService {
    pub name: String,
    pub url: String,
    pub service_type: ServiceType,
    pub current_status: ServiceHealth,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub stats: HealthStats,
    pub config: HealthCheckConfig,
}

/// Types of services that can be monitored
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    ReevAgent,
    Surfpool,
    Custom,
}

/// Health monitoring events
#[derive(Debug, Clone)]
pub enum HealthEvent {
    ServiceStatusChanged {
        service_name: String,
        old_status: ServiceHealth,
        new_status: ServiceHealth,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ServiceCheckCompleted {
        service_name: String,
        result: HealthCheckResult,
    },
    MonitoringStarted {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    MonitoringStopped {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            checker: HealthChecker::new(config),
            services: Arc::new(RwLock::new(HashMap::new())),
            monitor_handle: Arc::new(Mutex::new(None)),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a service to monitor
    pub async fn add_service(
        &self,
        name: String,
        url: String,
        service_type: ServiceType,
        config: Option<HealthCheckConfig>,
    ) -> Result<()> {
        let service = MonitoredService {
            name: name.clone(),
            url,
            service_type,
            current_status: ServiceHealth::Unknown,
            last_check: None,
            stats: HealthStats::default(),
            config: config.unwrap_or_else(|| self.checker.config().clone()),
        };

        let mut services = self.services.write().await;
        services.insert(name.clone(), service);

        info!(service_name = %name, "Added service to health monitor");
        Ok(())
    }

    /// Remove a service from monitoring
    pub async fn remove_service(&self, name: &str) -> Result<bool> {
        let mut services = self.services.write().await;
        let removed = services.remove(name).is_some();

        if removed {
            info!(service_name = %name, "Removed service from health monitor");
        } else {
            warn!(service_name = %name, "Service not found in health monitor");
        }

        Ok(removed)
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            return Err(HealthError::AlreadyRunning.into());
        }

        *is_monitoring = true;
        drop(is_monitoring);

        let services = self.services.clone();
        let checker = self.checker.clone();
        let event_sender = self.event_sender.clone();
        let is_monitoring_flag = self.is_monitoring.clone();

        let handle = tokio::spawn(async move {
            info!("Started continuous health monitoring");

            // Send monitoring started event
            let _ = event_sender.send(HealthEvent::MonitoringStarted {
                timestamp: chrono::Utc::now(),
            });

            let mut interval = interval(checker.config().check_interval);

            loop {
                // Check if monitoring should continue
                {
                    let is_monitoring = is_monitoring_flag.read().await;
                    if !*is_monitoring {
                        break;
                    }
                }

                interval.tick().await;

                // Check all services
                let services_to_check = {
                    let services = services.read().await;
                    services.clone()
                };

                for (name, service) in services_to_check {
                    let result = match service.service_type {
                        ServiceType::ReevAgent => checker.check_reev_agent(&service.url).await,
                        ServiceType::Surfpool => checker.check_surfpool(&service.url).await,
                        ServiceType::Custom => {
                            // For custom services, use generic HTTP check
                            checker.check_http_endpoint(&name, &service.url).await
                        }
                    };

                    // Update service status
                    {
                        let mut services = services.write().await;
                        if let Some(monitored_service) = services.get_mut(&name) {
                            let old_status = monitored_service.current_status.clone();
                            monitored_service.current_status = result.status.clone();
                            monitored_service.last_check = Some(result.timestamp);

                            // Update statistics
                            if result.is_healthy() {
                                monitored_service.stats.update_success(result.response_time);
                            } else {
                                monitored_service.stats.update_failure();
                            }

                            // Send status change event if needed
                            if old_status != result.status {
                                let _ = event_sender.send(HealthEvent::ServiceStatusChanged {
                                    service_name: name.clone(),
                                    old_status,
                                    new_status: result.status.clone(),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                    }

                    // Send check completed event
                    let _ = event_sender.send(HealthEvent::ServiceCheckCompleted {
                        service_name: name,
                        result,
                    });
                }
            }

            info!("Stopped continuous health monitoring");

            // Send monitoring stopped event
            let _ = event_sender.send(HealthEvent::MonitoringStopped {
                timestamp: chrono::Utc::now(),
            });
        });

        let mut monitor_handle = self.monitor_handle.lock().await;
        *monitor_handle = Some(handle);

        Ok(())
    }

    /// Stop continuous health monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if !*is_monitoring {
            return Err(HealthError::NotStarted.into());
        }

        *is_monitoring = false;
        drop(is_monitoring);

        // Wait for the monitoring task to finish
        let mut monitor_handle = self.monitor_handle.lock().await;
        if let Some(handle) = monitor_handle.take() {
            handle.abort();
        }

        info!("Stopped health monitoring");
        Ok(())
    }

    /// Get current status of all monitored services
    pub async fn get_all_statuses(&self) -> HashMap<String, ServiceHealth> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, service)| (name.clone(), service.current_status.clone()))
            .collect()
    }

    /// Get detailed information about a specific service
    pub async fn get_service_info(&self, name: &str) -> Option<MonitoredService> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    /// Get health statistics for all services
    pub async fn get_all_stats(&self) -> HashMap<String, HealthStats> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, service)| (name.clone(), service.stats.clone()))
            .collect()
    }

    /// Perform a one-time health check of all services
    pub async fn check_all_services(&self) -> HashMap<String, HealthCheckResult> {
        let services = self.services.read().await;
        let mut results = HashMap::new();

        for (name, service) in services.iter() {
            let result = match service.service_type {
                ServiceType::ReevAgent => self.checker.check_reev_agent(&service.url).await,
                ServiceType::Surfpool => self.checker.check_surfpool(&service.url).await,
                ServiceType::Custom => self.checker.check_http_endpoint(name, &service.url).await,
            };

            results.insert(name.clone(), result);
        }

        results
    }

    /// Wait for a specific service to become healthy
    pub async fn wait_for_service_health(
        &self,
        service_name: &str,
        timeout: Duration,
    ) -> Result<()> {
        let services = self.services.read().await;
        let service = services
            .get(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service '{service_name}' not found"))?;

        let service_type = service.service_type;
        let url = service.url.clone();
        let checker = self.checker.clone();

        self.checker
            .wait_for_health(
                service_name,
                move || {
                    let service_type = service_type;
                    let url = url.clone();
                    let checker = checker.clone();

                    async move {
                        match service_type {
                            ServiceType::ReevAgent => checker.check_reev_agent(&url).await,
                            ServiceType::Surfpool => checker.check_surfpool(&url).await,
                            ServiceType::Custom => {
                                checker.check_http_endpoint("custom", &url).await
                            }
                        }
                    }
                },
                timeout,
            )
            .await
    }

    /// Get the next health event (non-blocking)
    pub async fn try_recv_event(&self) -> Option<HealthEvent> {
        let mut receiver = self.event_receiver.lock().await;
        receiver.try_recv().ok()
    }

    /// Get the next health event (blocking with timeout)
    pub async fn recv_event_timeout(&self, timeout: Duration) -> Option<HealthEvent> {
        let mut receiver = self.event_receiver.lock().await;
        (tokio::time::timeout(timeout, receiver.recv()).await).unwrap_or_default()
    }

    /// Check if monitoring is currently active
    pub async fn is_monitoring(&self) -> bool {
        *self.is_monitoring.read().await
    }

    /// Get the number of services being monitored
    pub async fn service_count(&self) -> usize {
        self.services.read().await.len()
    }

    /// Get the number of healthy services
    pub async fn healthy_service_count(&self) -> usize {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.current_status == ServiceHealth::Healthy)
            .count()
    }

    /// Get the number of unhealthy services
    pub async fn unhealthy_service_count(&self) -> usize {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| matches!(s.current_status, ServiceHealth::Unhealthy(_)))
            .count()
    }
}

// Clone derived automatically on HealthChecker struct

impl Drop for HealthMonitor {
    fn drop(&mut self) {
        // Note: This is a synchronous drop, but we need to handle async cleanup
        // In a real implementation, you might want to use a different approach
        // or ensure proper cleanup is done before the monitor goes out of scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);

        assert!(!monitor.is_monitoring().await);
        assert_eq!(monitor.service_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_remove_service() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);

        monitor
            .add_service(
                "test-service".to_string(),
                "http://localhost:8080".to_string(),
                ServiceType::Custom,
                None,
            )
            .await
            .unwrap();

        assert_eq!(monitor.service_count().await, 1);

        let removed = monitor.remove_service("test-service").await.unwrap();
        assert!(removed);
        assert_eq!(monitor.service_count().await, 0);
    }

    #[tokio::test]
    async fn test_double_start_monitoring() {
        let config = HealthCheckConfig {
            check_interval: Duration::from_millis(100),
            ..Default::default()
        };
        let monitor = HealthMonitor::new(config);

        monitor.start_monitoring().await.unwrap();

        // Second start should fail
        let result = monitor.start_monitoring().await;
        assert!(result.is_err());

        monitor.stop_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_stop_without_start() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);

        let result = monitor.stop_monitoring().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_counts() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);

        assert_eq!(monitor.service_count().await, 0);
        assert_eq!(monitor.healthy_service_count().await, 0);
        assert_eq!(monitor.unhealthy_service_count().await, 0);

        // Add services
        monitor
            .add_service(
                "service1".to_string(),
                "http://localhost:8080".to_string(),
                ServiceType::Custom,
                None,
            )
            .await
            .unwrap();

        monitor
            .add_service(
                "service2".to_string(),
                "http://localhost:8081".to_string(),
                ServiceType::Custom,
                None,
            )
            .await
            .unwrap();

        assert_eq!(monitor.service_count().await, 2);
        // Both services start as Unknown, not counted as healthy or unhealthy
        assert_eq!(monitor.healthy_service_count().await, 0);
        assert_eq!(monitor.unhealthy_service_count().await, 0);
    }

    #[tokio::test]
    async fn test_events() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);

        // Should not have any events initially
        assert!(monitor.try_recv_event().await.is_none());

        // Add a service (should not generate events)
        monitor
            .add_service(
                "test-service".to_string(),
                "http://localhost:8080".to_string(),
                ServiceType::Custom,
                None,
            )
            .await
            .unwrap();

        // Still no events
        assert!(monitor.try_recv_event().await.is_none());
    }
}
