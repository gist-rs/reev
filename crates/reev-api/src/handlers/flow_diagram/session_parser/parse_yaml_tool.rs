//! Parse YAML Tool Module
//!
//! This module provides function for parsing individual tools from YAML values.

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

use super::{parse_timestamp_to_unix, types::ParsedToolCall, yaml_value_to_json};
use crate::handlers::flow_diagram::FlowDiagramError;

/// Parse a single tool from YAML value
pub fn parse_yaml_tool(tool: &YamlValue) -> Result<ParsedToolCall, FlowDiagramError> {
    let tool_name = tool
        .get("tool_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            FlowDiagramError::InvalidLogFormat("Missing tool_name in YAML tool".to_string())
        })?;

    // Parse start_time - handle both string and numeric formats
    let start_time = if let Some(time_str) = tool.get("start_time").and_then(|v| v.as_str()) {
        // Parse ISO 8601 timestamp to Unix timestamp
        parse_timestamp_to_unix(time_str).unwrap_or(0)
    } else {
        tool.get("start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or_default()
    };

    // Parse duration_ms
    let duration_ms = tool
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Parse input parameters - check for new ToolCallSummary fields first
    let params = tool
        .get("params")
        .or_else(|| tool.get("input"))
        .or_else(|| tool.get("tool_args"))
        .cloned()
        .map(|v| yaml_value_to_json(&v))
        .unwrap_or(JsonValue::Null);

    // Parse result data - check for new ToolCallSummary fields first
    let result_data = tool
        .get("result_data")
        .or_else(|| tool.get("output"))
        .or_else(|| tool.get("tool_output"))
        .cloned()
        .map(|v| yaml_value_to_json(&v));

    // Extract original tool arguments as JSON string - check for new field first
    let tool_args = tool
        .get("tool_args")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            tool.get("tool_input")
                .cloned()
                .map(|v| yaml_value_to_json(&v).to_string())
        });

    Ok(ParsedToolCall {
        tool_name: tool_name.to_string(),
        start_time,
        duration_ms,
        params,
        result_data,
        tool_args,
        extra_data: None,
        success: true,
    })
}
