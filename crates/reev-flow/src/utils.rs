//! Utility functions for FlowLog operations
//!
//! This module provides helper functions for creating, manipulating,
//! and analyzing flow logs and related structures.

use crate::types::*;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::SystemTime;
use thiserror::Error;

/// Errors that can occur during flow operations
#[derive(Debug, Error)]
pub enum FlowError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Timestamp conversion error: {0}")]
    TimestampError(String),

    #[error("Invalid flow data: {0}")]
    InvalidData(String),
}

/// Utility functions for FlowLog operations
pub struct FlowUtils;

impl FlowUtils {
    /// Create a new FlowLog with current timestamp
    pub fn create_flow_log(
        session_id: String,
        benchmark_id: String,
        agent_type: String,
    ) -> FlowLog {
        FlowLog {
            session_id,
            benchmark_id,
            agent_type,
            start_time: SystemTime::now(),
            end_time: None,
            events: Vec::new(),
            final_result: None,
        }
    }

    /// Add an event to a flow log
    pub fn add_event(flow_log: &mut FlowLog, event: FlowEvent) {
        flow_log.events.push(event);
    }

    /// Create a new flow event
    pub fn create_event(
        event_type: FlowEventType,
        depth: u32,
        data: serde_json::Value,
        metadata: HashMap<String, String>,
    ) -> FlowEvent {
        FlowEvent {
            timestamp: SystemTime::now(),
            event_type,
            depth,
            content: EventContent { data, metadata },
        }
    }

    /// Create an LLM request event
    pub fn create_llm_event(
        depth: u32,
        prompt: String,
        context_tokens: u32,
        model: String,
        request_id: String,
    ) -> FlowEvent {
        let data = serde_json::to_value(LlmRequestContent {
            prompt,
            context_tokens,
            model,
            request_id,
        })
        .unwrap();

        Self::create_event(FlowEventType::LlmRequest, depth, data, HashMap::new())
    }

    /// Create a tool call event
    pub fn create_tool_event(
        depth: u32,
        tool_name: String,
        tool_args: String,
        execution_time_ms: u32,
        result_status: ToolResultStatus,
        result_data: Option<serde_json::Value>,
        error_message: Option<String>,
    ) -> FlowEvent {
        let data = serde_json::to_value(ToolCallContent {
            tool_name,
            tool_args,
            execution_time_ms,
            result_status,
            result_data,
            error_message,
        })
        .unwrap();

        Self::create_event(FlowEventType::ToolCall, depth, data, HashMap::new())
    }

    /// Create a transaction execution event
    pub fn create_transaction_event(
        depth: u32,
        signature: String,
        instruction_count: u32,
        execution_time_ms: u32,
        success: bool,
        error: Option<String>,
    ) -> FlowEvent {
        let data = serde_json::to_value(TransactionExecutionContent {
            signature,
            instruction_count,
            execution_time_ms,
            success,
            error,
        })
        .unwrap();

        Self::create_event(
            FlowEventType::TransactionExecution,
            depth,
            data,
            HashMap::new(),
        )
    }

    /// Create an error event
    pub fn create_error_event(
        depth: u32,
        error_type: String,
        message: String,
        stack_trace: Option<String>,
        context: HashMap<String, String>,
    ) -> FlowEvent {
        let data = serde_json::to_value(ErrorContent {
            error_type,
            message,
            stack_trace,
            context,
        })
        .unwrap();

        Self::create_event(FlowEventType::Error, depth, data, HashMap::new())
    }

    /// Mark a flow log as completed with a result
    pub fn mark_completed(flow_log: &mut FlowLog, result: ExecutionResult) {
        flow_log.end_time = Some(SystemTime::now());
        flow_log.final_result = Some(result);
    }

    /// Calculate total execution duration
    pub fn calculate_duration(flow_log: &FlowLog) -> Option<std::time::Duration> {
        match flow_log.end_time {
            Some(end_time) => flow_log
                .start_time
                .duration_since(end_time)
                .or_else(|_| end_time.duration_since(flow_log.start_time))
                .ok(),
            None => None,
        }
    }

