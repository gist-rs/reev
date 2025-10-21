//! OpenTelemetry integration for flow tracing
//!
//! This module provides proper OpenTelemetry integration with the rig framework
//! for tool call tracing and flow monitoring. It follows the pattern from
//! rig's agent_with_tools_otel example to properly capture tool execution
//! without breaking the tool system.

use super::types::*;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing::{debug, info, instrument, warn, Span};

/// Proper OpenTelemetry flow tracer for rig integration
pub struct FlowTracer {
    enabled: bool,
    tracer_provider: Option<SdkTracerProvider>,
}

impl FlowTracer {
    /// Create a new OpenTelemetry flow tracer with rig integration
    pub fn new() -> Self {
        let enabled = std::env::var("REEV_OTEL_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        if enabled {
            info!("Initializing OpenTelemetry flow tracer with rig integration");

            // Initialize OpenTelemetry if enabled
            if let Ok(provider) = Self::init_otel() {
                Self {
                    enabled: true,
                    tracer_provider: Some(provider),
                }
            } else {
                warn!("Failed to initialize OpenTelemetry, falling back to disabled mode");
                Self {
                    enabled: false,
                    tracer_provider: None,
                }
            }
        } else {
            info!("OpenTelemetry flow tracing disabled");
            Self {
                enabled: false,
                tracer_provider: None,
            }
        }
    }

    /// Initialize OpenTelemetry with OTLP exporter
    fn init_otel() -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
            .build()?;

        let service_name =
            std::env::var("REEV_OTEL_SERVICE_NAME").unwrap_or_else(|_| "reev".to_string());

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(Resource::builder().with_service_name(service_name).build())
            .build();

        info!("OpenTelemetry initialized with service: {}", service_name);
        Ok(provider)
    }
}

impl Drop for FlowTracer {
    fn drop(&mut self) {
        if let Some(provider) = self.tracer_provider.take() {
            info!("Shutting down OpenTelemetry tracer provider");
            let _ = provider.shutdown();
        }
    }
}

impl Default for FlowTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowTracer {
    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Trace a complete flow log with proper OpenTelemetry spans
    #[instrument(skip(self, flow), fields(session_id = %flow.session_id, benchmark_id = %flow.benchmark_id))]
    pub fn trace_flow(&self, flow: &FlowLog) {
        if !self.enabled {
            return;
        }

        let span = Span::current();
        span.record("agent", flow.agent_type.as_str());
        span.record("events_count", flow.events.len() as i64);

        info!("Tracing flow execution for session: {}", flow.session_id);
        info!("  - Benchmark: {}", flow.benchmark_id);
        info!("  - Agent: {}", flow.agent_type);
        info!("  - Events: {}", flow.events.len());

        // Trace each event in the flow with proper spans
        for event in &flow.events {
            self.trace_flow_event(event);
        }

        // Log final result
        if let Some(result) = &flow.final_result {
            span.record("success", result.success);
            span.record("score", result.score);
            span.record("total_time_ms", result.total_time_ms as i64);
            span.record("llm_calls", result.statistics.total_llm_calls as i64);
            span.record("tool_calls", result.statistics.total_tool_calls as i64);

            info!("Flow completed:");
            info!("  - Success: {}", result.success);
            info!("  - Score: {:.3}", result.score);
            info!("  - Time: {}ms", result.total_time_ms);
            info!("  - LLM calls: {}", result.statistics.total_llm_calls);
            info!("  - Tool calls: {}", result.statistics.total_tool_calls);
        }

        debug!("Completed flow tracing for session: {}", flow.session_id);
    }

