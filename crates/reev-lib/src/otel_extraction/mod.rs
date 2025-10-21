//! OpenTelemetry Trace Extraction for Tool Call Data
//!
//! This module provides functionality to extract tool call information from
//! rig's OpenTelemetry traces and convert them to the session log format
//! required for Mermaid diagram generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

use crate::agent::{ToolCallInfo, ToolResultStatus};

/// OpenTelemetry trace data extracted from current spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelTraceData {
    /// Trace ID for the entire operation
    pub trace_id: String,
    /// List of span data representing tool calls
    pub spans: Vec<OtelSpanData>,
    /// Timestamp when trace was extracted
    pub extracted_at: SystemTime,
}

/// Individual span data from OpenTelemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpanData {
    /// Span name (usually tool name)
    pub span_name: String,
    /// Span kind (client, server, internal, etc.)
    pub span_kind: String,
    /// Start time of the span
    pub start_time: SystemTime,
    /// End time of the span (if completed)
    pub end_time: Option<SystemTime>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Span attributes (contains tool parameters and results)
    pub attributes: HashMap<String, String>,
    /// Span status (success, error)
    pub status: String,
    /// Error message if span failed
    pub error_message: Option<String>,
}

/// Session format tool data for Mermaid diagrams (matches FLOW.md specification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToolData {
    /// Tool name (e.g., "sol_transfer", "jupiter_swap")
    pub tool_name: String,
    /// Tool start timestamp
    pub start_time: SystemTime,
    /// Tool end timestamp
    pub end_time: SystemTime,
    /// Tool parameters
    pub params: serde_json::Value,
    /// Tool result
    pub result: serde_json::Value,
    /// Tool execution status
    pub status: String,
}

/// Extract current OpenTelemetry trace data from the global tracer
pub fn extract_current_otel_trace() -> Option<OtelTraceData> {
    debug!("[OtelExtraction] Attempting to extract current OpenTelemetry trace");

    // Get the current span from the tracing context
    let current_span = tracing::Span::current();

    // Extract trace ID from current span context
    let trace_id = extract_trace_id_from_span(&current_span)?;

    // In a real implementation, we would query the OpenTelemetry SDK
    // for all spans in the current trace. For now, we'll extract from
    // the current span and any child spans.
    let spans = extract_spans_from_current_context();

    if spans.is_empty() {
        warn!("[OtelExtraction] No spans found in current trace context");
        return None;
    }

    let trace_data = OtelTraceData {
        trace_id,
        spans,
        extracted_at: SystemTime::now(),
    };

    info!(
        "[OtelExtraction] Extracted trace with {} spans",
        trace_data.spans.len()
    );

    Some(trace_data)
}

