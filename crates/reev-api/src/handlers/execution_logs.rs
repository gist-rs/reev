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

// Import parser from local handlers directory
use super::parsers::ExecutionTraceParser;

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

            // Generate ASCII trace using the parser
            let parser = ExecutionTraceParser::new();
            let trace = match parser.generate_trace_from_state(&updated_state).await {
                Ok(ascii_trace) => ascii_trace,
                Err(e) => {
                    warn!("Failed to generate trace from state: {}", e);
                    // Try fallback methods
                    match state.db.get_session_log(exec_id).await {
                        Ok(Some(log_content)) => {
                            match parser
                                .generate_trace_from_session_log(&log_content, exec_id)
                                .await
                            {
                                Ok(ascii_trace) => ascii_trace,
                                Err(e) => {
                                    warn!("Failed to generate trace from session log: {}", e);
                                    parser.generate_error_trace(&e.to_string(), exec_id)
                                }
                            }
                        }
                        Ok(None) => {
                            warn!("No session log found for execution_id: {}", exec_id);
                            "📝 No execution trace available".to_string()
                        }
                        Err(e) => {
                            warn!("Failed to get session log: {}", e);
                            parser.generate_error_trace(&e.to_string(), exec_id)
                        }
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

// Note: Trace generation logic has been moved to the parser module
// This improves maintainability and enables reuse across the codebase
