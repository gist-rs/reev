//! Session Log Parser
//!
//! This module handles parsing of enhanced session logs to extract tool calls
//! and execution information for flow diagram generation.

use crate::handlers::flow_diagram::{DiagramMetadata, FlowDiagramError};
#[cfg(feature = "direct_runner")]
#[allow(unused_imports)]
use reev_tools::tool_names;
use serde_json::Value;
use tracing::{debug, info};

// Import YAML parsing capabilities
use serde_yaml;

/// Session log parser for extracting tool calls and execution data
pub struct SessionParser;

/// Parsed session data suitable for diagram generation
#[derive(Debug, Clone)]
pub struct ParsedSession {
    /// Session identifier
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Extracted tool calls
    pub tool_calls: Vec<ParsedToolCall>,
    /// Session prompt
    pub prompt: Option<String>,
    /// Session start time
    pub start_time: u64,
    /// Session end time
    pub end_time: Option<u64>,
}

/// Parsed tool call information
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    /// Tool identifier
    pub tool_name: String,
    /// Tool start time (Unix timestamp)
    pub start_time: u64,
    /// Tool parameters
    pub params: Value,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Tool result data (includes instruction count, etc.)
    pub result_data: Option<Value>,
    /// Original tool arguments as JSON string
    pub tool_args: Option<String>,
}

impl SessionParser {
    /// Parse session log content and extract tool calls
    pub fn parse_session_content(content: &str) -> Result<ParsedSession, FlowDiagramError> {
        debug!("Parsing session content (length: {})", content.len());

        // Try JSON first, then YAML format
        let session_log: Value = if let Ok(json_value) = serde_json::from_str(content) {
            json_value
        } else if let Ok(yaml_value) = serde_yaml::from_str(content) {
            debug!("Parsed content as YAML format");
            yaml_value
        } else {
            return Err(FlowDiagramError::InvalidLogFormat(
                "Failed to parse content as both JSON and YAML".to_string(),
            ));
        };

        // Extract basic session information
        let session_id = session_log
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing session_id".to_string()))?
            .to_string();

        let benchmark_id = session_log
            .get("benchmark_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let _agent_type = session_log
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let start_time = session_log
            .get("start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let end_time = session_log.get("end_time").and_then(|v| v.as_u64());

        // Extract prompt from different sources
        let prompt = if let Some(p) = session_log
            .get("final_result")
            .and_then(|fr| fr.get("data"))
            .and_then(|data| data.get("prompt"))
            .and_then(|p| p.as_str())
        {
            Some(p.to_string())
        } else if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str())
        {
            // Try to parse log_content as JSON to extract prompt
            if let Ok(log_json) = serde_json::from_str::<Value>(log_content) {
                log_json
                    .get("prompt")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
            } else {
                // Try to extract user_prompt from YAML-like format
                Self::extract_user_prompt_from_yaml(log_content)
            }
        } else {
            None
        };

        // Extract tool calls from multiple sources
        let mut tool_calls = Vec::new();

        // First try: log_content format (from enhanced_otel JSONL conversion)
        if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str()) {
            // Try to extract from log_content as YAML first (from enhanced_otel conversion)
            if let Ok(yaml_value) = serde_yaml::from_str::<Value>(log_content) {
                if let Some(yml_tools) = yaml_value
                    .get("tool_calls")
                    .and_then(|tools| tools.as_array())
                {
                    debug!("Found {} tools in YAML log_content", yml_tools.len());
                    for tool in yml_tools {
                        if let Ok(parsed_tool) = Self::parse_enhanced_otel_yml_tool(tool) {
                            tool_calls.push(parsed_tool);
                        }
                    }
                }
            }
            // Fallback: Try to extract from log_content JSON format
            else if let Ok(log_json) = serde_json::from_str::<Value>(log_content) {
                if let Some(steps) = log_json.get("steps").and_then(|s| s.as_array()) {
                    debug!("Found {} steps in log_content", steps.len());
                    for (index, step) in steps.iter().enumerate() {
                        if let Some(tool_call) = Self::parse_step_as_tool_call(step, index) {
                            tool_calls.push(tool_call);
                        }
                    }
                }
            }
        }
        // Second try: Extract from events (backward compatibility)
        else if let Some(events) = session_log.get("events").and_then(|e| e.as_array()) {
            debug!("Found {} events, extracting tools", events.len());
            Self::extract_tools_from_events(events, &mut tool_calls);
        }

        // Sort tool calls by start time
        tool_calls.sort_by_key(|t| t.start_time);

        let _final_result = session_log.get("final_result").cloned();

        info!(
            "Parsed session {} with {} tool calls",
            session_id,
            tool_calls.len()
        );

        Ok(ParsedSession {
            session_id,
            benchmark_id,
            tool_calls,
            prompt,
            start_time,
            end_time,
        })
    }

