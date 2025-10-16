//! Shared flow types for reev ecosystem
//!
//! This module contains core flow logging types that can be used across
//! different projects. These types are designed to be:
//! 1. Database-friendly (use String timestamps, JSON serializable)
//! 2. Generic enough for different use cases
//! 3. Easily convertible to/from domain-specific types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core flow log structure for database storage
///
/// This is the canonical FlowLog type that should be used for:
/// - Database operations
/// - API responses
/// - Cross-project communication
///
/// For domain-specific operations, create conversion traits to/from this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBFlowLog {
    /// Unique session identifier for grouping related logs
    pub session_id: String,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type (deterministic, local, gemini, etc.)
    pub agent_type: String,
    /// Start timestamp (RFC3339 format)
    pub start_time: String,
    /// End timestamp (RFC3339 format, optional)
    pub end_time: Option<String>,
    /// All events serialized as JSON
    pub flow_data: String,
    /// Final result serialized as JSON (optional)
    pub final_result: Option<String>,
    /// Database-specific fields
    pub id: Option<i64>,
    /// Creation timestamp
    pub created_at: Option<String>,
}

/// Individual flow event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEvent {
    /// Event timestamp (RFC3339 format)
    pub timestamp: String,
    /// Type of event
    pub event_type: FlowEventType,
    /// Conversation depth when event occurred
    pub depth: u32,
    /// Event-specific content
    pub content: EventContent,
}

/// Types of events that can occur during flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FlowEventType {
    /// LLM request/response cycle
    LlmRequest,
    /// Tool invocation
    ToolCall,
    /// Tool result/response
    ToolResult,
    /// Transaction execution
    TransactionExecution,
    /// Error occurred
    Error,
    /// Benchmark state change
    BenchmarkStateChange,
}

/// Event content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContent {
    /// Event-specific data
    pub data: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Final execution result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Overall success status
    pub success: bool,
    /// Final score
    pub score: f64,
    /// Total execution time
    pub total_time_ms: u64,
    /// Summary statistics
    pub statistics: ExecutionStatistics,
    /// Detailed scoring breakdown
    pub scoring_breakdown: Option<ScoringBreakdown>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionStatistics {
    /// Total LLM calls
    pub total_llm_calls: u32,
    /// Total tool calls
    pub total_tool_calls: u32,
    /// Total tokens used
    pub total_tokens: u64,
    /// Tool usage breakdown
    pub tool_usage: HashMap<String, u32>,
    /// Conversation depth reached
    pub max_depth: u32,
}

/// Detailed scoring breakdown for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringBreakdown {
    /// Instruction matching score (0-1)
    pub instruction_score: f64,
    /// On-chain execution score (0-1)
    pub onchain_score: f64,
    /// Weighted final score (0-1)
    pub final_score: f64,
    /// Issues that affected the score
    pub issues: Vec<String>,
    /// Specific mismatches found
    pub mismatches: Vec<String>,
}

/// Conversion trait for domain-specific FlowLog types
///
/// Implement this trait for your domain-specific FlowLog types
/// to convert to/from the shared FlowLog type.
pub trait FlowLogConverter<T> {
    /// Convert from domain type to shared FlowLog
    fn to_flow_log(&self) -> Result<DBFlowLog, ConversionError>;

    /// Convert from shared FlowLog to domain type
    fn from_flow_log(flow_log: &DBFlowLog) -> Result<T, ConversionError>;
}

/// Conversion error types
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

/// Utility functions for FlowLog operations
pub struct FlowLogUtils;

impl FlowLogUtils {
    /// Create a new FlowLog with current timestamp
    pub fn create(session_id: String, benchmark_id: String, agent_type: String) -> DBFlowLog {
        let now = chrono::Utc::now().to_rfc3339();
        DBFlowLog {
            session_id,
            benchmark_id,
            agent_type,
            start_time: now.clone(),
            end_time: None,
            flow_data: serde_json::to_string(&Vec::<FlowEvent>::new()).unwrap(),
            final_result: None,
            id: None,
            created_at: Some(now),
        }
    }

