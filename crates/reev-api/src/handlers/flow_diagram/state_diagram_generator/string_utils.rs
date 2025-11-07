//! String Utilities Module
//!
//! This module provides utility functions for cleaning and sanitizing strings for display in diagrams.

/// Sanitize tool name for Mermaid diagram compatibility
pub fn sanitize_tool_name(tool_name: &str) -> String {
    tool_name
        .replace("-", "")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
}
