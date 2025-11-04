//! Session Log Parser
//!
//! This module handles parsing of enhanced session logs to extract tool calls
//! and execution information for flow diagram generation.

// Import YAML parsing capabilities
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

use tracing::{debug, info, warn};

use super::{DiagramMetadata, FlowDiagramError};

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
    /// Original prompt (optional)
    pub prompt: Option<String>,
    /// Session start time
    pub start_time: u64,
    /// Session end time
    pub end_time: Option<u64>,
}

/// Parsed tool call information
#[derive(Debug, Clone, Serialize)]
pub struct ParsedToolCall {
    /// Tool identifier
    pub tool_name: String,
    /// Tool start time (Unix timestamp)
    pub start_time: u64,
    /// Tool parameters
    pub params: JsonValue,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Tool result data (includes instruction count, etc.)
    pub result_data: Option<JsonValue>,
    /// Raw tool arguments from agent execution
    pub tool_args: Option<String>,
}

impl SessionParser {
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

    /// Parse session log content and extract tool calls
    pub fn parse_session_content(content: &str) -> Result<ParsedSession, FlowDiagramError> {
        debug!("Parsing session content (length: {})", content.len());

        // Try JSON first, then YAML format
        let session_log: JsonValue = if let Ok(json_value) = serde_json::from_str(content) {
            json_value
        } else {
            // Try YAML parsing for OTEL-derived content
            let yaml_value: YamlValue = serde_yaml::from_str(content).map_err(|_| {
                FlowDiagramError::InvalidLogFormat(
                    "Failed to parse content as both JSON and YAML".to_string(),
                )
            })?;
            debug!("Parsed content as YAML format");

            // Convert YAML to JSON for consistent processing
            return parse_yaml_session_content(&yaml_value);
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

        let start_time = session_log
            .get("start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let end_time = session_log.get("end_time").and_then(|v| v.as_u64());

        // Extract tool calls
        let mut tool_calls = Vec::new();

        // NEW: Direct tool_calls format (for enhanced ToolCallSummary objects)
        if let Some(direct_tools) = session_log.get("tool_calls").and_then(|t| t.as_array()) {
            for (i, tool) in direct_tools.iter().enumerate() {
                if let Ok(parsed_tool) = parse_direct_tool_call(tool, i) {
                    tool_calls.push(parsed_tool);
                } else {
                    warn!("❌ Failed to parse direct tool {}: {}", i, "parsing error");
                }
            }
        }
        // First try: Extract from log_content (enhanced_otel format)
        else if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str())
        {
            debug!("🔍 Parsing log_content (length: {})", log_content.len());

            // Try YAML parsing first (for OTEL-derived data)
            match serde_yaml::from_str::<YamlValue>(log_content) {
                Ok(yaml_value) => {
                    debug!("✅ YAML parsing successful");
                    if let Some(yml_tools) =
                        yaml_value.get("tool_calls").and_then(|t| t.as_sequence())
                    {
                        debug!("🛠️  Found {} tools in YAML log_content", yml_tools.len());
                        for (i, tool) in yml_tools.iter().enumerate() {
                            debug!("🛠️  Parsing tool {}: {:?}", i, tool);
                            if let Ok(parsed_tool) = parse_yaml_tool(tool) {
                                tool_calls.push(parsed_tool);
                            } else {
                                warn!("❌ Failed to parse tool {}: {}", i, "parsing error");
                            }
                        }
                    } else {
                        debug!("⚠️  YAML parsed but no tool_calls array found");
                    }
                }
                Err(yaml_err) => {
                    warn!("❌ YAML parsing failed: {}", yaml_err);
                    info!("🔄 Trying JSON fallback");

                    // Fallback: Try to extract from log_content JSON format
                    if let Ok(log_json) = serde_json::from_str::<JsonValue>(log_content) {
                        if let Some(steps) = log_json.get("steps").and_then(|s| s.as_array()) {
                            debug!("Found {} steps in log_content", steps.len());
                            for (index, step) in steps.iter().enumerate() {
                                if let Some(tool_call) = parse_step_as_tool_call(step, index) {
                                    tool_calls.push(tool_call);
                                }
                            }
                        } else {
                            warn!("⚠️  JSON parsed but no steps array found");
                        }
                    } else {
                        warn!("❌ JSON fallback parsing also failed");
                    }
                }
            }
        }
        // Second try: Extract from events (backward compatibility)
        else if let Some(events) = session_log.get("events").and_then(|e| e.as_array()) {
            debug!("Found {} events, extracting tools", events.len());
            extract_tools_from_events(events, &mut tool_calls);
        }

        // Sort tool calls by start time
        tool_calls.sort_by_key(|t| t.start_time);

        let _final_result = session_log.get("final_result").cloned();

        info!(
            "Parsed session {} with {} tool calls",
            session_id,
            tool_calls.len()
        );

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
            if let Ok(log_json) = serde_json::from_str::<JsonValue>(log_content) {
                log_json
                    .get("prompt")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
            } else {
                // Try to extract user_prompt from YAML-like format
                SessionParser::extract_user_prompt_from_yaml(log_content)
            }
        } else {
            None
        };

        Ok(ParsedSession {
            session_id,
            benchmark_id,
            tool_calls,
            prompt,
            start_time,
            end_time,
        })
    }

    /// Extract user prompt from YAML-like log content
    fn extract_user_prompt_from_yaml(log_content: &str) -> Option<String> {
        // Look for user_prompt pattern in the text
        for line in log_content.lines() {
            if line.trim().starts_with("user_prompt:") {
                let prompt_start = line.find(':').unwrap_or(0) + 1;
                let prompt = line[prompt_start..].trim().trim_matches('"');
                return Some(prompt.to_string());
            }
        }
        None
    }
}

