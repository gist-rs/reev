//! Parse YAML Session Content Module
//!
//! This module provides function for parsing YAML session content from OTEL-derived data.

use serde_yaml::Value as YamlValue;
use tracing::{debug, info};

use super::{find_tool_calls_in_300_series_yaml_structure, parse_yaml_tool, types::ParsedSession};
use crate::handlers::flow_diagram::FlowDiagramError;

/// Parse YAML session content from OTEL-derived data
pub fn parse_yaml_session_content(
    yaml_value: &YamlValue,
) -> Result<ParsedSession, FlowDiagramError> {
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
