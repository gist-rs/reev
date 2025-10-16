//! Conversion module for reev-lib flow types
//!
//! This module provides conversions between reev-lib domain-specific types
//! and the shared types from reev-db. This allows reev-lib to maintain
//! its domain model while using the generic database operations.

use crate::flow::types::*;
use reev_db::shared::flow::ConversionError;
use reev_db::shared::flow::{DBFlowLog, FlowLogConverter, FlowLogUtils};

/// Converter for reev-lib FlowLog to shared FlowLog
impl FlowLogConverter<crate::flow::types::FlowLog> for crate::flow::types::FlowLog {
    fn to_flow_log(&self) -> Result<DBFlowLog, ConversionError> {
        // Convert SystemTime to RFC3339 string
        let start_time_str = FlowLogUtils::system_time_to_rfc3339(self.start_time)?;
        let end_time_str = self
            .end_time
            .map(FlowLogUtils::system_time_to_rfc3339)
            .transpose()?;

        // Convert events to shared events
        let shared_events: Result<Vec<_>, _> =
            self.events.iter().map(convert_reev_lib_event).collect();
        let events_json = FlowLogUtils::serialize_events(&shared_events?)?;

        // Convert final result if present
        let final_result_json = self
            .final_result
            .as_ref()
            .map(convert_reev_lib_result)
            .transpose()?;

        Ok(DBFlowLog {
            session_id: self.session_id.clone(),
            benchmark_id: self.benchmark_id.clone(),
            agent_type: self.agent_type.clone(),
            start_time: start_time_str.clone(),
            end_time: end_time_str,
            flow_data: events_json,
            final_result: final_result_json,
            id: None,
            created_at: Some(start_time_str),
        })
    }

    fn from_flow_log(flow_log: &DBFlowLog) -> Result<crate::flow::types::FlowLog, ConversionError> {
        let start_time = FlowLogUtils::rfc3339_to_system_time(&flow_log.start_time)?;
        let end_time = flow_log
            .end_time
            .as_ref()
            .map(|et| FlowLogUtils::rfc3339_to_system_time(et))
            .transpose()?;

        let events = FlowLogUtils::deserialize_events(&flow_log.flow_data)?;
        let reev_lib_events: Result<Vec<_>, _> =
            events.iter().map(convert_to_reev_lib_event).collect();

        let final_result = flow_log
            .final_result
            .as_ref()
            .map(|fr| convert_to_reev_lib_result(fr))
            .transpose()?;

        Ok(crate::flow::types::FlowLog {
            session_id: flow_log.session_id.clone(),
            benchmark_id: flow_log.benchmark_id.clone(),
            agent_type: flow_log.agent_type.clone(),
            start_time,
            end_time,
            events: reev_lib_events?,
            final_result,
        })
    }
}

/// Convert reev-lib FlowEvent to shared FlowEvent
fn convert_reev_lib_event(
    event: &FlowEvent,
) -> Result<reev_db::shared::flow::FlowEvent, ConversionError> {
    let timestamp_str = FlowLogUtils::system_time_to_rfc3339(event.timestamp)?;
    let shared_event_type = match event.event_type {
        FlowEventType::LlmRequest => reev_db::shared::flow::FlowEventType::LlmRequest,
        FlowEventType::ToolCall => reev_db::shared::flow::FlowEventType::ToolCall,
        FlowEventType::ToolResult => reev_db::shared::flow::FlowEventType::ToolResult,
        FlowEventType::TransactionExecution => {
            reev_db::shared::flow::FlowEventType::TransactionExecution
        }
        FlowEventType::Error => reev_db::shared::flow::FlowEventType::Error,
        FlowEventType::BenchmarkStateChange => {
            reev_db::shared::flow::FlowEventType::BenchmarkStateChange
        }
    };

    Ok(reev_db::shared::flow::FlowEvent {
        timestamp: timestamp_str,
        event_type: shared_event_type,
        depth: event.depth,
        content: reev_db::shared::flow::EventContent {
            data: event.content.data.clone(),
            metadata: event.content.metadata.clone(),
        },
    })
}

