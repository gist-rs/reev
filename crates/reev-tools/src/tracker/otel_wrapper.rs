//! OpenTelemetry Tool Wrapper for Rig Integration
//!
//! This module provides proper OpenTelemetry integration for rig tools
//! without breaking the tool system. It follows the pattern from
//! rig's agent_with_tools_otel example to properly capture tool execution
//! through OpenTelemetry spans instead of manual interception.
//!
//! Unlike the broken manual tool call tracking, this approach:
//! - Uses proper OpenTelemetry instrumentation
//! - Doesn't interfere with rig's tool execution
//! - Provides automatic span creation for tool calls
//! - Follows OpenTelemetry best practices

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::{global, KeyValue};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, instrument};

/// OpenTelemetry-enabled tool wrapper
///
/// This wrapper adds proper OpenTelemetry tracing to any rig tool
/// without breaking the tool's execution flow.
pub struct OtelToolWrapper<T> {
    /// The underlying rig tool
    inner: T,
    /// Tool name for tracing
    tool_name: String,
}

impl<T> OtelToolWrapper<T> {
    /// Create a new OpenTelemetry wrapper for a tool
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

impl<T> OtelToolWrapper<T>
where
    T: Tool,
{
    /// Execute the tool with OpenTelemetry tracing
    #[instrument(skip(self, args), fields(tool_name = %self.tool_name))]
    pub async fn execute_with_tracing(&self, args: T::Args) -> Result<T::Output, T::Error> {
        let start_time = Instant::now();

        // Get the global tracer
        let tracer = global::tracer("reev-tools");

        // Create a span for the tool execution
        let mut span = tracer
            .span_builder(&format!("tool_{}", self.tool_name))
            .with_attributes(vec![
                KeyValue::new("tool.name", self.tool_name.clone()),
                KeyValue::new("tool.args", format!("{:?}", args)),
            ])
            .start(&tracer);

        // Set the span as active
        let _guard = span.enter();

        info!("[OtelToolWrapper] Executing tool: {}", self.tool_name);
        debug!("[OtelToolWrapper] Tool args: {:?}", args);

        // Execute the actual tool
        let result = self.inner.call(args).await;

        // Record execution time
        let execution_time = start_time.elapsed();
        span.set_attribute(KeyValue::new(
            "tool.execution_time_ms",
            execution_time.as_millis() as u64,
        ));

        // Record result
        match &result {
            Ok(output) => {
                span.set_attribute(KeyValue::new("tool.success", true));
                span.set_attribute(KeyValue::new("tool.output_length", output.len() as u64));
                info!(
                    "[OtelToolWrapper] Tool {} completed successfully in {}ms",
                    self.tool_name,
                    execution_time.as_millis()
                );
            }
            Err(error) => {
                span.set_attribute(KeyValue::new("tool.success", false));
                span.set_attribute(KeyValue::new("tool.error", format!("{:?}", error)));
                span.set_status(opentelemetry::trace::Status::error(format!(
                    "Tool failed: {:?}",
                    error
                )));
                warn!(
                    "[OtelToolWrapper] Tool {} failed in {}ms: {:?}",
                    self.tool_name,
                    execution_time.as_millis(),
                    error
                );
            }
        }

        // End the span
        span.end();

        result
    }
}

/// Macro to wrap a tool with OpenTelemetry tracing
#[macro_export]
macro_rules! otel_tool {
    ($tool:expr, $name:expr) => {
        $crate::tracker::otel_wrapper::OtelToolWrapper::new($tool, $name)
    };
}

/// Tool execution metrics collected via OpenTelemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionMetrics {
    /// Tool name
    pub tool_name: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the execution succeeded
    pub success: bool,
    /// Output length (if successful)
    pub output_length: Option<usize>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Timestamp of execution
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ToolExecutionMetrics {
    /// Create new metrics for a successful execution
    pub fn success(tool_name: String, execution_time_ms: u64, output_length: usize) -> Self {
        Self {
            tool_name,
            execution_time_ms,
            success: true,
            output_length: Some(output_length),
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
            output_length: None,
            error_message: Some(error.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Collector for tool execution metrics from OpenTelemetry
pub struct OtelMetricsCollector {
    /// Cached metrics
    metrics: HashMap<String, Vec<ToolExecutionMetrics>>,
}

impl OtelMetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Collect metrics for a specific tool
    pub fn collect_tool_metrics(&mut self, tool_name: &str) -> Vec<ToolExecutionMetrics> {
        // In a real implementation, this would query the OpenTelemetry backend
        // For now, return cached metrics
        self.metrics.get(tool_name).cloned().unwrap_or_default()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, Vec<ToolExecutionMetrics>> {
        self.metrics.clone()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    /// Add metrics (for testing)
    #[cfg(test)]
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

/// Initialize OpenTelemetry for tool tracing
pub fn init_otel_tool_tracing() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use opentelemetry_sdk::Resource;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Check if OpenTelemetry is enabled
    let enabled = std::env::var("REEV_OTEL_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if !enabled {
        info!("OpenTelemetry tool tracing disabled");
        return Ok(());
    }

    info!("Initializing OpenTelemetry tool tracing...");

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .build()?;

    // Create tracer provider
    let service_name =
        std::env::var("REEV_OTEL_SERVICE_NAME").unwrap_or_else(|_| "reev-tools".to_string());

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(service_name).build())
        .build();

    let tracer = provider.tracer("reev-tools");

    // Initialize tracing subscriber
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    info!("OpenTelemetry tool tracing initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::native::SolTransferTool;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_otel_tool_wrapper() {
        // Create a mock tool
        let sol_tool = SolTransferTool {
            key_map: HashMap::new(),
        };

        // Wrap it with OpenTelemetry
        let wrapped_tool = OtelToolWrapper::new(sol_tool, "sol_transfer_test");

        // Verify the wrapper
        assert_eq!(wrapped_tool.tool_name(), "sol_transfer_test");
    }

    #[test]
    fn test_tool_execution_metrics() {
        let success_metrics = ToolExecutionMetrics::success("test_tool".to_string(), 100, 42);
        assert!(success_metrics.success);
        assert_eq!(success_metrics.execution_time_ms, 100);
        assert_eq!(success_metrics.output_length, Some(42));

        let failure_metrics =
            ToolExecutionMetrics::failure("test_tool".to_string(), 50, "test error");
        assert!(!failure_metrics.success);
        assert_eq!(failure_metrics.execution_time_ms, 50);
        assert_eq!(
            failure_metrics.error_message,
            Some("test error".to_string())
        );
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = OtelMetricsCollector::new();

        // Add some test metrics
        let metrics = ToolExecutionMetrics::success("test_tool".to_string(), 100, 42);
        collector.add_metrics(metrics.clone());

        // Collect metrics
        let collected = collector.collect_tool_metrics("test_tool");
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].tool_name, "test_tool");

        // Clear metrics
        collector.clear();
        let collected_after_clear = collector.collect_tool_metrics("test_tool");
        assert!(collected_after_clear.is_empty());
    }
}
