//! Flow logging and analysis module
//!
//! This module provides a thin integration layer over the reev-flow crate
//! for backward compatibility and reev-lib specific integration.
//! Now uses unified SessionFileLogger for simplified logging.

use anyhow::Result;
use std::path::{Path, PathBuf};

// Re-export everything from reev-flow for backward compatibility
pub use reev_flow::{
    calculate_execution_statistics, get_default_flow_log_path, init_flow_tracing,
    render_flow_file_as_ascii_tree, AgentBehaviorAnalysis, ErrorContent, EventContent,
    ExecutionResult, ExecutionStatistics, FlowEdge, FlowError, FlowEvent, FlowEventType, FlowGraph,
    FlowLog, FlowLogDbExt, FlowLogRenderer, FlowLogger, FlowNode, FlowResult, FlowTracer,
    LlmRequestContent, PerformanceMetrics, ScoringBreakdown, ToolCallContent, ToolResultStatus,
    ToolUsageStats, TransactionExecutionContent, WebsiteData, WebsiteExporter,
};

// Re-export from database module for backward compatibility
#[cfg(feature = "database")]
pub use reev_flow::database::DBFlowLog;

// Re-export reev-lib specific types
pub use crate::db::AgentPerformanceData;

// Re-export SessionFileLogger for unified logging
pub use crate::session_logger::{
    convert_legacy_flow_event, load_session_log, ExecutionResult as SessionExecutionResult,
    SessionEvent, SessionEventType, SessionFileLogger, SessionLog, SessionStatistics,
};

/// Create a new flow logger with default configuration
/// For backward compatibility - delegates to SessionFileLogger
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

/// Create a new session file logger with unified logging
pub fn create_session_logger(
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    log_dir: Option<PathBuf>,
) -> Result<SessionFileLogger> {
    let log_dir = log_dir.unwrap_or_else(|| {
        std::env::var("REEV_SESSION_LOG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("logs/sessions"))
    });

    SessionFileLogger::new(session_id, benchmark_id, agent_type, &log_dir)
}

/// Get default session log path
pub fn get_default_session_log_path() -> PathBuf {
    std::env::var("REEV_SESSION_LOG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("logs/sessions"))
}

/// Quick render function for the most common use case
pub fn render_flow_as_ascii_tree(file_path: &Path) -> FlowResult<String> {
    render_flow_file_as_ascii_tree(file_path)
}
