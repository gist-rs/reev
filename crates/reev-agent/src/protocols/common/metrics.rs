//! Metrics collection utilities for protocols
//!
//! This module provides metrics collection and aggregation functionality
//! for monitoring protocol performance and usage patterns.

use crate::protocols::common::HealthStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Protocol metrics for monitoring and analytics
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
    /// Minimum response time in milliseconds
    pub min_response_time_ms: Option<u64>,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: Option<u64>,
    /// Last successful request timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Last failed request timestamp
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    /// Current health status
    pub health_status: HealthStatus,
    /// Request counts by operation type
    pub operation_counts: HashMap<String, u64>,
    /// Error counts by error type
    pub error_counts: HashMap<String, u64>,
    /// Total volume processed (in smallest token units)
    pub total_volume: u64,
    /// Total fees paid (in smallest token units)
    pub total_fees: u64,
    /// Unique users count
    pub unique_users: u64,
    /// Metrics collection start time
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl ProtocolMetrics {
    pub fn new() -> Self {
        Self {
            start_time: chrono::Utc::now(),
            ..Default::default()
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.failed_requests as f64 / self.total_requests as f64
    }

    pub fn uptime_percentage(&self) -> f64 {
        let total_time = chrono::Utc::now() - self.start_time;
        let total_seconds = total_time.num_seconds() as f64;

        if total_seconds == 0.0 {
            return 100.0;
        }

        // Calculate uptime based on health status changes
        // This is simplified - in practice you'd track status changes over time
        match self.health_status {
            HealthStatus::Healthy => 100.0,
            HealthStatus::Degraded { .. } => 75.0,
            HealthStatus::Unhealthy { .. } => 0.0,
            HealthStatus::Unknown => 50.0,
        }
    }

    pub fn record_success(
        &mut self,
        operation: &str,
        response_time: Duration,
        volume: u64,
        fees: u64,
    ) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.last_success = Some(chrono::Utc::now());

        // Update response time statistics
        let response_time_ms = response_time.as_millis() as u64;
        self.min_response_time_ms = Some(
            self.min_response_time_ms
                .map(|min| min.min(response_time_ms))
                .unwrap_or(response_time_ms),
        );
        self.max_response_time_ms = Some(
            self.max_response_time_ms
                .map(|max| max.max(response_time_ms))
                .unwrap_or(response_time_ms),
        );

        // Update average response time
        let total_time = self.avg_response_time_ms * (self.successful_requests - 1) as f64;
        self.avg_response_time_ms =
            (total_time + response_time_ms as f64) / self.successful_requests as f64;

        // Update operation counts
        *self
            .operation_counts
            .entry(operation.to_string())
            .or_insert(0) += 1;

        // Update volume and fees
        self.total_volume += volume;
        self.total_fees += fees;
    }

    pub fn record_failure(&mut self, operation: &str, error: &str) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.last_failure = Some(chrono::Utc::now());

        // Update operation counts
        *self
            .operation_counts
            .entry(operation.to_string())
            .or_insert(0) += 1;

        // Update error counts
        *self.error_counts.entry(error.to_string()).or_insert(0) += 1;
    }

    pub fn update_health(&mut self, status: HealthStatus) {
        self.health_status = status;
    }

    pub fn record_user(&mut self) {
        self.unique_users += 1;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get metrics summary for reporting
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_requests: self.total_requests,
            success_rate: self.success_rate(),
            error_rate: self.error_rate(),
            avg_response_time_ms: self.avg_response_time_ms,
            uptime_percentage: self.uptime_percentage(),
            total_volume: self.total_volume,
            total_fees: self.total_fees,
            unique_users: self.unique_users,
            health_status: self.health_status.clone(),
            top_operations: self.get_top_operations(5),
            top_errors: self.get_top_errors(5),
        }
    }

    fn get_top_operations(&self, limit: usize) -> Vec<(String, u64)> {
        let mut operations: Vec<_> = self
            .operation_counts
            .iter()
            .map(|(op, count)| (op.clone(), *count))
            .collect();
        operations.sort_by(|a, b| b.1.cmp(&a.1));
        operations.truncate(limit);
        operations
    }

    fn get_top_errors(&self, limit: usize) -> Vec<(String, u64)> {
        let mut errors: Vec<_> = self
            .error_counts
            .iter()
            .map(|(error, count)| (error.clone(), *count))
            .collect();
        errors.sort_by(|a, b| b.1.cmp(&a.1));
        errors.truncate(limit);
        errors
    }
}