/// Parse YAML session content from OTEL-derived data
fn parse_yaml_session_content(yaml_value: &YamlValue) -> Result<ParsedSession, FlowDiagramError> {
    debug!("Parsing YAML session content from OTEL data");

    // Extract session_id from YAML structure
    let session_id = yaml_value
        .get("session_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            // Look for session_id in nested structure (300-series format)
            yaml_value
                .as_mapping()
                .and_then(|map| map.keys().find(|k| k.as_str() == Some("session_id")))
                .and_then(|k| yaml_value.get(k))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("unknown")
        .to_string();

    // Extract benchmark_id
    let benchmark_id = yaml_value
        .get("benchmark_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract tool_calls from YAML structure
    let mut tool_calls = Vec::new();

    // Method 1: Direct tool_calls array (001-series format)
    if let Some(tools) = yaml_value.get("tool_calls").and_then(|t| t.as_sequence()) {
        debug!("Found {} tools in direct tool_calls array", tools.len());
        for tool in tools {
            if let Ok(parsed_tool) = parse_yaml_tool(tool) {
                tool_calls.push(parsed_tool);
            }
        }
    }
    // Method 2: Look through 300-series YAML structure with headers and comments
    else {
        debug!("Looking for tools in 300-series YAML structure");
        find_tool_calls_in_300_series_yaml_structure(yaml_value, &mut tool_calls);
    }

    // Sort tool calls by start time
    tool_calls.sort_by_key(|t| t.start_time);

    info!(
        "Parsed YAML session {} with {} tool calls",
        session_id,
        tool_calls.len()
    );

    Ok(ParsedSession {
        session_id,
        benchmark_id,
        tool_calls,
        prompt: None, // YAML format may not contain prompt
        start_time: 0,
        end_time: None,
    })
}

/// Find tool calls in 300-series YAML structure with headers and comments
fn find_tool_calls_in_300_series_yaml_structure(
    yaml_value: &YamlValue,
    tool_calls: &mut Vec<ParsedToolCall>,
) {
    // Look through the YAML mapping for tool call entries
    if let Some(mapping) = yaml_value.as_mapping() {
        for (key, value) in mapping {
            // Look for keys that contain tool information
            if let Some(key_str) = key.as_str() {
                if key_str.contains("tool_call") || key_str.contains("jupiter") {
                    debug!("Found potential tool entry: {}", key_str);
                    if let Some(tool_sequence) = value.as_sequence() {
                        for tool in tool_sequence {
                            if let Ok(parsed_tool) = parse_yaml_tool(tool) {
                                tool_calls.push(parsed_tool);
                            }
                        }
                    } else if let Ok(parsed_tool) = parse_yaml_tool(value) {
                        tool_calls.push(parsed_tool);
                    }
                }
            }
        }
    }

    // Also check if this is a sequence containing tools
    if let Some(sequence) = yaml_value.as_sequence() {
        for item in sequence {
            find_tool_calls_in_300_series_yaml_structure(item, tool_calls);
        }
    }
}

/// Parse a single tool from YAML value
fn parse_yaml_tool(tool: &YamlValue) -> Result<ParsedToolCall, FlowDiagramError> {
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
    })
}

/// Convert YAML Value to JSON Value
fn yaml_value_to_json(yaml_value: &YamlValue) -> JsonValue {
    match yaml_value {
        YamlValue::String(s) => JsonValue::String(s.clone()),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            } else {
                JsonValue::Null
            }
        }
        YamlValue::Bool(b) => JsonValue::Bool(*b),
        YamlValue::Null => JsonValue::Null,
        YamlValue::Sequence(seq) => JsonValue::Array(seq.iter().map(yaml_value_to_json).collect()),
        YamlValue::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    json_map.insert(key_str.to_string(), yaml_value_to_json(v));
                }
            }
            JsonValue::Object(json_map)
        }
        // Handle YAML tags and other complex types as strings
        _ => JsonValue::String(format!("{yaml_value:?}")),
    }
}

/// Parse ISO 8601 timestamp to Unix timestamp (simplified)
fn parse_timestamp_to_unix(timestamp: &str) -> Option<u64> {
    // Simple parsing for common formats
    // In production, you'd use a proper datetime library
    if timestamp.contains('T') {
        // Extract the timestamp part before 'Z' if present
        let _clean_time = timestamp.trim_end_matches('Z');
        // For now, return a simplified timestamp - this is a basic implementation
        Some(1700000000) // Placeholder - would need proper datetime parsing
    } else {
        timestamp.parse::<u64>().ok()
    }
}

#[allow(dead_code)]
/// Parse enhanced_otel YAML tool call from tool_calls array
fn parse_enhanced_otel_yml_tool(tool: &JsonValue) -> Result<ParsedToolCall, FlowDiagramError> {
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

/// Extract tool calls from events array (backward compatibility)
fn extract_tools_from_events(events: &[JsonValue], tool_calls: &mut Vec<ParsedToolCall>) {
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

/// Parse a step as a tool call (for backward compatibility)
fn parse_step_as_tool_call(step: &JsonValue, _step_index: usize) -> Option<ParsedToolCall> {
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
                    });
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse direct ToolCallSummary format from enhanced dynamic flows
fn parse_direct_tool_call(
    tool: &JsonValue,
    _index: usize,
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
        "Parsed direct tool: {} ({}ms, params: {}, result: {}, args: {:?})",
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
