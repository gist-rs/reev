//! Health monitoring utilities for protocols
//!
//! This module provides health check functionality for monitoring
//! the operational status of blockchain protocols.

use crate::common::{HealthStatus, ProtocolError};
use std::time::{Duration, Instant};

/// Health checker configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for health check requests
    pub timeout: Duration,
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking as healthy
    pub success_threshold: u32,
    /// Whether to enable automatic recovery
    pub auto_recovery: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            success_threshold: 2,
            auto_recovery: true,
        }
    }
}

/// Health checker for protocol monitoring
pub struct HealthChecker {
    config: HealthCheckConfig,
    status: HealthStatus,
    last_check: Option<Instant>,
    consecutive_failures: u32,
    consecutive_successes: u32,
}

impl HealthChecker {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            status: HealthStatus::Unknown,
            last_check: None,
            consecutive_failures: 0,
            consecutive_successes: 0,
        }
    }

    pub fn status(&self) -> &HealthStatus {
        &self.status
    }

    pub fn last_check(&self) -> Option<Instant> {
        self.last_check
    }

    pub fn should_check(&self) -> bool {
        match self.last_check {
            Some(last) => last.elapsed() >= self.config.check_interval,
            None => true,
        }
    }

    pub async fn check_health<F, Fut>(&mut self, health_check_fn: F) -> Result<(), ProtocolError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<HealthStatus, ProtocolError>>,
    {
        let now = Instant::now();
        self.last_check = Some(now);

        // Execute health check with timeout
        let result = tokio::time::timeout(self.config.timeout, health_check_fn()).await;

        let status = match result {
            Ok(Ok(status)) => {
                self.consecutive_successes += 1;
                self.consecutive_failures = 0;

                if self.consecutive_successes >= self.config.success_threshold {
                    // Recover from degraded/unhealthy state
                    tracing::info!(
                        protocol = "health_check",
                        consecutive_successes = self.consecutive_successes,
                        "Protocol recovered to healthy state"
                    );
                }

                status
            }
            Ok(Err(e)) => {
                self.consecutive_failures += 1;
                self.consecutive_successes = 0;

                tracing::warn!(
                    protocol = "health_check",
                    error = %e,
                    consecutive_failures = self.consecutive_failures,
                    "Health check failed"
                );

                if self.consecutive_failures >= self.config.failure_threshold {
                    HealthStatus::Unhealthy {
                        message: format!(
                            "Health check failed {} times: {}",
                            self.consecutive_failures, e
                        ),
                    }
                } else {
                    HealthStatus::Degraded {
                        message: format!("Health check failed: {e}"),
                    }
                }
            }
            Err(_) => {
                self.consecutive_failures += 1;
                self.consecutive_successes = 0;

                tracing::error!(
                    protocol = "health_check",
                    timeout_ms = self.config.timeout.as_millis(),
                    "Health check timed out"
                );

                HealthStatus::Unhealthy {
                    message: format!("Health check timed out after {:?}", self.config.timeout),
                }
            }
        };

        self.status = status;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.status = HealthStatus::Unknown;
        self.last_check = None;
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
    }
}

/// Multi-protocol health monitor
pub struct HealthMonitor {
    checkers: std::collections::HashMap<String, HealthChecker>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checkers: std::collections::HashMap::new(),
        }
    }

    pub fn register_protocol(&mut self, name: String, config: HealthCheckConfig) {
        self.checkers.insert(name, HealthChecker::new(config));
    }

    pub fn get_status(&self, protocol_name: &str) -> Option<&HealthStatus> {
        self.checkers
            .get(protocol_name)
            .map(|checker| checker.status())
    }

    pub fn get_all_status(&self) -> std::collections::HashMap<String, &HealthStatus> {
        self.checkers
            .iter()
            .map(|(name, checker)| (name.clone(), checker.status()))
            .collect()
    }

    pub async fn check_all<F, Fut>(
        &mut self,
        health_check_fn: F,
    ) -> Vec<(String, Result<(), ProtocolError>)>
    where
        F: Fn(&str) -> Fut,
        Fut: std::future::Future<Output = Result<HealthStatus, ProtocolError>>,
    {
        let mut results = Vec::new();

        for (name, checker) in self.checkers.iter_mut() {
            if checker.should_check() {
                let result = checker.check_health(|| health_check_fn(name)).await;
                results.push((name.clone(), result));
            }
        }

        results
    }

    pub fn overall_health(&self) -> HealthStatus {
        if self.checkers.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut unhealthy_count = 0;
        let mut degraded_count = 0;

        for checker in self.checkers.values() {
            match checker.status() {
                HealthStatus::Unhealthy { .. } => unhealthy_count += 1,
                HealthStatus::Degraded { .. } => degraded_count += 1,
                _ => {}
            }
        }

        let total = self.checkers.len();

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy {
                message: format!("{unhealthy_count} of {total} protocols are unhealthy"),
            }
        } else if degraded_count > 0 {
            HealthStatus::Degraded {
                message: format!("{degraded_count} of {total} protocols are degraded"),
            }
        } else {
            HealthStatus::Healthy
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to create a basic health check
pub async fn basic_health_check(
    protocol_name: &str,
    endpoint_url: &str,
) -> Result<HealthStatus, ProtocolError> {
    let client = reqwest::Client::new();
    let response = client
        .get(endpoint_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!(
                protocol = protocol_name,
                status = resp.status().as_u16(),
                "Health check successful"
            );
            Ok(HealthStatus::Healthy)
        }
        Ok(resp) => {
            tracing::warn!(
                protocol = protocol_name,
                status = resp.status().as_u16(),
                "Health check returned non-success status"
            );
            Ok(HealthStatus::Degraded {
                message: format!("HTTP {}", resp.status()),
            })
        }
        Err(e) => {
            tracing::error!(
                protocol = protocol_name,
                error = %e,
                "Health check request failed"
            );
            Err(ProtocolError::Network(e.to_string()))
        }
    }
}