    /// Trace an individual flow event with proper OpenTelemetry spans
    #[instrument(skip(self, event), fields(event_type = ?event.event_type, depth = event.depth))]
    pub fn trace_flow_event(&self, event: &FlowEvent) {
        if !self.enabled {
            return;
        }

        let span = Span::current();
        let event_name = match &event.event_type {
            FlowEventType::LlmRequest => {
                span.record("event_name", "LLM Request");
                "LLM Request"
            }
            FlowEventType::ToolCall => {
                span.record("event_name", "Tool Call");
                "Tool Call"
            }
            FlowEventType::ToolResult => {
                span.record("event_name", "Tool Result");
                "Tool Result"
            }
            FlowEventType::TransactionExecution => {
                span.record("event_name", "Transaction");
                "Transaction"
            }
            FlowEventType::Error => {
                span.record("event_name", "Error");
                "Error"
            }
            FlowEventType::BenchmarkStateChange => {
                span.record("event_name", "State Change");
                "State Change"
            }
        };

        debug!(
            "Tracing flow event: {} at depth {}",
            event_name, event.depth
        );

        // Record event-specific details as span attributes
        match &event.event_type {
            FlowEventType::LlmRequest => {
                if let Some(model) = event.content.data.get("model").and_then(|v| v.as_str()) {
                    span.record("model", model);
                    debug!("  - Model: {}", model);
                }
                if let Some(tokens) = event
                    .content
                    .data
                    .get("context_tokens")
                    .and_then(|v| v.as_u64())
                {
                    span.record("context_tokens", tokens as i64);
                    debug!("  - Context tokens: {}", tokens);
                }
            }
            FlowEventType::ToolCall => {
                if let Some(tool_name) =
                    event.content.data.get("tool_name").and_then(|v| v.as_str())
                {
                    span.record("tool_name", tool_name);
                    debug!("  - Tool: {}", tool_name);
                }
                if let Some(exec_time) = event
                    .content
                    .data
                    .get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                {
                    span.record("execution_time_ms", exec_time as i64);
                    debug!("  - Execution time: {}ms", exec_time);
                }
            }
            FlowEventType::Error => {
                if let Some(error_message) =
                    event.content.data.get("message").and_then(|v| v.as_str())
                {
                    span.record("error_message", error_message);
                    warn!("  - Error: {}", error_message);
                }
            }
            _ => {}
        }
    }

    /// Create a proper OpenTelemetry span for tool execution
    #[instrument(skip(self, args), fields(tool_name, args_length = args.len()))]
    pub fn create_tool_span(&self, tool_name: &str, args: &str) {
        if !self.enabled {
            return;
        }

        let span = Span::current();
        span.record("tool_name", tool_name);
        span.record("args_length", args.len() as i64);

        info!(
            "Tool execution started: {} (args length: {})",
            tool_name,
            args.len()
        );

        // Note: No async delay here - this is handled automatically by OpenTelemetry
        // The rig framework handles proper timing and ordering
    }

    /// Create a proper OpenTelemetry span for LLM interaction
    #[instrument(skip(self), fields(model, prompt_length))]
    pub fn create_llm_span(&self, model: &str, prompt_length: usize) {
        if !self.enabled {
            return;
        }

        let span = Span::current();
        span.record("model", model);
        span.record("prompt_length", prompt_length as i64);

        info!(
            "LLM interaction started: {} (prompt length: {})",
            model, prompt_length
        );
    }

    /// Record metrics for performance monitoring with proper attributes
    #[instrument(skip(self, flow), fields(session_id = %flow.session_id))]
    pub fn record_metrics(&self, flow: &FlowLog) {
        if !self.enabled {
            return;
        }

        let span = Span::current();

        // Record flow duration
        if let Some(end) = flow.end_time {
            let start = flow.start_time;
            if let Ok(duration) = end.duration_since(start) {
                span.record("flow_duration_ms", duration.as_millis() as i64);
                info!("Flow duration: {}ms", duration.as_millis());
            }
        }

        // Record metrics
        if let Some(result) = &flow.final_result {
            span.record("llm_calls", result.statistics.total_llm_calls as i64);
            span.record("tool_calls", result.statistics.total_tool_calls as i64);
            span.record("total_tokens", result.statistics.total_tokens as i64);
            span.record("success_rate", if result.success { 100.0 } else { 0.0 });

            info!("Metrics for session {}:", flow.session_id);
            info!("  - LLM calls: {}", result.statistics.total_llm_calls);
            info!("  - Tool calls: {}", result.statistics.total_tool_calls);
            info!("  - Total tokens: {}", result.statistics.total_tokens);
            info!(
                "  - Success rate: {:.1}%",
                if result.success { 100.0 } else { 0.0 }
            );
        }

        debug!("Recorded flow metrics for session: {}", flow.session_id);
    }
}

/// Initialize proper OpenTelemetry flow tracing with rig integration
pub async fn init_flow_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let enabled = std::env::var("REEV_OTEL_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if !enabled {
        info!("OpenTelemetry flow tracing initialization skipped");
        return Ok(());
    }

    info!("Initializing OpenTelemetry flow tracing with rig integration...");

    let service_name =
        std::env::var("REEV_OTEL_SERVICE_NAME").unwrap_or_else(|_| "reev".to_string());

    // Initialize the tracing subscriber with OpenTelemetry layer
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(service_name).build())
        .build();

    let tracer = provider.tracer("reev-flow");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer().pretty();

    // Initialize the global subscriber
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    info!(
        "OpenTelemetry flow tracing initialized with service: {}",
        service_name
    );
    info!("Tool calls will be properly traced without breaking the rig framework");

    Ok(())
}
