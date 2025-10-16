//! Flow logging and analysis module
//!
//! This module provides a thin integration layer over the reev-flow crate
//! for backward compatibility and reev-lib specific integration.

use std::path::{Path, PathBuf};

// Re-export everything from reev-flow
pub use reev_flow::{
    calculate_execution_statistics, get_default_flow_log_path, init_flow_tracing,
    is_flow_logging_enabled, render_flow_file_as_ascii_tree, AgentBehaviorAnalysis, ErrorContent,
    EventContent, ExecutionResult, ExecutionStatistics, FlowEdge, FlowError, FlowEvent,
    FlowEventType, FlowGraph, FlowLog, FlowLogDbExt, FlowLogRenderer, FlowLogger, FlowNode,
    FlowResult, FlowTracer, LlmRequestContent, PerformanceMetrics, ScoringBreakdown,
    ToolCallContent, ToolResultStatus, ToolUsageStats, TransactionExecutionContent, WebsiteData,
    WebsiteExporter,
};

// Re-export from database module for backward compatibility
#[cfg(feature = "database")]
pub use reev_flow::database::DBFlowLog;

// Re-export reev-lib specific types
pub use crate::db::AgentPerformanceData;

/// Create a new flow logger with default configuration
pub fn create_flow_logger(
    benchmark_id: String,
    agent_type: String,
    output_path: Option<std::path::PathBuf>,
) -> FlowLogger {
    let output_path = output_path.unwrap_or_else(|| {
        std::env::var("REEV_FLOW_LOG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("logs/flows"))
    });

    FlowLogger::new(benchmark_id, agent_type, output_path)
}

/// Quick render function for the most common use case
pub fn render_flow_as_ascii_tree(file_path: &Path) -> FlowResult<String> {
    render_flow_file_as_ascii_tree(file_path)
}
