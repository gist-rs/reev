//! Flow Diagram Generation Module
//!
//! This module provides Mermaid stateDiagram generation from agent execution logs.
//! It supports both session logs and OpenTelemetry logs for comprehensive flow visualization.

use serde::Serialize;

pub mod session_parser;
pub mod state_diagram_generator;

pub use session_parser::{ParsedSession, SessionParser};
pub use state_diagram_generator::StateDiagramGenerator;

/// Flow diagram generation result
#[derive(Debug, Clone)]
pub struct FlowDiagram {
    /// Generated Mermaid stateDiagram content
    pub diagram: String,
    /// Metadata about the diagram
    pub metadata: DiagramMetadata,
}

/// Diagram metadata
#[derive(Debug, Clone, Serialize)]
pub struct DiagramMetadata {
    /// Number of states in the diagram
    pub state_count: usize,
    /// Number of tool calls
    pub tool_count: usize,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Benchmark ID
    pub benchmark_id: String,
    /// Session ID
    pub session_id: Option<String>,
}

/// Flow diagram generation error
#[derive(Debug)]
pub enum FlowDiagramError {
    SessionNotFound(String),
    InvalidLogFormat(String),
    NoToolCalls,
    #[allow(dead_code)]
    ParseError(String),
}

impl std::fmt::Display for FlowDiagramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowDiagramError::SessionNotFound(msg) => write!(f, "Session log not found: {msg}"),
            FlowDiagramError::InvalidLogFormat(msg) => {
                write!(f, "Invalid session log format: {msg}")
            }
            FlowDiagramError::NoToolCalls => write!(f, "No tool calls found in session"),
            FlowDiagramError::ParseError(msg) => write!(f, "Parsing error: {msg}"),
        }
    }
}

impl std::error::Error for FlowDiagramError {}