    /// Convert SystemTime to RFC3339 string
    pub fn system_time_to_rfc3339(time: std::time::SystemTime) -> Result<String, ConversionError> {
        let duration = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ConversionError::TimestampError(e.to_string()))?;
        let datetime =
            chrono::DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
                .ok_or_else(|| ConversionError::TimestampError("Invalid timestamp".to_string()))?;
        Ok(datetime.to_rfc3339())
    }

    /// Parse RFC3339 string to SystemTime
    pub fn rfc3339_to_system_time(rfc3339: &str) -> Result<std::time::SystemTime, ConversionError> {
        let datetime = chrono::DateTime::parse_from_rfc3339(rfc3339)?;
        let system_time = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(datetime.timestamp() as u64)
            + std::time::Duration::from_nanos(datetime.timestamp_subsec_nanos() as u64);
        Ok(system_time)
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
        match &flow_log.end_time {
            Some(end_time) => {
                let start = chrono::DateTime::parse_from_rfc3339(&flow_log.start_time)?;
                let end = chrono::DateTime::parse_from_rfc3339(end_time)?;
                let duration = end.signed_duration_since(start);
                Ok(Some(std::time::Duration::from_secs(
                    duration.num_seconds() as u64
                )))
            }
            None => Ok(None),
        }
    }

    /// Mark FlowLog as completed
    pub fn mark_completed(
        flow_log: &mut DBFlowLog,
        result: Option<ExecutionResult>,
    ) -> Result<(), ConversionError> {
        flow_log.end_time = Some(chrono::Utc::now().to_rfc3339());
        if let Some(result) = result {
            flow_log.final_result = Some(Self::serialize_result(&result)?);
        }
        Ok(())
    }

    /// Add event to flow log
    pub fn add_event(flow_log: &mut DBFlowLog, event: FlowEvent) -> Result<(), ConversionError> {
        let mut events = Self::deserialize_events(&flow_log.flow_data)?;
        events.push(event);
        flow_log.flow_data = Self::serialize_events(&events)?;
        Ok(())
    }

    /// Get all events from flow log
    pub fn get_events(flow_log: &DBFlowLog) -> Result<Vec<FlowEvent>, ConversionError> {
        Self::deserialize_events(&flow_log.flow_data)
    }

    /// Get execution result from flow log
    pub fn get_result(flow_log: &DBFlowLog) -> Result<Option<ExecutionResult>, ConversionError> {
        match &flow_log.final_result {
            Some(json) => Ok(Some(Self::deserialize_result(json)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_log_creation() {
        let flow_log = FlowLogUtils::create(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        assert_eq!(flow_log.session_id, "session-123");
        assert_eq!(flow_log.benchmark_id, "benchmark-456");
        assert_eq!(flow_log.agent_type, "llm");
        assert!(flow_log.end_time.is_none());
        assert!(flow_log.final_result.is_none());
    }

    #[test]
    fn test_event_serialization() {
        let event = FlowEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: FlowEventType::LlmRequest,
            depth: 1,
            content: EventContent {
                data: serde_json::json!({"test": "data"}),
                metadata: HashMap::new(),
            },
        };

        let events = vec![event];
        let json = FlowLogUtils::serialize_events(&events).unwrap();
        let deserialized = FlowLogUtils::deserialize_events(&json).unwrap();

        assert_eq!(events.len(), deserialized.len());
    }

    #[test]
    fn test_system_time_conversion() {
        let now = std::time::SystemTime::now();
        let rfc3339 = FlowLogUtils::system_time_to_rfc3339(now).unwrap();
        let converted_back = FlowLogUtils::rfc3339_to_system_time(&rfc3339).unwrap();

        // Allow small difference due to precision
        let diff = now
            .duration_since(converted_back)
            .unwrap_or_else(|_| converted_back.duration_since(now).unwrap());
        assert!(diff.as_millis() < 1000); // Less than 1 second difference
    }
}