    /// Parse enhanced_otel YAML tool call from tool_calls array
    fn parse_enhanced_otel_yml_tool(tool: &Value) -> Result<ParsedToolCall, FlowDiagramError> {
        let tool_name = tool
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowDiagramError::InvalidLogFormat(
                    "Missing tool_name in enhanced_otel YAML".to_string(),
                )
            })?
            .to_string();

        // Parse start_time from RFC3339 format to timestamp
        let start_time_str = tool
            .get("start_time")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowDiagramError::InvalidLogFormat(
                    "Missing start_time in enhanced_otel YAML".to_string(),
                )
            })?;

        let start_time = chrono::DateTime::parse_from_rfc3339(start_time_str)
            .map_err(|_| {
                FlowDiagramError::InvalidLogFormat(
                    "Invalid start_time format in enhanced_otel YAML".to_string(),
                )
            })?
            .timestamp_micros() as u64;

        let duration_ms = tool
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000); // Default to 1 second if missing

        let input_params = tool.get("input").cloned().unwrap_or(Value::Null);
        let output_result = tool.get("output").cloned();

        Ok(ParsedToolCall {
            tool_name,
            start_time,
            params: input_params,
            duration_ms,
            result_data: output_result,
            tool_args: None,
        })
    }

    /// Extract tool calls from events array (backward compatibility)
    fn extract_tools_from_events(events: &[Value], tool_calls: &mut Vec<ParsedToolCall>) {
        let mut tool_starts = std::collections::HashMap::new();

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
                            event
                                .get("data")
                                .and_then(|d| d.get("status"))
                                .and_then(|v| v.as_str()),
                        ) {
                            if let Some((start_time, params)) = tool_starts.remove(tool_name) {
                                let duration_ms = end_time.saturating_sub(start_time) * 1000;
                                tool_calls.push(ParsedToolCall {
                                    tool_name: tool_name.to_string(),
                                    start_time,
                                    params,
                                    duration_ms,
                                    result_data: None,
                                    tool_args: None,
                                });
                            }
                        }
                    }
                    _ => {
                        // Handle other event types that we don't need to process
                    }
                }
            }
        }

        debug!("Extracted {} tool calls from events", tool_calls.len());
    }

    /// Parse a step from log_content as a tool call
    fn parse_step_as_tool_call(step: &Value, step_index: usize) -> Option<ParsedToolCall> {
        if let (Some(action), Some(observation)) = (step.get("action"), step.get("observation")) {
            // Extract action details
            if let Some(action_array) = action.as_array() {
                if let Some(first_action) = action_array.first() {
                    let program_id = first_action
                        .get("program_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown_program");

                    let accounts_vec = first_action
                        .get("accounts")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let accounts = &accounts_vec;

                    let data = first_action
                        .get("data")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    // Create tool name based on program_id using centralized tool names
                    #[cfg(feature = "direct_runner")]
                    let tool_name = reev_tools::tool_names::tool_name_from_program_id(program_id);

                    #[cfg(not(feature = "direct_runner"))]
                    let tool_name = format!("program_{program_id}");

                    // Extract account information for parameters
                    let mut params = serde_json::Map::new();
                    params.insert("program".to_string(), Value::String(program_id.to_string()));
                    params.insert("data".to_string(), Value::String(data.to_string()));
                    params.insert("data_length".to_string(), Value::Number(data.len().into()));

                    if let Some(from_account) = accounts.first() {
                        if let Some(from_pubkey) =
                            from_account.get("pubkey").and_then(|v| v.as_str())
                        {
                            params
                                .insert("from".to_string(), Value::String(from_pubkey.to_string()));
                        }
                    }

                    if accounts.len() > 1 {
                        if let Some(to_account) = accounts.get(1) {
                            if let Some(to_pubkey) =
                                to_account.get("pubkey").and_then(|v| v.as_str())
                            {
                                params
                                    .insert("to".to_string(), Value::String(to_pubkey.to_string()));
                            }
                        }
                    }

                    // Extract observation result
                    let _result = observation
                        .get("last_transaction_status")
                        .and_then(|s| s.as_str())
                        .map(|status| {
                            let mut result_map = serde_json::Map::new();
                            result_map
                                .insert("status".to_string(), Value::String(status.to_string()));
                            Value::Object(result_map)
                        });

                    // Create tool call with mock timestamps
                    let start_time = step_index as u64;
                    let _end_time = start_time + 1;

                    return Some(ParsedToolCall {
                        tool_name,
                        start_time,
                        params: Value::Object(params),
                        duration_ms: 1000,
                        result_data: None,
                        tool_args: None,
                    });
                }
            }
        }
        None
    }

    /// Create diagram metadata from parsed session
    pub fn create_metadata(parsed: &ParsedSession) -> DiagramMetadata {
        let execution_time_ms = if let Some(end) = parsed.end_time {
            end.saturating_sub(parsed.start_time) * 1000
        } else {
            parsed.tool_calls.iter().map(|t| t.duration_ms).sum()
        };

        DiagramMetadata {
            state_count: 2 + parsed.tool_calls.len(), // Start + End + Tools
            tool_count: parsed.tool_calls.len(),
            execution_time_ms,
            benchmark_id: parsed.benchmark_id.clone(),
            session_id: Some(parsed.session_id.clone()),
        }
    }

    /// Extract user_prompt from YAML-like session log format
    fn extract_user_prompt_from_yaml(log_content: &str) -> Option<String> {
        // Look for user_prompt in the YAML-like format
        // It can be nested inside a "prompt:" section or at top level
        let lines: Vec<&str> = log_content.lines().collect();

        let mut in_main_prompt_section = false;
        let mut in_user_prompt_section = false;
        let mut prompt_lines = Vec::new();

        for line in lines {
            if line.trim() == "prompt:" {
                in_main_prompt_section = true;
                continue;
            } else if (line.trim() == "user_prompt:" || line.trim().starts_with("user_prompt:"))
                && in_main_prompt_section
            {
                in_user_prompt_section = true;
                // Skip the "user_prompt:" line itself
                continue;
            } else if in_user_prompt_section {
                // Handle both nested (4-space) and top-level (2-space) indented content
                let indent_level = if in_main_prompt_section { 4 } else { 2 };
                let indent_str = " ".repeat(indent_level);

                if line.starts_with(&indent_str) && !line.trim().is_empty() {
                    // This is a continuation of the prompt
                    prompt_lines.push(line.trim_start());
                } else if line.trim().is_empty() && !prompt_lines.is_empty() {
                    // Empty line after we've started collecting, might be part of prompt
                    prompt_lines.push("");
                } else if !line.starts_with(&indent_str)
                    && !line.trim().is_empty()
                    && !prompt_lines.is_empty()
                {
                    // End of prompt section - we've moved to a non-indented line after collecting content
                    break;
                }
            } else if in_main_prompt_section {
                // Skip other sections within prompt section
                if line.starts_with("  ") && line.trim().ends_with(":") {
                    // We've encountered another subsection, skip until we find user_prompt or exit prompt section
                    if line.trim() != "user_prompt:" {
                        in_user_prompt_section = false;
                    }
                }
            } else if line.trim() == "user_prompt:" || line.trim().starts_with("user_prompt:") {
                // Top level user_prompt (not nested)
                in_user_prompt_section = true;
                continue;
            }
        }

        if prompt_lines.is_empty() {
            None
        } else {
            // Filter out any trailing empty lines
            while prompt_lines.last() == Some(&"") {
                prompt_lines.pop();
            }
            Some(prompt_lines.join("\n"))
        }
    }
}
