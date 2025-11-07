//! Extract Transfer Details Module
//!
//! This module provides utility functions for extracting transfer details from parsed tool calls.

use crate::handlers::flow_diagram::session_parser::ParsedToolCall;

use super::extract_tool_details;

/// Extract transfer details (from, to, amount) from a parsed tool call
pub fn extract_transfer_details(tool_call: &ParsedToolCall) -> Option<(String, String, String)> {
    extract_tool_details(tool_call)
}
