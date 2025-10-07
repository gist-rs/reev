//! Health monitoring system for dependency management
//!
//! This module provides health checking capabilities for external services
//! like reev-agent and surfpool, with configurable intervals and thresholds.

pub mod health_checker;
pub mod health_monitor;

pub use health_checker::HealthChecker;
pub use health_monitor::HealthMonitor;

use std::time::Duration;

/// Health status of a service
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceHealth {
    /// Service is healthy and responding
    Healthy,
    /// Service is unhealthy with specific reason
    Unhealthy(String),
    /// Service health is unknown (not checked yet)
    Unknown,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub service_name: String,
    pub status: ServiceHealth,
    pub response_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: Option<String>,
}

impl HealthCheckResult {
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            status: ServiceHealth::Unknown,
            response_time: Duration::from_millis(0),
            timestamp: chrono::Utc::now(),
            details: None,
        }
    }

    pub fn with_status(mut self, status: ServiceHealth) -> Self {
        self.status = status;
        self
    }

    pub fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ServiceHealth::Healthy)
    }

    pub fn is_unhealthy(&self) -> bool {
        matches!(self.status, ServiceHealth::Unhealthy(_))
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub check_interval: Duration,

    /// Timeout for individual health checks
    pub timeout: Duration,

    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: usize,

    /// Number of consecutive successes before marking as healthy
    pub success_threshold: usize,

    /// Whether to enable detailed logging
    pub verbose_logging: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            verbose_logging: false,
        }
    }
}

/// Health check statistics
#[derive(Debug, Clone, Default)]
pub struct HealthStats {
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub average_response_time: Duration,
    pub last_check_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl HealthStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.successful_checks as f64 / self.total_checks as f64
        }
    }

    pub fn update_success(&mut self, response_time: Duration) {
        self.total_checks += 1;
        self.successful_checks += 1;
        self.last_check_time = Some(chrono::Utc::now());
        self.last_success_time = Some(chrono::Utc::now());
        self.update_average_response_time(response_time);
    }

    pub fn update_failure(&mut self) {
        self.total_checks += 1;
        self.failed_checks += 1;
        self.last_check_time = Some(chrono::Utc::now());
        self.last_failure_time = Some(chrono::Utc::now());
    }

    fn update_average_response_time(&mut self, response_time: Duration) {
        let total_time =
            self.average_response_time * (self.successful_checks - 1) as u32 + response_time;
        self.average_response_time = total_time / self.successful_checks as u32;
    }
}

/// Error types for health monitoring
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("Health check failed for {service}: {reason}")]
    CheckFailed { service: String, reason: String },

    #[error("Health check timeout for {service} after {timeout_ms}ms")]
    Timeout { service: String, timeout_ms: u64 },

    #[error("Invalid health check configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Health monitoring not started")]
    NotStarted,

    #[error("Health monitoring already running")]
    AlreadyRunning,
}
