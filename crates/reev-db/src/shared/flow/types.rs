//! Shared flow types for reev ecosystem
//!
//! This module re-exports flow types from the reev-flow crate
//! to maintain backward compatibility while centralizing type definitions.

use serde::{Deserialize, Serialize};

// Re-export core types from reev-flow
pub use reev_flow::{
    AgentBehaviorAnalysis, ErrorContent, EventContent, ExecutionResult, ExecutionStatistics,
    FlowEdge, FlowError, FlowEvent, FlowEventType, FlowGraph, FlowLog, FlowNode, FlowSummary,
    FlowUtils, LlmRequestContent, PerformanceMetrics, ScoringBreakdown, ToolCallContent,
    ToolUsageStats, TransactionExecutionContent, WebsiteData,
};

// Re-export ToolResultStatus from reev-types
pub use reev_types::ToolResultStatus;

// Re-export database types when feature is enabled
pub use reev_flow::database::{
    DBFlowLog, DBFlowLogConverter, DBStorageFormat, FlowLogDB, FlowLogQuery,
};

// Re-export conversion traits and utilities for backward compatibility
pub use reev_flow::database::DBFlowLogConverter as FlowLogConverter;

/// Legacy conversion error type for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionError {
    /// JSON serialization/deserialization error
    JsonError(String),
    /// Timestamp parsing error
    TimestampError(String),
    /// Missing required field
    MissingField(String),
    /// Invalid data format
    InvalidFormat(String),
    /// Other conversion error
    Other(String),
}

impl From<serde_json::Error> for ConversionError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}

impl From<chrono::ParseError> for ConversionError {
    fn from(err: chrono::ParseError) -> Self {
        Self::TimestampError(err.to_string())
    }
}

impl From<FlowError> for ConversionError {
    fn from(err: FlowError) -> Self {
        match err {
            FlowError::JsonError(e) => Self::JsonError(e.to_string()),
            FlowError::TimestampError(e) => Self::TimestampError(e.to_string()),
            FlowError::InvalidData(e) => Self::InvalidFormat(e),
            FlowError::SerializationError(e) => Self::JsonError(e),
            FlowError::FileError(e) => Self::JsonError(e),
            FlowError::ConfigError(e) => Self::JsonError(e),
            FlowError::IoError(e) => Self::JsonError(e.to_string()),
            FlowError::YamlError(e) => Self::JsonError(e.to_string()),
            FlowError::DatabaseError(e) => Self::JsonError(e),
        }
    }
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonError(msg) => write!(f, "JSON error: {msg}"),
            Self::TimestampError(msg) => write!(f, "Timestamp error: {msg}"),
            Self::MissingField(field) => write!(f, "Missing required field: {field}"),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            Self::Other(msg) => write!(f, "Conversion error: {msg}"),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Legacy utility functions for backward compatibility
pub struct FlowLogUtils;

impl FlowLogUtils {
    /// Create a new FlowLog with current timestamp
    pub fn create(session_id: String, benchmark_id: String, agent_type: String) -> DBFlowLog {
        reev_flow::database::FlowLogDB::create(session_id, benchmark_id, agent_type)
    }

    /// Convert SystemTime to RFC3339 string
    pub fn system_time_to_rfc3339(time: std::time::SystemTime) -> Result<String, ConversionError> {
        FlowUtils::system_time_to_rfc3339(time).map_err(Into::into)
    }

    /// Parse RFC3339 string to SystemTime
    pub fn rfc3339_to_system_time(rfc3339: &str) -> Result<std::time::SystemTime, ConversionError> {
        FlowUtils::rfc3339_to_system_time(rfc3339).map_err(Into::into)
    }

    /// Serialize events to JSON string
    pub fn serialize_events(events: &[FlowEvent]) -> Result<String, ConversionError> {
        serde_json::to_string(events).map_err(Into::into)
    }

    /// Deserialize events from JSON string
    pub fn deserialize_events(json: &str) -> Result<Vec<FlowEvent>, ConversionError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serialize execution result to JSON string
    pub fn serialize_result(result: &ExecutionResult) -> Result<String, ConversionError> {
        serde_json::to_string(result).map_err(Into::into)
    }

    /// Deserialize execution result from JSON string
    pub fn deserialize_result(json: &str) -> Result<ExecutionResult, ConversionError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Calculate execution duration
    pub fn calculate_duration(
        flow_log: &DBFlowLog,
    ) -> Result<Option<std::time::Duration>, ConversionError> {
        Ok(flow_log.duration_ms().map(std::time::Duration::from_millis))
    }

    /// Mark FlowLog as completed
    pub fn mark_completed(
        flow_log: &mut DBFlowLog,
        result: Option<ExecutionResult>,
    ) -> Result<(), ConversionError> {
        if let Some(result) = result {
            reev_flow::database::FlowLogDB::mark_completed(flow_log, result)?;
        }
        Ok(())
    }

    /// Add event to flow log
    pub fn add_event(flow_log: &mut DBFlowLog, event: FlowEvent) -> Result<(), ConversionError> {
        reev_flow::database::FlowLogDB::add_event(
            flow_log,
            event.event_type,
            event.depth,
            event.content.data,
        )?;
        Ok(())
    }

    /// Get all events from flow log
    pub fn get_events(flow_log: &DBFlowLog) -> Result<Vec<FlowEvent>, ConversionError> {
        Ok(flow_log.flow.events.clone())
    }

    /// Get execution result from flow log
    pub fn get_result(flow_log: &DBFlowLog) -> Result<Option<ExecutionResult>, ConversionError> {
        Ok(flow_log.flow.final_result.clone())
    }
}
