//! Transaction logs handler for benchmark transaction information

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

// Import parser from local handlers directory
use super::parsers::ExecutionTraceParser;

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Check if specific execution_id is requested
    let target_execution_id = params.get("execution_id");

    // Use provided execution_id or find most recent execution for this benchmark
    let execution_id = if let Some(exec_id) = target_execution_id {
        exec_id.clone()
    } else {
        // Find most recent execution for this benchmark
        match state
            .db
            .list_execution_states_by_benchmark(&benchmark_id)
            .await
        {
            Ok(executions) => {
                if let Some(latest_execution) = executions.first() {
                    info!(
                        "Using latest execution_id for benchmark {}: {}",
                        benchmark_id, latest_execution.execution_id
                    );
                    latest_execution.execution_id.clone()
                } else {
                    warn!("No executions found for benchmark: {}", benchmark_id);
                    return Json(json!({
                        "benchmark_id": benchmark_id,
                        "execution_id": null,
                        "is_running": false,
                        "message": "No execution history found for this benchmark",
                        "trace": "📝 No execution trace available"
                    }))
                    .into_response();
                }
            }
            Err(e) => {
                warn!(
                    "Failed to list executions for benchmark {}: {}",
                    benchmark_id, e
                );
                return Json(json!({
                    "benchmark_id": benchmark_id,
                    "execution_id": null,
                    "is_running": false,
                    "message": format!("Database error: {}", e),
                    "trace": "❌ Database error occurred"
                }))
                .into_response();
            }
        }
    };

    // ALWAYS check database first when execution_id is provided for fresh data
    // This ensures we don't return stale in-memory cache
    match state.db.get_execution_state(&execution_id).await {
        Ok(Some(updated_state)) => {
            let db_status = updated_state.status.clone();

            info!(
                "📊 Returning fresh database data for transaction logs execution_id: {} (status: {:?})",
                execution_id, db_status
            );

            // Generate ASCII trace using the parser
            let parser = ExecutionTraceParser::new();
            let trace = match parser.generate_trace_from_state(&updated_state).await {
                Ok(ascii_trace) => ascii_trace,
                Err(e) => {
                    warn!("Failed to generate trace from state: {}", e);
                    // Try fallback methods
                    match state.db.get_session_log(&execution_id).await {
                        Ok(Some(log_content)) => {
                            match parser
                                .generate_trace_from_session_log(&log_content, &execution_id)
                                .await
                            {
                                Ok(ascii_trace) => ascii_trace,
                                Err(e) => {
                                    warn!("Failed to generate trace from session log: {}", e);
                                    parser.generate_error_trace(&e.to_string(), &execution_id)
                                }
                            }
                        }
                        Ok(None) => {
                            warn!("No session log found for execution_id: {}", execution_id);
                            "📝 No execution trace available".to_string()
                        }
                        Err(e) => {
                            warn!("Failed to get session log: {}", e);
                            parser.generate_error_trace(&e.to_string(), &execution_id)
                        }
                    }
                }
            };

            // Return execution results with ASCII trace
            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "agent_type": updated_state.agent,
                "interface": "web",
                "status": format!("{db_status:?}").to_lowercase(),
                "final_status": db_status,
                "trace": trace,
                "is_running": false,
                "progress": 1.0,
                "result": updated_state.result_data.clone()
            });

            Json(response).into_response()
        }
        Ok(None) => {
            warn!(
                "❌ No execution state found for execution_id: {}",
                execution_id
            );

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "error": "Execution not found",
                "message": format!("No execution found with ID: {}", execution_id),
                "trace": "",
                "is_running": false,
                "progress": 0.0
            });

            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            warn!(
                "❌ Database lookup failed for execution_id: {}",
                execution_id
            );

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "error": "Database error",
                "message": format!("Failed to get execution: {}", e),
                "trace": "",
                "is_running": false,
                "progress": 0.0
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Note: Trace generation logic has been moved to the parser module
// This improves maintainability and enables reuse across the codebase