/// Convert shared FlowEvent to reev-lib FlowEvent
fn convert_to_reev_lib_event(
    event: &reev_db::shared::flow::FlowEvent,
) -> Result<FlowEvent, ConversionError> {
    let timestamp = FlowLogUtils::rfc3339_to_system_time(&event.timestamp)?;
    let reev_lib_event_type = match event.event_type {
        reev_db::shared::flow::FlowEventType::LlmRequest => FlowEventType::LlmRequest,
        reev_db::shared::flow::FlowEventType::ToolCall => FlowEventType::ToolCall,
        reev_db::shared::flow::FlowEventType::ToolResult => FlowEventType::ToolResult,
        reev_db::shared::flow::FlowEventType::TransactionExecution => {
            FlowEventType::TransactionExecution
        }
        reev_db::shared::flow::FlowEventType::Error => FlowEventType::Error,
        reev_db::shared::flow::FlowEventType::BenchmarkStateChange => {
            FlowEventType::BenchmarkStateChange
        }
    };

    Ok(FlowEvent {
        timestamp,
        event_type: reev_lib_event_type,
        depth: event.depth,
        content: EventContent {
            data: event.content.data.clone(),
            metadata: event.content.metadata.clone(),
        },
    })
}

/// Convert reev-lib ExecutionResult to shared ExecutionResult
fn convert_reev_lib_result(result: &ExecutionResult) -> Result<String, ConversionError> {
    let shared_result = reev_db::shared::flow::ExecutionResult {
        success: result.success,
        score: result.score,
        total_time_ms: result.total_time_ms,
        statistics: reev_db::shared::flow::ExecutionStatistics {
            total_llm_calls: result.statistics.total_llm_calls,
            total_tool_calls: result.statistics.total_tool_calls,
            total_tokens: result.statistics.total_tokens,
            tool_usage: result.statistics.tool_usage.clone(),
            max_depth: result.statistics.max_depth,
        },
        scoring_breakdown: result.scoring_breakdown.as_ref().map(|sb| {
            reev_db::shared::flow::ScoringBreakdown {
                instruction_score: sb.instruction_score,
                onchain_score: sb.onchain_score,
                final_score: sb.final_score,
                issues: sb.issues.clone(),
                mismatches: sb.mismatches.clone(),
            }
        }),
    };
    FlowLogUtils::serialize_result(&shared_result)
}

/// Convert shared ExecutionResult to reev-lib ExecutionResult
fn convert_to_reev_lib_result(json: &str) -> Result<ExecutionResult, ConversionError> {
    let shared_result = FlowLogUtils::deserialize_result(json)?;
    Ok(ExecutionResult {
        success: shared_result.success,
        score: shared_result.score,
        total_time_ms: shared_result.total_time_ms,
        statistics: ExecutionStatistics {
            total_llm_calls: shared_result.statistics.total_llm_calls,
            total_tool_calls: shared_result.statistics.total_tool_calls,
            total_tokens: shared_result.statistics.total_tokens,
            tool_usage: shared_result.statistics.tool_usage,
            max_depth: shared_result.statistics.max_depth,
        },
        scoring_breakdown: shared_result.scoring_breakdown.map(|sb| ScoringBreakdown {
            instruction_score: sb.instruction_score,
            onchain_score: sb.onchain_score,
            final_score: sb.final_score,
            issues: sb.issues,
            mismatches: sb.mismatches,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // std::collections::HashMap is no longer needed here
    use std::time::SystemTime;

    #[test]
    fn test_flow_log_conversion() {
        let reev_lib_flow = crate::flow::types::FlowLog {
            session_id: "test-session".to_string(),
            benchmark_id: "test-benchmark".to_string(),
            agent_type: "test-agent".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            events: vec![],
            final_result: None,
        };

        // Convert to shared FlowLog
        let shared_flow = reev_lib_flow.to_flow_log().unwrap();
        assert_eq!(shared_flow.session_id, reev_lib_flow.session_id);
        assert_eq!(shared_flow.benchmark_id, reev_lib_flow.benchmark_id);
        assert_eq!(shared_flow.agent_type, reev_lib_flow.agent_type);

        // Convert back to reev-lib FlowLog
        let converted_back = crate::flow::types::FlowLog::from_flow_log(&shared_flow).unwrap();
        assert_eq!(converted_back.session_id, reev_lib_flow.session_id);
        assert_eq!(converted_back.benchmark_id, reev_lib_flow.benchmark_id);
        assert_eq!(converted_back.agent_type, reev_lib_flow.agent_type);
    }
}
