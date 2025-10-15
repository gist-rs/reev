//! Flow logging and analysis module
//!
//! This module provides comprehensive flow logging for benchmark execution,
//! including detailed event tracking, scoring analysis, and ASCII tree rendering.

use std::path::PathBuf;
use tracing::info;

pub mod error;
pub mod logger;
pub mod otel;
pub mod renderer;
pub mod types;
pub mod utils;
pub mod website_exporter;

// Re-export commonly used types for convenience
pub use types::{
    AgentBehaviorAnalysis, ErrorContent, EventContent, ExecutionResult, ExecutionStatistics,
    FlowEdge, FlowEvent, FlowEventType, FlowGraph, FlowLog, FlowNode, LlmRequestContent,
    PerformanceMetrics, ScoringBreakdown, ToolCallContent, ToolResultStatus, ToolUsageStats,
    TransactionExecutionContent, WebsiteData,
};

pub use error::{FlowError, FlowResult};
pub use logger::{AgentPerformanceData, FlowLogger};
pub use renderer::render_flow_file_as_ascii_tree;
pub use website_exporter::WebsiteExporter;

use std::path::Path;

/// Initialize flow tracing if enabled
pub fn init_flow_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let enabled = std::env::var("REEV_OTEL_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if !enabled {
        info!("Flow tracing initialization skipped");
        return Ok(());
    }

    info!("Initializing flow tracing...");

    let service_name =
        std::env::var("REEV_OTEL_SERVICE_NAME").unwrap_or_else(|_| "reev".to_string());

    info!("Flow tracing service name: {}", service_name);
    info!(
        "Note: This is simplified flow tracing. Full OpenTelemetry integration can be added later."
    );

    info!("Flow tracing initialization completed");
    Ok(())
}

/// Quick render function for the most common use case
pub fn render_flow_as_ascii_tree(file_path: &Path) -> FlowResult<String> {
    renderer::render_flow_file_as_ascii_tree(file_path)
}

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

// Re-export utility functions
pub use utils::{
    calculate_execution_statistics, get_default_flow_log_path, is_flow_logging_enabled,
};
