//! Find Tool Calls in 300 Series YAML Structure Module
//!
//! This module provides function for finding tool calls in 300-series YAML structure.

use serde_yaml::Value as YamlValue;
use tracing::debug;

use super::{parse_yaml_tool, types::ParsedToolCall};

/// Find tool calls in 300-series YAML structure
pub fn find_tool_calls_in_300_series_yaml_structure(
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
