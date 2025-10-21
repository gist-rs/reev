//! Simple OpenTelemetry integration for flow tracing
//!
//! This module provides simple tracing with OpenTelemetry backend for tool call
//! monitoring. Uses stdout exporter with file output following the pattern from
//! opentelemetry_stdout examples.

use super::types::*;
use opentelemetry::global;
use opentelemetry_sdk::trace as sdktrace;
use std::fs::File;
use tracing::{debug, info, info_span, instrument, warn, Span};
use tracing_subscriber::prelude::*;

/// Simple flow tracer with OpenTelemetry backend
pub struct FlowTracer {
    enabled: bool,
}

impl FlowTracer {
    /// Create a new flow tracer
    pub fn new() -> Self {
        let enabled = std::env::var("REEV_OTEL_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(false);

        if enabled {
            info!("Flow tracing enabled with OpenTelemetry backend");
            Self { enabled: true }
        } else {
            info!("Flow tracing disabled");
            Self { enabled: false }
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

    /// Trace a complete flow log
    pub fn trace_flow(&self, flow: &FlowLog) {
        if !self.enabled {
            return;
        }

        let span = info_span!(
            "flow_execution",
            session_id = %flow.session_id,
            benchmark_id = %flow.benchmark_id,
            agent = %flow.agent_type,
            events_count = flow.events.len()
        );
        let _guard = span.enter();

        info!("Tracing flow execution for session: {}", flow.session_id);
        info!("  - Benchmark: {}", flow.benchmark_id);
        info!("  - Agent: {}", flow.agent_type);
        info!("  - Events: {}", flow.events.len());

        // Trace each event in the flow
        for event in &flow.events {
            self.trace_flow_event(event);
        }

        // Log final result
        if let Some(result) = &flow.final_result {
            tracing::info!(
                success = result.success,
                score = result.score,
                total_time_ms = result.total_time_ms,
                llm_calls = result.statistics.total_llm_calls,
                tool_calls = result.statistics.total_tool_calls,
                "Flow completed"
            );
        }

        tracing::debug!("Completed flow tracing for session: {}", flow.session_id);
    }

    /// Trace an individual flow event
    pub fn trace_flow_event(&self, event: &FlowEvent) {
        if !self.enabled {
            return;
        }

        let event_name = match &event.event_type {
            FlowEventType::LlmRequest => "LLM Request",
            FlowEventType::ToolCall => "Tool Call",
            FlowEventType::ToolResult => "Tool Result",
            FlowEventType::TransactionExecution => "Transaction",
            FlowEventType::Error => "Error",
            FlowEventType::BenchmarkStateChange => "State Change",
        };

        let span = info_span!("flow_event", event_type = event_name, depth = event.depth);
        let _guard = span.enter();

        debug!(
            "Tracing flow event: {} at depth {}",
            event_name, event.depth
        );

        // Record event-specific details
        match &event.event_type {
            FlowEventType::LlmRequest => {
                if let Some(model) = event.content.data.get("model").and_then(|v| v.as_str()) {
                    tracing::info!(model, "LLM Request");
                    debug!("  - Model: {}", model);
                }
                if let Some(tokens) = event
                    .content
                    .data
                    .get("context_tokens")
                    .and_then(|v| v.as_u64())
                {
                    tracing::info!(context_tokens = tokens as i64, "LLM Request tokens");
                    debug!("  - Context tokens: {}", tokens);
                }
            }
            FlowEventType::ToolCall => {
                if let Some(tool_name) =
                    event.content.data.get("tool_name").and_then(|v| v.as_str())
                {
                    tracing::info!(tool_name, "Tool Call");
                    debug!("  - Tool: {}", tool_name);
                }
                if let Some(exec_time) = event
                    .content
                    .data
                    .get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                {
                    tracing::info!(execution_time_ms = exec_time as i64, "Tool execution time");
                    debug!("  - Execution time: {}ms", exec_time);
                }
            }
            FlowEventType::Error => {
                if let Some(error_message) =
                    event.content.data.get("message").and_then(|v| v.as_str())
                {
                    tracing::error!(error_message, "Flow event error");
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
    /// Record metrics for performance monitoring
    pub fn record_metrics(&self, flow: &FlowLog) {
        if !self.enabled {
            return;
        }

        let span = info_span!("flow_metrics", session_id = %flow.session_id);
        let _guard = span.enter();

        // Record flow duration
        if let Some(end) = flow.end_time {
            let start = flow.start_time;
            if let Ok(duration) = end.duration_since(start) {
                tracing::info!(
                    flow_duration_ms = duration.as_millis() as i64,
                    "Flow duration"
                );
                info!("Flow duration: {}ms", duration.as_millis());
            }
        }

        // Record metrics
        if let Some(result) = &flow.final_result {
            tracing::info!(
                llm_calls = result.statistics.total_llm_calls as i64,
                tool_calls = result.statistics.total_tool_calls as i64,
                total_tokens = result.statistics.total_tokens as i64,
                success_rate = if result.success { 100.0 } else { 0.0 },
                "Flow metrics"
            );

            info!("Metrics for session {}:", flow.session_id);
            info!("  - LLM calls: {}", result.statistics.total_llm_calls);
            info!("  - Tool calls: {}", result.statistics.total_tool_calls);
            info!("  - Total tokens: {}", result.statistics.total_tokens);
            info!(
                "  - Success rate: {:.1}%",
                if result.success { 100.0 } else { 0.0 }
            );
        }

        tracing::debug!("Recorded flow metrics for session: {}", flow.session_id);
    }
}

/// Initialize flow tracing with file output
pub fn init_flow_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let enabled = std::env::var("REEV_OTEL_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(false);

    if !enabled {
        info!("Flow tracing initialization skipped");
        return Ok(());
    }

    info!("Initializing flow tracing with file output...");

    // Set up stdout exporter for OpenTelemetry traces
    let exporter = opentelemetry_stdout::SpanExporter::default();

    let tracer_provider = sdktrace::SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    global::set_tracer_provider(tracer_provider);

    // Specify the log file
    let log_file = std::env::var("REEV_TRACE_FILE").unwrap_or_else(|_| "traces.log".to_string());
    let file = File::create(&log_file)?;
    let log_file_for_info = log_file.clone();

    // Set up tracing subscriber to write to the file
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(move || {
                    file.try_clone()
                        .unwrap_or_else(|_| std::fs::File::create(&log_file).unwrap())
                }),
        )
        .init();

    info!(
        "Flow tracing initialized with file output: {}",
        log_file_for_info
    );
    info!("Tool calls will be automatically traced to file");

    Ok(())
}

/// Shutdown tracer provider
pub fn shutdown_tracer_provider() {
    info!("Shutting down tracer provider");
    // Note: No explicit shutdown needed for stdout exporter
}
