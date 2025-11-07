//! Parse Session Content Module
//!
//! This module provides the main function for parsing session log content.

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use tracing::{debug, info, warn};

use super::{
    extract_tools_from_events, parse_direct_tool_call, parse_step_as_tool_call,
    parse_yaml_session_content, parse_yaml_tool, types::ParsedSession,
};
use crate::handlers::flow_diagram::FlowDiagramError;

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
    else if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str()) {
        debug!("🔍 Parsing log_content (length: {})", log_content.len());

        // Try YAML parsing first (for OTEL-derived data)
        match serde_yaml::from_str::<YamlValue>(log_content) {
            Ok(yaml_value) => {
                debug!("✅ YAML parsing successful");
                if let Some(yml_tools) = yaml_value.get("tool_calls").and_then(|t| t.as_sequence())
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
    } else if let Some(log_content) = session_log.get("log_content").and_then(|lc| lc.as_str()) {
        // Try to parse log_content as JSON to extract prompt
        if let Ok(log_json) = serde_json::from_str::<JsonValue>(log_content) {
            log_json
                .get("prompt")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        } else {
            // Try to extract user_prompt from YAML-like format
            super::extract_user_prompt_from_yaml(log_content)
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