/// Metrics summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub uptime_percentage: f64,
    pub total_volume: u64,
    pub total_fees: u64,
    pub unique_users: u64,
    pub health_status: HealthStatus,
    pub top_operations: Vec<(String, u64)>,
    pub top_errors: Vec<(String, u64)>,
}

/// Metrics collector for multiple protocols
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, ProtocolMetrics>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_metrics(&self, protocol_name: &str) -> Option<ProtocolMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(protocol_name).cloned()
    }

    pub async fn set_metrics(&self, protocol_name: &str, metrics: ProtocolMetrics) {
        let mut metrics_map = self.metrics.write().await;
        metrics_map.insert(protocol_name.to_string(), metrics);
    }

    pub async fn update_metrics<F>(&self, protocol_name: &str, updater: F)
    where
        F: FnOnce(&mut ProtocolMetrics),
    {
        let mut metrics_map = self.metrics.write().await;
        let metrics = metrics_map.entry(protocol_name.to_string()).or_default();
        updater(metrics);
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, ProtocolMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn get_all_summaries(&self) -> HashMap<String, MetricsSummary> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .map(|(name, metrics)| (name.clone(), metrics.summary()))
            .collect()
    }

    pub async fn reset_protocol(&self, protocol_name: &str) {
        let mut metrics_map = self.metrics.write().await;
        if let Some(metrics) = metrics_map.get_mut(protocol_name) {
            metrics.reset();
        }
    }

    pub async fn reset_all(&self) {
        let mut metrics_map = self.metrics.write().await;
        for (_, metrics) in metrics_map.iter_mut() {
            metrics.reset();
        }
    }

    pub async fn aggregate_summary(&self) -> AggregatedMetrics {
        let metrics_map = self.metrics.read().await;
        let mut total_requests = 0;
        let mut total_successful = 0;
        let mut total_failed = 0;
        let mut total_response_time = 0.0;
        let mut total_response_time_count = 0;
        let mut total_volume = 0;
        let mut total_fees = 0;
        let mut total_unique_users = 0;
        let mut healthy_protocols = 0;
        let total_protocols = metrics_map.len();

        for (_, metrics) in metrics_map.iter() {
            total_requests += metrics.total_requests;
            total_successful += metrics.successful_requests;
            total_failed += metrics.failed_requests;

            if metrics.successful_requests > 0 {
                total_response_time +=
                    metrics.avg_response_time_ms * metrics.successful_requests as f64;
                total_response_time_count += metrics.successful_requests;
            }

            total_volume += metrics.total_volume;
            total_fees += metrics.total_fees;
            total_unique_users += metrics.unique_users;

            if matches!(metrics.health_status, HealthStatus::Healthy) {
                healthy_protocols += 1;
            }
        }

        let overall_success_rate = if total_requests > 0 {
            total_successful as f64 / total_requests as f64
        } else {
            0.0
        };

        let overall_avg_response_time = if total_response_time_count > 0 {
            total_response_time / total_response_time_count as f64
        } else {
            0.0
        };

        AggregatedMetrics {
            total_protocols: total_protocols as u64,
            healthy_protocols,
            total_requests,
            success_rate: overall_success_rate,
            avg_response_time_ms: overall_avg_response_time,
            total_volume,
            total_fees,
            total_unique_users,
            total_failed,
            collection_time: chrono::Utc::now(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated metrics across all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_protocols: u64,
    pub healthy_protocols: u64,
    pub total_requests: u64,
    pub total_failed: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub total_volume: u64,
    pub total_fees: u64,
    pub total_unique_users: u64,
    pub collection_time: chrono::DateTime<chrono::Utc>,
}

impl AggregatedMetrics {
    pub fn health_percentage(&self) -> f64 {
        if self.total_protocols == 0 {
            return 100.0;
        }
        (self.healthy_protocols as f64 / self.total_protocols as f64) * 100.0
    }
}

/// Utility trait for measuring operation metrics
pub trait Measurable {
    fn measure_operation<F, T, E>(&self, operation: &str, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display;
}

#[macro_export]
macro_rules! measure_operation {
    ($metrics:expr, $operation:expr, $async_block:block) => {{
        let start = std::time::Instant::now();
        let result = $async_block;
        let duration = start.elapsed();

        match result {
            Ok((output, volume, fees)) => {
                $metrics.record_success($operation, duration, volume, fees);
                tracing::debug!(
                    protocol = $operation,
                    duration_ms = duration.as_millis(),
                    volume = volume,
                    fees = fees,
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
