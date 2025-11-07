//! Create Metadata Module
//!
//! This module provides the function for creating diagram metadata from parsed sessions.

use super::types::ParsedSession;
use crate::handlers::flow_diagram::DiagramMetadata;

/// Create diagram metadata from parsed session
pub fn create_metadata(parsed: &ParsedSession) -> DiagramMetadata {
    let execution_time_ms = if let Some(end) = parsed.end_time {
        end.saturating_sub(parsed.start_time) * 1000
    } else {
        parsed.tool_calls.iter().map(|t| t.duration_ms).sum()
    };

    DiagramMetadata {
        state_count: 2 + parsed.tool_calls.len(), // Start + End + Tools
        tool_count: parsed.tool_calls.len(),
        execution_time_ms,
        benchmark_id: parsed.benchmark_id.clone(),
        session_id: Some(parsed.session_id.clone()),
    }
}
