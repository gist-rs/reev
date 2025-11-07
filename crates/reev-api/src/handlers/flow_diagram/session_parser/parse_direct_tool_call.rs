//! Parse Direct Tool Call Module
//!
//! This module provides function for parsing direct ToolCallSummary format from enhanced dynamic flows.

use serde_json::Value as JsonValue;
use tracing::debug;

use super::types::ParsedToolCall;
use crate::handlers::flow_diagram::FlowDiagramError;

/// Parse direct ToolCallSummary format from enhanced dynamic flows
pub fn parse_direct_tool_call(
    tool: &JsonValue,
    index: usize,
) -> Result<ParsedToolCall, FlowDiagramError> {
    let tool_name = tool
        .get("tool_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            FlowDiagramError::InvalidLogFormat("Missing tool_name in direct tool call".to_string())
        })?;

    // Parse timestamp
    let start_time = tool
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .and_then(|ts| ts.try_into().ok())
        .unwrap_or(0);

    // Parse duration
    let duration_ms = tool
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Parse params (new field)
    let params = tool.get("params").cloned().unwrap_or(JsonValue::Null);

    // Parse result_data (new field)
    let result_data = tool.get("result_data").cloned();

    // Parse tool_args (new field)
    let tool_args = tool
        .get("tool_args")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    debug!(
        "Parsed direct tool {}: {} ({}ms, params: {}, result: {}, args: {:?})",
        index,
        tool_name,
        duration_ms,
        !params.is_null(),
        result_data.is_some(),
        tool_args
    );

    Ok(ParsedToolCall {
        tool_name: tool_name.to_string(),
        start_time,
        duration_ms,
        params,
        result_data,
        tool_args,
    })
}
