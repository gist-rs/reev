//! Parse Step As Tool Call Module
//!
//! This module provides function for parsing individual steps as tool calls (for backward compatibility).

use serde_json::Value as JsonValue;
use tracing::debug;

use super::types::ParsedToolCall;

/// Parse a step as a tool call (for backward compatibility)
pub fn parse_step_as_tool_call(step: &JsonValue, step_index: usize) -> Option<ParsedToolCall> {
    if let (Some(step_type), Some(data)) = (
        step.get("step_type").and_then(|v| v.as_str()),
        step.get("data"),
    ) {
        match step_type {
            "tool_call" => {
                if let (Some(tool_name), Some(params), Some(start_time)) = (
                    data.get("tool_name").and_then(|v| v.as_str()),
                    data.get("params").cloned(),
                    data.get("start_time").and_then(|v| v.as_u64()),
                ) {
                    return Some(ParsedToolCall {
                        tool_name: tool_name.to_string(),
                        start_time,
                        duration_ms: data
                            .get("duration_ms")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1000),
                        params,
                        result_data: data.get("result").cloned(),
                        tool_args: None,
                        extra_data: None,
                        success: true,
                    });
                }
            }
            "solana_instruction" => {
                // Parse Solana instruction as tool call
                if let (Some(program_id), Some(instruction_data), Some(accounts)) = (
                    data.get("program_id").and_then(|v| v.as_str()),
                    data.get("data").and_then(|v| v.as_str()),
                    data.get("accounts").and_then(|v| v.as_array()),
                ) {
                    let mut params = serde_json::Map::new();
                    params.insert(
                        "program".to_string(),
                        JsonValue::String(program_id.to_string()),
                    );
                    params.insert(
                        "data".to_string(),
                        JsonValue::String(instruction_data.to_string()),
                    );
                    params.insert(
                        "data_length".to_string(),
                        JsonValue::Number(instruction_data.len().into()),
                    );

                    // Extract key accounts
                    if !accounts.is_empty() {
                        if let Some(from_pubkey) =
                            accounts[0].get("pubkey").and_then(|v| v.as_str())
                        {
                            params.insert(
                                "from".to_string(),
                                JsonValue::String(from_pubkey.to_string()),
                            );
                        }

                        if accounts.len() > 1 {
                            if let Some(to_pubkey) =
                                accounts[1].get("pubkey").and_then(|v| v.as_str())
                            {
                                params.insert(
                                    "to".to_string(),
                                    JsonValue::String(to_pubkey.to_string()),
                                );
                            }
                        }
                    }

                    return Some(ParsedToolCall {
                        tool_name: format!("solana_{program_id}"),
                        start_time: data.get("start_time").and_then(|v| v.as_u64()).unwrap_or(0),
                        duration_ms: data
                            .get("duration_ms")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1000),
                        params: JsonValue::Object(params),
                        result_data: data.get("result").cloned(),
                        tool_args: None,
                        extra_data: None,
                        success: true,
                    });
                }
            }
            _ => {}
        }
    }

    debug!("Step {}: No valid tool call found", step_index);

    None
}
