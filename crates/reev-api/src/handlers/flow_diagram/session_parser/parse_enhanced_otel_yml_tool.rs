//! Parse Enhanced OTEL YAML Tool Module
//!
//! This module provides function for parsing enhanced OTEL YAML tool calls from tool_calls array.

use serde_json::Value as JsonValue;

use super::{parse_timestamp_to_unix, types::ParsedToolCall};
use crate::handlers::flow_diagram::FlowDiagramError;

/// Parse enhanced_otel YAML tool call from tool_calls array
pub fn parse_enhanced_otel_yml_tool(tool: &JsonValue) -> Result<ParsedToolCall, FlowDiagramError> {
    let tool_name = tool
        .get("tool_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            FlowDiagramError::InvalidLogFormat(
                "Missing tool_name in enhanced_otel YAML".to_string(),
            )
        })?;

    // Parse start_time - handle both string and numeric formats
    let start_time = tool
        .get("start_time")
        .and_then(|v| v.as_str())
        .and_then(parse_timestamp_to_unix)
        .or_else(|| tool.get("start_time").and_then(|v| v.as_u64()))
        .unwrap_or_default();

    // Parse duration_ms
    let duration_ms = tool
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Parse input parameters
    let params = tool
        .get("input")
        .or_else(|| tool.get("tool_args"))
        .cloned()
        .unwrap_or(JsonValue::Null);

    // Parse result data
    let result_data = tool
        .get("output")
        .or_else(|| tool.get("tool_output"))
        .cloned();

    // Extract original tool arguments as JSON string
    let tool_args = tool.get("tool_input").cloned().map(|v| v.to_string());

    Ok(ParsedToolCall {
        tool_name: tool_name.to_string(),
        start_time,
        duration_ms,
        params,
        result_data,
        tool_args,
    })
}