    /// Get events by type
    pub fn get_events_by_type<'a>(
        flow_log: &'a FlowLog,
        event_type: &'a FlowEventType,
    ) -> Vec<&'a FlowEvent> {
        flow_log
            .events
            .iter()
            .filter(|event| {
                std::mem::discriminant(&event.event_type) == std::mem::discriminant(event_type)
            })
            .collect()
    }

    /// Count events by type
    pub fn count_events_by_type(flow_log: &FlowLog) -> HashMap<String, u32> {
        let mut counts = HashMap::new();

        for event in &flow_log.events {
            let type_name = match &event.event_type {
                FlowEventType::LlmRequest => "LlmRequest",
                FlowEventType::ToolCall => "ToolCall",
                FlowEventType::ToolResult => "ToolResult",
                FlowEventType::TransactionExecution => "TransactionExecution",
                FlowEventType::Error => "Error",
                FlowEventType::BenchmarkStateChange => "BenchmarkStateChange",
            };

            *counts.entry(type_name.to_string()).or_insert(0) += 1;
        }

        counts
    }

    /// Get maximum depth reached
    pub fn get_max_depth(flow_log: &FlowLog) -> u32 {
        flow_log
            .events
            .iter()
            .map(|event| event.depth)
            .max()
            .unwrap_or(0)
    }

    /// Get average execution time for tools
    pub fn get_average_tool_execution_time(flow_log: &FlowLog) -> Option<f64> {
        let tool_events: Vec<&FlowEvent> = flow_log
            .events
            .iter()
            .filter(|event| matches!(event.event_type, FlowEventType::ToolCall))
            .collect();

        if tool_events.is_empty() {
            return None;
        }

        let total_time: u32 = tool_events
            .iter()
            .filter_map(|event| {
                if let Ok(content) =
                    serde_json::from_value::<ToolCallContent>(event.content.data.clone())
                {
                    Some(content.execution_time_ms)
                } else {
                    None
                }
            })
            .sum();

        Some(total_time as f64 / tool_events.len() as f64)
    }

    /// Convert SystemTime to RFC3339 string
    pub fn system_time_to_rfc3339(time: SystemTime) -> Result<String, FlowError> {
        let duration = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| FlowError::TimestampError(e.to_string()))?;

        let datetime = DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
            .ok_or_else(|| FlowError::TimestampError("Invalid timestamp".to_string()))?;

        Ok(datetime.to_rfc3339())
    }

    /// Parse RFC3339 string to SystemTime
    pub fn rfc3339_to_system_time(rfc3339: &str) -> Result<SystemTime, FlowError> {
        let datetime = DateTime::parse_from_rfc3339(rfc3339)
            .map_err(|e| FlowError::TimestampError(e.to_string()))?;

        let system_time = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(datetime.timestamp() as u64)
            + std::time::Duration::from_nanos(datetime.timestamp_subsec_nanos() as u64);

        Ok(system_time)
    }

    /// Generate a summary of the flow execution
    pub fn generate_summary(flow_log: &FlowLog) -> FlowSummary {
        let event_counts = Self::count_events_by_type(flow_log);
        let max_depth = Self::get_max_depth(flow_log);
        let duration = Self::calculate_duration(flow_log);
        let avg_tool_time = Self::get_average_tool_execution_time(flow_log);

        FlowSummary {
            session_id: flow_log.session_id.clone(),
            benchmark_id: flow_log.benchmark_id.clone(),
            agent_type: flow_log.agent_type.clone(),
            total_events: flow_log.events.len(),
            event_counts,
            max_depth,
            duration_ms: duration.map(|d| d.as_millis() as u64),
            average_tool_execution_time_ms: avg_tool_time.map(|t| t as u64),
            success: flow_log
                .final_result
                .as_ref()
                .map(|r| r.success)
                .unwrap_or(false),
            final_score: flow_log.final_result.as_ref().map(|r| r.score),
        }
    }
}

/// Summary of flow execution for quick analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub total_events: usize,
    pub event_counts: HashMap<String, u32>,
    pub max_depth: u32,
    pub duration_ms: Option<u64>,
    pub average_tool_execution_time_ms: Option<u64>,
    pub success: bool,
    pub final_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_log_creation() {
        let flow_log = FlowUtils::create_flow_log(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        assert_eq!(flow_log.session_id, "session-123");
        assert_eq!(flow_log.benchmark_id, "benchmark-456");
        assert_eq!(flow_log.agent_type, "llm");
        assert!(flow_log.end_time.is_none());
        assert!(flow_log.final_result.is_none());
        assert!(flow_log.events.is_empty());
    }

    #[test]
    fn test_add_event() {
        let mut flow_log = FlowUtils::create_flow_log(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        let event = FlowUtils::create_event(
            FlowEventType::LlmRequest,
            1,
            serde_json::json!({"test": "data"}),
            HashMap::new(),
        );

        FlowUtils::add_event(&mut flow_log, event);
        assert_eq!(flow_log.events.len(), 1);
    }

    #[test]
    fn test_llm_event_creation() {
        let event = FlowUtils::create_llm_event(
            1,
            "test prompt".to_string(),
            100,
            "gpt-4".to_string(),
            "req-123".to_string(),
        );

        assert!(matches!(event.event_type, FlowEventType::LlmRequest));
        assert_eq!(event.depth, 1);
    }

    #[test]
    fn test_tool_event_creation() {
        let event = FlowUtils::create_tool_event(
            2,
            "test_tool".to_string(),
            "{}".to_string(),
            100,
            ToolResultStatus::Success,
            Some(serde_json::json!({"result": "ok"})),
            None,
        );

        assert!(matches!(event.event_type, FlowEventType::ToolCall));
        assert_eq!(event.depth, 2);
    }

    #[test]
    fn test_event_counting() {
        let mut flow_log = FlowUtils::create_flow_log(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        let llm_event = FlowUtils::create_llm_event(
            1,
            "test prompt".to_string(),
            100,
            "gpt-4".to_string(),
            "req-123".to_string(),
        );

        let tool_event = FlowUtils::create_tool_event(
            2,
            "test_tool".to_string(),
            "{}".to_string(),
            100,
            ToolResultStatus::Success,
            None,
            None,
        );

        FlowUtils::add_event(&mut flow_log, llm_event);
        FlowUtils::add_event(&mut flow_log, tool_event);

        let counts = FlowUtils::count_events_by_type(&flow_log);
        assert_eq!(counts.get("LlmRequest"), Some(&1));
        assert_eq!(counts.get("ToolCall"), Some(&1));
    }

    #[test]
    fn test_flow_summary() {
        let mut flow_log = FlowUtils::create_flow_log(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        let event = FlowUtils::create_llm_event(
            1,
            "test prompt".to_string(),
            100,
            "gpt-4".to_string(),
            "req-123".to_string(),
        );

        FlowUtils::add_event(&mut flow_log, event);

        let result = ExecutionResult {
            success: true,
            score: 0.85,
            total_time_ms: 1000,
            statistics: ExecutionStatistics::default(),
            scoring_breakdown: None,
        };

        FlowUtils::mark_completed(&mut flow_log, result);

        let summary = FlowUtils::generate_summary(&flow_log);
        assert_eq!(summary.session_id, "session-123");
        assert_eq!(summary.total_events, 1);
        assert!(summary.success);
        assert_eq!(summary.final_score, Some(0.85));
    }
}
