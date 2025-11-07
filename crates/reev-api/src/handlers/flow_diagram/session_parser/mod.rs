//! Session Parser Modules
//!
//! This module contains all the sub-modules for parsing session logs and extracting tool calls.
//! The original SessionParser implementation has been split into focused modules for better maintainability.

// Import all sub-modules
pub mod create_metadata;
pub mod extract_tools_from_events;
pub mod extract_user_prompt_from_yaml;
pub mod find_tool_calls_in_300_series_yaml_structure;
pub mod parse_direct_tool_call;
// pub mod parse_enhanced_otel_yml_tool; // Currently unused
pub mod parse_session_content;
pub mod parse_step_as_tool_call;
pub mod parse_timestamp_to_unix;
pub mod parse_yaml_session_content;
pub mod parse_yaml_tool;
pub mod types;
pub mod yaml_value_to_json;

// Re-export main types and public functions
pub use create_metadata::create_metadata;
pub use parse_session_content::parse_session_content;
pub use types::{ParsedSession, ParsedToolCall};

// Re-export internal utility functions for use within the module
pub use extract_tools_from_events::extract_tools_from_events;
pub use extract_user_prompt_from_yaml::extract_user_prompt_from_yaml;
pub use find_tool_calls_in_300_series_yaml_structure::find_tool_calls_in_300_series_yaml_structure;
pub use parse_direct_tool_call::parse_direct_tool_call;
// pub use parse_enhanced_otel_yml_tool::parse_enhanced_otel_yml_tool; // Currently unused
pub use parse_step_as_tool_call::parse_step_as_tool_call;
pub use parse_timestamp_to_unix::parse_timestamp_to_unix;
pub use parse_yaml_session_content::parse_yaml_session_content;
pub use parse_yaml_tool::parse_yaml_tool;
pub use yaml_value_to_json::yaml_value_to_json;

/// Session log parser for extracting tool calls and execution data
pub struct SessionParser;

impl SessionParser {
    /// Create diagram metadata from parsed session
    pub fn create_metadata(
        parsed: &ParsedSession,
    ) -> crate::handlers::flow_diagram::DiagramMetadata {
        create_metadata(parsed)
    }

    /// Parse session log content and extract tool calls
    pub fn parse_session_content(
        content: &str,
    ) -> Result<ParsedSession, crate::handlers::flow_diagram::FlowDiagramError> {
        parse_session_content(content)
    }
}
