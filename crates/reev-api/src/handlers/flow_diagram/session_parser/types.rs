//! Session Parser Types
//!
//! This module contains the core type definitions for session parsing.

use serde::Serialize;
use serde_json::Value as JsonValue;

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