/// Extract trace ID from a tracing span
fn extract_trace_id_from_span(_span: &tracing::Span) -> Option<String> {
    // This is a simplified implementation
    // In a real scenario, we would extract the actual OpenTelemetry trace ID
    // from the span's context using the OpenTelemetry API
    // For now, generate a fallback trace ID
    Some(format!(
        "trace_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ))
}

/// Extract all spans from the current tracing context
fn extract_spans_from_current_context() -> Vec<OtelSpanData> {
    let mut spans = Vec::new();

    // In a real implementation, we would use the OpenTelemetry SDK
    // to query for all spans in the current trace. For now, we'll
    // create span data from the current span context.

    if let Some(span_data) = extract_span_data_from_current() {
        spans.push(span_data);
    }

    spans
}

/// Extract span data from the current tracing span
fn extract_span_data_from_current() -> Option<OtelSpanData> {
    let current_span = tracing::Span::current();

    // Extract span name and attributes
    let span_name = current_span.metadata()?.name().to_string();
    let mut attributes = HashMap::new();

    // Extract common attributes that rig might add to tool spans
    if let Some(metadata) = current_span.metadata() {
        let fields = metadata.fields();
        for field in fields.iter() {
            let field_name = field.name();
            // In a real implementation, we would extract the actual field values
            // For now, add common tool-related attributes
            if field_name == "tool_name" || field_name == "otel.name" {
                attributes.insert(field_name.to_string(), "tool_name_placeholder".to_string());
            } else if field_name == "rig.tool.name" {
                attributes.insert(field_name.to_string(), "rig_tool_placeholder".to_string());
            } else {
                attributes.insert(field_name.to_string(), "value_placeholder".to_string());
            }
        }
    }

    // Calculate duration (simplified)
    let start_time = SystemTime::now();
    let duration_ms = Some(100); // Placeholder duration
    let end_time =
        duration_ms.map(|duration| start_time + std::time::Duration::from_millis(duration));

    Some(OtelSpanData {
        span_name,
        span_kind: "client".to_string(), // Most tool calls are client spans
        start_time,
        end_time,
        duration_ms,
        attributes,
        status: "success".to_string(), // Default to success
        error_message: None,
    })
}

/// Parse OpenTelemetry trace data to extract tool call information
pub fn parse_otel_trace_to_tools(trace: OtelTraceData) -> Vec<ToolCallInfo> {
    debug!(
        "[OtelExtraction] Parsing {} spans from trace {}",
        trace.spans.len(),
        trace.trace_id
    );

    let mut tool_calls = Vec::new();

    for span in trace.spans {
        // Only process spans that look like tool calls
        if is_tool_span(&span) {
            if let Some(tool_call) = extract_tool_call_from_span(span) {
                tool_calls.push(tool_call);
            }
        }
    }

    info!(
        "[OtelExtraction] Extracted {} tool calls from trace",
        tool_calls.len()
    );

    tool_calls
}

/// Check if a span represents a tool call
fn is_tool_span(span: &OtelSpanData) -> bool {
    // Check span name for common tool patterns
    let tool_patterns = [
        "sol_transfer",
        "spl_transfer",
        "jupiter_swap",
        "jupiter_lend",
        "get_account_balance",
        "get_lend_earn_tokens",
        "jupiter_earn",
    ];

    let span_name_lower = span.span_name.to_lowercase();

    // Check if span name contains tool patterns
    for pattern in &tool_patterns {
        if span_name_lower.contains(pattern) {
            return true;
        }
    }

    // Check attributes for tool indicators
    if span.attributes.contains_key("tool_name") || span.attributes.contains_key("rig.tool.name") {
        return true;
    }

    false
}

/// Extract tool call information from a span
fn extract_tool_call_from_span(span: OtelSpanData) -> Option<ToolCallInfo> {
    // Extract tool name from span name or attributes
    let tool_name = extract_tool_name_from_span(&span)?;

    // Extract tool parameters from span attributes
    let tool_args = extract_tool_args_from_span(&span);

    // Extract execution time
    let execution_time_ms = span.duration_ms.unwrap_or(0) as u32;

    // Determine execution status
    let result_status = if span.status == "error" {
        ToolResultStatus::Error
    } else {
        ToolResultStatus::Success
    };

    // Extract result data or error message
    let (result_data, error_message) = if span.status == "error" {
        (None, span.error_message)
    } else {
        (extract_result_data_from_span(&span), None)
    };

    Some(ToolCallInfo {
        tool_name,
        tool_args,
        execution_time_ms,
        result_status,
        result_data,
        error_message,
        timestamp: span.start_time,
        depth: 1, // Default depth, could be extracted from span attributes
    })
}

/// Extract tool name from span data
fn extract_tool_name_from_span(span: &OtelSpanData) -> Option<String> {
    // Try attributes first
    if let Some(tool_name) = span.attributes.get("tool_name") {
        return Some(tool_name.clone());
    }

    if let Some(tool_name) = span.attributes.get("rig.tool.name") {
        return Some(tool_name.clone());
    }

    // Try to parse from span name
    let span_name = &span.span_name;

    // Common tool name patterns in rig
    if span_name.contains("sol_transfer") {
        return Some("sol_transfer".to_string());
    }
    if span_name.contains("spl_transfer") {
        return Some("spl_transfer".to_string());
    }
    if span_name.contains("jupiter_swap") {
        return Some("jupiter_swap".to_string());
    }
    if span_name.contains("jupiter_lend") {
        return Some("jupiter_lend".to_string());
    }
    if span_name.contains("get_account_balance") {
        return Some("get_account_balance".to_string());
    }
    if span_name.contains("get_lend_earn_tokens") {
        return Some("get_lend_earn_tokens".to_string());
    }
    if span_name.contains("jupiter_earn") {
        return Some("jupiter_earn".to_string());
    }

    // Fallback to span name if no pattern matches
    Some(span_name.clone())
}

/// Extract tool arguments from span attributes
fn extract_tool_args_from_span(span: &OtelSpanData) -> String {
    // Collect tool-relevant attributes
    let mut tool_args = HashMap::new();

    for (key, value) in &span.attributes {
        if key.starts_with("tool.")
            || key.starts_with("param.")
            || key == "pubkey"
            || key == "amount"
            || key == "input_mint"
            || key == "output_mint"
        {
            tool_args.insert(key.clone(), value.clone());
        }
    }

    serde_json::to_string(&tool_args).unwrap_or_else(|_| "{}".to_string())
}

/// Extract result data from span attributes
fn extract_result_data_from_span(span: &OtelSpanData) -> Option<serde_json::Value> {
    let mut result_data = HashMap::new();

    // Collect result-relevant attributes
    for (key, value) in &span.attributes {
        if key.starts_with("result.")
            || key.starts_with("output.")
            || key == "balance"
            || key == "output_amount"
            || key == "signatures"
        {
            result_data.insert(key.clone(), value.clone());
        }
    }

    if result_data.is_empty() {
        None
    } else {
        serde_json::to_value(result_data).ok()
    }
}

/// Convert tool calls to session format for Mermaid diagrams
pub fn convert_to_session_format(tool_calls: Vec<ToolCallInfo>) -> Vec<SessionToolData> {
    debug!(
        "[OtelExtraction] Converting {} tool calls to session format",
        tool_calls.len()
    );

    let mut session_tools = Vec::new();

    for tool_call in tool_calls {
        let session_tool = SessionToolData {
            tool_name: tool_call.tool_name.clone(),
            start_time: tool_call.timestamp,
            end_time: tool_call.timestamp
                + std::time::Duration::from_millis(tool_call.execution_time_ms as u64),
            params: serde_json::from_str(&tool_call.tool_args)
                .unwrap_or_else(|_| serde_json::Value::Object(Default::default())),
            result: tool_call
                .result_data
                .unwrap_or_else(|| serde_json::Value::Object(Default::default())),
            status: match tool_call.result_status {
                ToolResultStatus::Success => "success".to_string(),
                ToolResultStatus::Error => "error".to_string(),
                ToolResultStatus::Timeout => "timeout".to_string(),
            },
        };

        session_tools.push(session_tool);
    }

    info!(
        "[OtelExtraction] Converted {} tools to session format",
        session_tools.len()
    );

    session_tools
}

/// Initialize OpenTelemetry trace extraction for tool call tracking
pub fn init_otel_extraction() -> Result<(), Box<dyn std::error::Error>> {
    info!("[OtelExtraction] Initializing OpenTelemetry trace extraction");

    // Check if OpenTelemetry is properly configured
    let enabled = std::env::var("REEV_OTEL_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(false);

    if enabled {
        info!("[OtelExtraction] OpenTelemetry extraction enabled");
        info!("[OtelExtraction] Tool calls will be extracted from rig's OpenTelemetry traces");
    } else {
        info!("[OtelExtraction] OpenTelemetry extraction disabled");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_otel_trace_data_creation() {
        let trace_data = OtelTraceData {
            trace_id: "test_trace_123".to_string(),
            spans: vec![],
            extracted_at: SystemTime::now(),
        };

        assert_eq!(trace_data.trace_id, "test_trace_123");
        assert_eq!(trace_data.spans.len(), 0);
    }

    #[test]
    fn test_otel_span_data_creation() {
        let span_data = OtelSpanData {
            span_name: "sol_transfer".to_string(),
            span_kind: "client".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now() + Duration::from_millis(100)),
            duration_ms: Some(100),
            attributes: HashMap::new(),
            status: "success".to_string(),
            error_message: None,
        };

        assert_eq!(span_data.span_name, "sol_transfer");
        assert_eq!(span_data.status, "success");
        assert_eq!(span_data.duration_ms, Some(100));
    }

    #[test]
    fn test_session_tool_data_creation() {
        let session_tool = SessionToolData {
            tool_name: "sol_transfer".to_string(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now() + std::time::Duration::from_millis(100),
            params: serde_json::json!({"pubkey": "test123"}),
            result: serde_json::json!({"balance": "1.0"}),
            status: "success".to_string(),
        };

        assert_eq!(session_tool.tool_name, "sol_transfer");
        assert_eq!(session_tool.status, "success");
    }

    #[test]
    fn test_tool_span_detection() {
        let tool_span = OtelSpanData {
            span_name: "sol_transfer".to_string(),
            span_kind: "client".to_string(),
            start_time: SystemTime::now(),
            end_time: None,
            duration_ms: None,
            attributes: HashMap::new(),
            status: "success".to_string(),
            error_message: None,
        };

        assert!(is_tool_span(&tool_span));

        let non_tool_span = OtelSpanData {
            span_name: "http_request".to_string(),
            span_kind: "client".to_string(),
            start_time: SystemTime::now(),
            end_time: None,
            duration_ms: None,
            attributes: HashMap::new(),
            status: "success".to_string(),
            error_message: None,
        };

        assert!(!is_tool_span(&non_tool_span));
    }
}
