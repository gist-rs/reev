//! Session Log Parser
//!
//! This module handles parsing of enhanced session logs to extract tool calls
//! and execution information for flow diagram generation.

use crate::handlers::flow_diagram::{DiagramMetadata, FlowDiagramError};
use reev_tools::tool_names;
use serde_json::Value;
use std::path::Path;
use tracing::{debug, info};

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
    /// Parse a session log file and extract tool calls
    pub async fn parse_session_file(file_path: &Path) -> Result<ParsedSession, FlowDiagramError> {
        info!("Parsing session file: {}", file_path.display());

        let content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| FlowDiagramError::ParseError(format!("Failed to read file: {e}")))?;

        Self::parse_session_content(&content)
    }

    /// Parse session log content and extract tool calls
    pub fn parse_session_content(content: &str) -> Result<ParsedSession, FlowDiagramError> {
        debug!("Parsing session content (length: {})", content.len());

        let session_log: Value = serde_json::from_str(content)
            .map_err(|e| FlowDiagramError::InvalidLogFormat(format!("JSON parsing failed: {e}")))?;

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
                None
            }
        } else {
            None
        };

        // Extract tool calls from multiple sources
        let mut tool_calls = Vec::new();

        // First try: Enhanced session logs with tools array
        if let Some(tools) = session_log
            .get("final_result")
            .and_then(|fr| fr.get("data"))
            .and_then(|data| data.get("tools"))
            .and_then(|tools| tools.as_array())
        {
            debug!("Found {} tools in enhanced session log", tools.len());
            for tool in tools {
                if let Ok(parsed_tool) = Self::parse_enhanced_tool(tool) {
                    tool_calls.push(parsed_tool);
                }
            }
        } else if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str())
        {
            // Try to extract from log_content JSON format
            if let Ok(log_json) = serde_json::from_str::<Value>(log_content) {
                if let Some(steps) = log_json.get("steps").and_then(|s| s.as_array()) {
                    debug!("Found {} steps in log_content", steps.len());
                    for (index, step) in steps.iter().enumerate() {
                        if let Some(tool_call) = Self::parse_step_as_tool_call(step, index) {
                            tool_calls.push(tool_call);
                        }
                    }
                }
            }
        } else {
            debug!("No tools array found, trying to extract from events");
            // Second try: Extract from events (backward compatibility)
            if let Some(events) = session_log.get("events").and_then(|e| e.as_array()) {
                Self::extract_tools_from_events(events, &mut tool_calls);
            }
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

    /// Parse enhanced tool call from tools array
    fn parse_enhanced_tool(tool: &Value) -> Result<ParsedToolCall, FlowDiagramError> {
        let tool_name = tool
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing tool_name".to_string()))?
            .to_string();

        let start_time = tool
            .get("start_time")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing start_time".to_string()))?;

        let end_time = tool
            .get("end_time")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| FlowDiagramError::InvalidLogFormat("Missing end_time".to_string()))?;

        let params = tool.get("params").cloned().unwrap_or(Value::Null);
        let result = tool.get("result").cloned();
        let tool_args = tool
            .get("tool_args")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let _status = tool
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let duration_ms = end_time.saturating_sub(start_time) * 1000;

        Ok(ParsedToolCall {
            tool_name,
            start_time,
            params,
            duration_ms,
            result_data: result,
            tool_args,
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
                    let tool_name = tool_names::tool_name_from_program_id(program_id);

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
}
