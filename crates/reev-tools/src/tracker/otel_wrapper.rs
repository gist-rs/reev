//! Simple Tool Metrics Collection for OpenTelemetry Integration
//!
//! This module provides basic tool identification and metrics collection
//! that works with rig's built-in OpenTelemetry integration without
//! interfering with the tool execution flow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple tool wrapper for identification purposes
///
/// This wrapper doesn't interfere with rig's OpenTelemetry integration
/// but provides a way to identify tools for metrics collection.
pub struct SimpleToolWrapper<T> {
    /// The underlying rig tool
    inner: T,
    /// Tool name for metrics and identification
    tool_name: String,
}

impl<T> SimpleToolWrapper<T> {
    /// Create a new simple wrapper for a tool
    pub fn new(tool: T, tool_name: &str) -> Self {
        Self {
            inner: tool,
            tool_name: tool_name.to_string(),
        }
    }

    /// Get the underlying tool
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get the tool name
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }
}

/// Macro to wrap a tool with simple identification
#[macro_export]
macro_rules! simple_tool {
    ($tool:expr, $name:expr) => {
        $crate::tracker::otel_wrapper::SimpleToolWrapper::new($tool, $name)
    };
}

/// Tool execution metrics collected from OpenTelemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionMetrics {
    /// Tool name
    pub tool_name: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the execution succeeded
    pub success: bool,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Timestamp of execution
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ToolExecutionMetrics {
    /// Create new metrics for a successful execution
    pub fn success(tool_name: String, execution_time_ms: u64) -> Self {
        Self {
            tool_name,
            execution_time_ms,
            success: true,
            error_message: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create new metrics for a failed execution
    pub fn failure(tool_name: String, execution_time_ms: u64, error: &str) -> Self {
        Self {
            tool_name,
            execution_time_ms,
            success: false,
            error_message: Some(error.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Collector for tool execution metrics from OpenTelemetry traces
///
/// This collector extracts metrics from rig's OpenTelemetry integration
/// without interfering with tool execution.
pub struct OtelMetricsCollector {
    /// Cached metrics collected from OpenTelemetry
    metrics: HashMap<String, Vec<ToolExecutionMetrics>>,
}

impl OtelMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Collect metrics for a specific tool from OpenTelemetry traces
    pub fn collect_tool_metrics(&mut self, tool_name: &str) -> Vec<ToolExecutionMetrics> {
        // In a real implementation, this would query the OpenTelemetry backend
        // for spans related to this tool. For now, return cached metrics.
        self.metrics.get(tool_name).cloned().unwrap_or_default()
    }

    /// Get all collected metrics
    pub fn get_all_metrics(&self) -> HashMap<String, Vec<ToolExecutionMetrics>> {
        self.metrics.clone()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    /// Add metrics manually (for testing or manual tracking)
    pub fn add_metrics(&mut self, metrics: ToolExecutionMetrics) {
        self.metrics
            .entry(metrics.tool_name.clone())
            .or_default()
            .push(metrics);
    }
}

impl Default for OtelMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize simple tool tracing
///
/// This function just logs that tool tracing relies on rig's built-in
/// OpenTelemetry integration. The actual tracing is handled automatically
/// by the rig framework when tools are executed.
pub fn init_simple_tool_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Tool tracing relies on rig's built-in OpenTelemetry integration");
    tracing::info!("Tool calls are automatically traced to REEV_TRACE_FILE=traces.log");
    Ok(())
}
