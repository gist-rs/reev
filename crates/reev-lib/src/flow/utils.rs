//! Flow utility functions
//!
//! This module contains helper functions for flow logging and statistics.

use super::types::*;
use std::collections::HashMap;

/// Calculate execution statistics from flow events
pub fn calculate_execution_statistics(events: &[FlowEvent]) -> ExecutionStatistics {
    let mut stats = ExecutionStatistics {
        total_llm_calls: 0,
        total_tool_calls: 0,
        total_tokens: 0,
        tool_usage: HashMap::new(),
        max_depth: 0,
    };

    for event in events {
        stats.max_depth = stats.max_depth.max(event.depth);

        match event.event_type {
            FlowEventType::LlmRequest => {
                stats.total_llm_calls += 1;
                if let Some(tokens) = event
                    .content
                    .data
                    .get("context_tokens")
                    .and_then(|v| v.as_u64())
                {
                    stats.total_tokens += tokens;
                }
            }
            FlowEventType::ToolCall => {
                stats.total_tool_calls += 1;
                if let Some(tool_name) =
                    event.content.data.get("tool_name").and_then(|v| v.as_str())
                {
                    *stats.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;
                }
            }
            _ => {}
        }
    }

    stats
}

/// Check if flow logging is enabled
/// Defaults to true unless explicitly set to "false" or "0"
pub fn is_flow_logging_enabled() -> bool {
    match std::env::var("REEV_ENABLE_FLOW_LOGGING") {
        Ok(val) => {
            let val_lower = val.to_lowercase();
            val_lower != "false" && val_lower != "0"
        }
        Err(_) => true, // Default to true when not set
    }
}

/// Get the default flow log output path
pub fn get_default_flow_log_path() -> std::path::PathBuf {
    std::env::var("REEV_FLOW_LOG_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("logs/flows"))
}
