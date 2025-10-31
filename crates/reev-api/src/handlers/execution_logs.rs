//! Execution logs handler for benchmark execution information

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reev_db::writer::DatabaseWriterTrait;

use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::types::ApiState;

/// Get execution trace for a benchmark
pub async fn get_execution_trace(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Check if specific execution_id is requested
    let target_execution_id = params.get("execution_id");

    // If no execution_id is specified, return error to force frontend to use new two-step approach
    if target_execution_id.is_none() {
        let response = json!({
            "benchmark_id": benchmark_id,
            "error": "execution_id parameter is required",
            "message": "Please use GET /api/v1/benchmarks/{id} to get recent executions, then call with execution_id",
            "trace": "",
            "is_running": false,
            "progress": 0.0
        });

        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // ALWAYS check database first when execution_id is provided for fresh data
    // This ensures we don't return stale in-memory cache
    if let Some(ref exec_id) = target_execution_id {
        if let Ok(Some(updated_state)) = state.db.get_execution_state(exec_id).await {
            let db_status = updated_state.status.clone();

            info!(
                "📊 Returning fresh database data for execution_id: {} (status: {:?})",
                exec_id, db_status
            );

            // Generate ASCII trace from session data if available
            let trace = if let Some(ref result_data) = updated_state.result_data {
                match generate_ascii_trace_from_session_data(result_data, exec_id).await {
                    Ok(ascii_trace) => ascii_trace,
                    Err(e) => {
                        warn!("Failed to generate ASCII trace: {}", e);
                        format!("⚠️  Failed to generate ASCII tree: {e}")
                    }
                }
            } else {
                // Try to get session log as fallback and convert to FlowLog format
                match state.db.get_session_log(exec_id).await {
                    Ok(Some(log_content)) => {
                        match generate_ascii_trace_from_session_log(&log_content, exec_id).await {
                            Ok(ascii_trace) => ascii_trace,
                            Err(e) => {
                                warn!("Failed to generate ASCII trace from session log: {}", e);
                                format!("⚠️  Failed to generate ASCII tree from session log: {e}")
                            }
                        }
                    }
                    Ok(None) => {
                        warn!("No session log found for execution_id: {}", exec_id);
                        "📝 No execution trace available".to_string()
                    }
                    Err(e) => {
                        warn!("Failed to get session log: {}", e);
                        format!("⚠️  Failed to retrieve session log: {e}")
                    }
                }
            };

            // Return execution results with ASCII trace
            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": exec_id,
                "agent_type": updated_state.agent,
                "interface": "web",
                "status": format!("{db_status:?}").to_lowercase(),
                "final_status": db_status,
                "trace": trace,
                "is_running": false,
                "progress": 1.0,
                "result": updated_state.result_data.clone()
            });

            return Json(response).into_response();
        } else {
            warn!("❌ Database lookup failed for execution_id: {}", exec_id);

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": exec_id,
                "error": "Execution not found",
                "message": format!("No execution found with ID: {}", exec_id),
                "trace": "",
                "is_running": false,
                "progress": 0.0
            });

            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
    }

    // This should never be reached due to the check above
    let response = json!({
        "benchmark_id": benchmark_id,
        "error": "Invalid request",
        "message": "Missing execution_id parameter",
        "trace": "",
        "is_running": false,
        "progress": 0.0
    });

    (StatusCode::BAD_REQUEST, Json(response)).into_response()
}

/// Generate ASCII trace from session data by creating FlowLog struct and using existing renderer
async fn generate_ascii_trace_from_session_data(
    result_data: &serde_json::Value,
    execution_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use reev_lib::flow::{EventContent, FlowEventType, FlowLog, FlowLogRenderer};
    use reev_lib::flow::{ExecutionResult, ExecutionStatistics};
    use std::collections::HashMap;

    // Extract session information from result data
    let session_id = result_data
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or(execution_id);

    let benchmark_id = result_data
        .get("benchmark_id")
        .and_then(|v| v.as_str())
        .unwrap_or("001-sol-transfer");

    let agent_type = result_data
        .get("agent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("deterministic");

    let start_time = std::time::SystemTime::now();
    let end_time = start_time + std::time::Duration::from_secs(60); // Default 1 minute

    // Create FlowLog from session data
    let mut flow_log = FlowLog {
        session_id: session_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent_type: agent_type.to_string(),
        start_time,
        end_time: Some(end_time),
        events: Vec::new(),
        final_result: None, // Will be set below if data available
    };

    // Extract events from session data - convert steps to events
    if let Some(steps) = result_data.get("steps").and_then(|v| v.as_array()) {
        for (i, step_data) in steps.iter().enumerate() {
            let timestamp = (i as u64) * 1000; // Simple timestamp based on step order

            // Create LLM Request event for each step
            let flow_event = reev_lib::flow::FlowEvent {
                timestamp: std::time::SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_millis(timestamp),
                event_type: FlowEventType::LlmRequest,
                depth: i as u32,
                content: EventContent {
                    data: serde_json::json!({
                        "model": "deterministic",
                        "context_tokens": 1000,
                        "step_index": i
                    }),
                },
            };

            flow_log.events.push(flow_event);

            // Create Tool Call event for each action
            if let Some(action) = step_data.get("action").and_then(|v| v.as_array()) {
                if !action.is_empty() {
                    let tool_event = reev_lib::flow::FlowEvent {
                        timestamp: std::time::SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_millis(timestamp + 500),
                        event_type: FlowEventType::ToolCall,
                        depth: i as u32 + 1,
                        content: EventContent {
                            data: serde_json::json!({
                                "tool_name": "execute_transaction",
                                "tool_args": format!("Step {} action", i + 1)
                            }),
                        },
                    };

                    flow_log.events.push(tool_event);

                    // Create Tool Result event
                    let result_event = reev_lib::flow::FlowEvent {
                        timestamp: std::time::SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_millis(timestamp + 1000),
                        event_type: FlowEventType::ToolResult,
                        depth: i as u32 + 1,
                        content: EventContent {
                            data: serde_json::json!({
                                "tool_name": "execute_transaction",
                                "result_status": "success",
                                "result_data": action
                            }),
                        },
                    };

                    flow_log.events.push(result_event);
                }
            }
        }
    }

    // Extract final result - ensure always set for proper status display
    let success = result_data
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to success for completed executions

    let score = result_data
        .get("score")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let execution_result = ExecutionResult {
        success,
        score,
        total_time_ms: 60000, // Default 1 minute
        statistics: ExecutionStatistics {
            total_llm_calls: flow_log
                .events
                .iter()
                .filter(|e| matches!(e.event_type, FlowEventType::LlmRequest))
                .count() as u32,
            total_tool_calls: flow_log
                .events
                .iter()
                .filter(|e| matches!(e.event_type, FlowEventType::ToolCall))
                .count() as u32,
            total_tokens: 0,
            tool_usage: HashMap::new(),
            max_depth: 0,
        },
        scoring_breakdown: None,
    };

    flow_log.final_result = Some(execution_result);

    // Render as ASCII tree using the existing renderer
    Ok(flow_log.render_as_ascii_tree())
}

/// Generate ASCII trace from session log content by converting to FlowLog format and using existing renderer
async fn generate_ascii_trace_from_session_log(
    log_content: &str,
    execution_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Parse the log content as JSON to get session data
    let session_data: serde_json::Value = serde_json::from_str(log_content)?;

    // Use the same conversion logic as session data
    generate_ascii_trace_from_session_data(&session_data, execution_id).await
}
