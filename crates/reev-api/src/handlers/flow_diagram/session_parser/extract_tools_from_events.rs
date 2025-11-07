//! Extract Tools From Events Module
//!
//! This module provides function for extracting tool calls from events array (backward compatibility).

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::debug;

use super::types::ParsedToolCall;

/// Extract tool calls from events array (backward compatibility)
pub fn extract_tools_from_events(events: &[JsonValue], tool_calls: &mut Vec<ParsedToolCall>) {
    let mut tool_starts = HashMap::new();

    for event in events {
        if let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) {
            match event_type {
                "ToolCall" => {
                    if let (Some(tool_name), Some(start_time), Some(params)) = (
                        event
                            .get("data")
                            .and_then(|d| d.get("tool_name"))
                            .and_then(|v| v.as_str()),
                        event
                            .get("data")
                            .and_then(|d| d.get("start_time"))
                            .and_then(|v| v.as_u64()),
                        event.get("data").and_then(|d| d.get("params")),
                    ) {
                        tool_starts.insert(tool_name.to_string(), (start_time, params.clone()));
                    }
                }
                "ToolResult" => {
                    if let (Some(tool_name), Some(end_time), Some(_result), Some(_status)) = (
                        event
                            .get("data")
                            .and_then(|d| d.get("tool_name"))
                            .and_then(|v| v.as_str()),
                        event
                            .get("data")
                            .and_then(|d| d.get("end_time"))
                            .and_then(|v| v.as_u64()),
                        event.get("data").and_then(|d| d.get("result")),
                        event.get("data").and_then(|d| d.get("status")),
                    ) {
                        if let Some((start_time, params)) = tool_starts.remove(tool_name) {
                            let duration_ms = end_time.saturating_sub(start_time);
                            tool_calls.push(ParsedToolCall {
                                tool_name: tool_name.to_string(),
                                start_time,
                                duration_ms,
                                params: params.clone(),
                                result_data: event
                                    .get("data")
                                    .and_then(|d| d.get("result"))
                                    .cloned(),
                                tool_args: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }
    debug!("Extracted {} tool calls from events", tool_calls.len());
}
