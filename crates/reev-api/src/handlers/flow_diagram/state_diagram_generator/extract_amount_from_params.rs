//! Extract Amount From Params Module
//!
//! This module provides utility functions for extracting amounts from parameters for display in diagrams.

use crate::handlers::flow_diagram::session_parser::ParsedToolCall;
use reev_types::ToolName;

/// Extract amount from parameters for display
pub fn extract_amount_from_params(tool_call: &ParsedToolCall) -> Option<String> {
    // For transfer operations, return instruction count from result_data
    if let Some(result_data) = &tool_call.result_data {
        if let Some(instruction_count) = result_data.get("instruction_count") {
            if let Some(count) = instruction_count.as_u64() {
                // Proper pluralization: 1 ix, 2 ixs, 3 ixs, etc.
                let suffix = if count == 1 { "ix" } else { "ixs" };
                return Some(format!("{count} {suffix}"));
            }
        }
    }
    // Fallback: determine based on tool type and avoid hardcoding
    if tool_call
        .tool_name
        .contains(ToolName::SolTransfer.to_string().as_str())
        || tool_call
            .tool_name
            .contains(ToolName::SplTransfer.to_string().as_str())
    {
        Some("1 ix".to_string()) // Default to singular for transfers
    } else {
        Some("operation".to_string()) // Generic fallback for other tools
    }
}
