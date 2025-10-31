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
                    // Try fallback methods - prioritize session file for flow benchmarks
                    match state.db.get_session_log(exec_id).await {
                        Ok(Some(log_content)) => {
                            match parser
                                .generate_trace_from_session_log(&log_content, exec_id)
                                .await
                            {
                                Ok(ascii_trace) => ascii_trace,
                                Err(e) => {
                                    warn!("Failed to generate trace from session log: {}", e);
                                    // Final fallback: try to get session file directly for flow benchmarks
                                    match try_session_file_fallback(exec_id).await {
                                        Ok(Some(session_data)) => {
                                            match parser
                                                .generate_trace_from_session_data(
                                                    &session_data,
                                                    exec_id,
                                                )
                                                .await
                                            {
                                                Ok(ascii_trace) => ascii_trace,
                                                Err(e) => parser.generate_error_trace(
                                                    &format!("Session file parse failed: {e}"),
                                                    exec_id,
                                                ),
                                            }
                                        }
                                        Ok(None) => parser
                                            .generate_error_trace("No session data found", exec_id),
                                        Err(e) => parser.generate_error_trace(
                                            &format!("Session file error: {e}"),
                                            exec_id,
                                        ),
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            warn!("No session log found for execution_id: {}", exec_id);
                            // Try session file fallback for flow benchmarks
                            info!(
                                "🔍 Attempting session file fallback for execution_id: {}",
                                exec_id
                            );
                            match try_session_file_fallback(exec_id).await {
                                Ok(Some(session_data)) => {
                                    info!("✅ Session file found, generating trace from session data for execution_id: {}", exec_id);
                                    match parser
                                        .generate_trace_from_session_data(&session_data, exec_id)
                                        .await
                                    {
                                        Ok(ascii_trace) => {
                                            info!("✅ Successfully generated trace from session file for execution_id: {}", exec_id);
                                            ascii_trace
                                        }
                                        Err(e) => parser.generate_error_trace(
                                            &format!("Session file parse failed: {e}"),
                                            exec_id,
                                        ),
                                    }
                                }
                                Ok(None) => {
                                    warn!("❌ No session file found for execution_id: {}", exec_id);
                                    parser.generate_error_trace("No session data found", exec_id)
                                }
                                Err(e) => {
                                    warn!(
                                        "❌ Session file error for execution_id {}: {}",
                                        exec_id, e
                                    );
                                    parser.generate_error_trace(
                                        &format!("Session file error: {e}"),
                                        exec_id,
                                    )
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get session log: {}", e);
                            parser.generate_error_trace(&e.to_string(), exec_id)
                        }
                    }
                }
            };

            // Create a clean result object without huge stdout
            // Only include essential execution metadata, not raw logs
            let clean_result = if let Some(ref result_data) = updated_state.result_data {
                // For flow benchmarks, result_data contains huge stdout
                // Extract only essential fields to avoid sending huge logs to web
                json!({
                    "duration_ms": result_data.get("duration_ms").unwrap_or(&json!(0)),
                    "exit_code": result_data.get("exit_code").unwrap_or(&json!(0)),
                    // Exclude "stdout" field as it contains huge logs
                    // Session file data will be used for trace generation instead
                })
            } else {
                json!({})
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
                "result": clean_result
            });

            return Json(response).into_response();
        } else {
            warn!("❌ Database lookup failed for execution_id: {}", exec_id);
            info!(
                "🔍 Attempting session file fallback for execution_id: {}",
                exec_id
            );

            // Try session file fallback for flow benchmarks when database lookup fails
            let parser = ExecutionTraceParser::new();
            let trace = match try_session_file_fallback(exec_id).await {
                Ok(Some(session_data)) => {
                    info!("✅ Session file found, generating trace from session data for execution_id: {}", exec_id);
                    match parser
                        .generate_trace_from_session_data(&session_data, exec_id)
                        .await
                    {
                        Ok(ascii_trace) => {
                            info!("✅ Successfully generated trace from session file for execution_id: {}", exec_id);
                            ascii_trace
                        }
                        Err(e) => {
                            warn!("❌ Failed to generate trace from session file for execution_id: {}: {}", exec_id, e);
                            parser.generate_error_trace(
                                &format!("Session file parse failed: {e}"),
                                exec_id,
                            )
                        }
                    }
                }
                Ok(None) => {
                    warn!("❌ No session file found for execution_id: {}", exec_id);
                    parser.generate_error_trace("No session data found", exec_id)
                }
                Err(e) => {
                    warn!("❌ Session file error for execution_id {}: {}", exec_id, e);
                    parser.generate_error_trace(&format!("Session file error: {e}"), exec_id)
                }
            };

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": exec_id,
                "agent_type": "FlowAgent",
                "interface": "web",
                "status": "completed",
                "final_status": "Completed",
                "trace": trace,
                "is_running": false,
                "progress": 1.0,
                "result": {
                    "duration_ms": 0,
                    "exit_code": 0
                }
            });

            return Json(response).into_response();
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

/// Try to read session file directly as fallback for flow benchmarks
async fn try_session_file_fallback(
    execution_id: &str,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Try to find session file for this execution_id
    let sessions_dir = std::path::Path::new("logs/sessions");

    if !sessions_dir.exists() {
        return Ok(None);
    }

    // Look for session file with this execution_id
    let session_pattern = format!("session_{execution_id}.json");

    for entry in std::fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name == session_pattern {
            let content = std::fs::read_to_string(entry.path())?;
            let session_data: serde_json::Value = serde_json::from_str(&content)?;
            return Ok(Some(session_data));
        }
    }

    Ok(None)
}

// Note: Trace generation logic has been moved to the parser module
// This improves maintainability and enables reuse across the codebase
