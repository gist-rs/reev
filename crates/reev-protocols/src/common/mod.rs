//! Common protocol abstractions and utilities
//!
//! This module provides traits and utilities for implementing
//! consistent protocol interfaces across all blockchain protocols.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Common error types for all protocols
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Rate limited: retry after {retry_after:?}")]
    RateLimited { retry_after: Duration },
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    #[error("Protocol-specific error: {protocol} - {message}")]
    ProtocolSpecific { protocol: String, message: String },
    #[error("Health check failed: {status}")]
    HealthCheckFailed { status: String },
    #[error("Timeout after {timeout:?}")]
    Timeout { timeout: Duration },
}

/// Protocol health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { message: String },
    Unhealthy { message: String },
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Protocol metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last successful request timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Last failed request timestamp
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    /// Current health status
    pub health_status: HealthStatus,
}

impl ProtocolMetrics {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }

    pub fn record_success(&mut self, response_time: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.last_success = Some(chrono::Utc::now());

        // Update average response time
        let total_time = self.avg_response_time_ms * (self.successful_requests - 1) as f64;
        self.avg_response_time_ms =
            (total_time + response_time.as_millis() as f64) / self.successful_requests as f64;
    }

    pub fn record_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_failure = Some(chrono::Utc::now());
    }

    pub fn update_health(&mut self, status: HealthStatus) {
        self.health_status = status;
    }
}

/// Common protocol trait for all blockchain operations
#[async_trait]
pub trait Protocol: Send + Sync {
    /// Protocol name for identification
    fn name(&self) -> &'static str;

    /// Check if the protocol is healthy and operational
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get current protocol metrics
    fn metrics(&self) -> &ProtocolMetrics;

    /// Reset protocol metrics
    fn reset_metrics(&mut self);

    /// Validate configuration
    fn validate_config(&self) -> Result<()> {
        // Default implementation - override if needed
        Ok(())
    }
}

// Re-export traits from the traits module
pub use traits::*;

/// Macro to implement common protocol functionality
/// Macro to implement common protocol functionality
#[macro_export]
macro_rules! impl_protocol_common {
    ($struct_name:ty, $protocol_name:expr) => {
        impl $crate::common::Protocol for $struct_name {
            fn name(&self) -> &'static str {
                $protocol_name
            }

            fn metrics(&self) -> &$crate::common::ProtocolMetrics {
                &self.metrics
            }

            fn reset_metrics(&mut self) {
                self.metrics = Default::default();
            }
        }
    };
}

/// Macro to measure protocol operation metrics
#[macro_export]
macro_rules! measure_protocol_operation {
    ($metrics:expr, $operation:expr, $async_block:block) => {{
        let start = std::time::Instant::now();
        let result = $async_block;
        let duration = start.elapsed();

        match result {
            Ok(output) => {
                $metrics.record_success($operation, duration, 0, 0);
                tracing::debug!(
                    protocol = $operation,
                    duration_ms = duration.as_millis(),
                    "Operation completed successfully"
                );
                Ok(output)
            }
            Err(e) => {
                $metrics.record_failure($operation, &e.to_string());
                tracing::error!(
                    protocol = $operation,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "Operation failed"
                );
                Err(e)
            }
        }
    }};
}

pub mod health;
pub mod metrics;
pub mod traits;

// Re-export commonly used items
pub use health::*;
pub use metrics::*;
