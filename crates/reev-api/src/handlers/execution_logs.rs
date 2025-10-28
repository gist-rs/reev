use crate::types::ExecutionStatus;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::types::ApiState;

/// Get execution trace for a benchmark
pub async fn get_execution_trace(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Getting execution trace for benchmark: {}", benchmark_id);

    // First check for active executions (like transaction logs does)
    let executions = state.executions.lock().await;
    let mut found_execution = None;

    for (execution_id, execution) in executions.iter() {
        if execution.benchmark_id == benchmark_id {
            found_execution = Some((execution_id.clone(), execution.clone()));
            break;
        }
    }

    drop(executions); // Release lock before processing

    if let Some((execution_id, execution)) = found_execution {
        let is_running = execution.status == ExecutionStatus::Running
            || execution.status == ExecutionStatus::Pending;

        info!(
            "Found execution for benchmark: {} (status: {:?})",
            benchmark_id, execution.status
        );

        // Handle running executions like execution trace - return raw trace or loading message
        if is_running {
            let execution_trace = if execution.trace.trim().is_empty() {
                "🔄 Loading execution trace...\n\n⏳ Execution in progress - please wait"
                    .to_string()
            } else {
                // Return the raw execution trace
                execution.trace.clone()
            };

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "agent_type": execution.agent,
                "interface": "web",
                "status": format!("{:?}", execution.status).to_lowercase(),
                "final_status": execution.status,
                "trace": execution_trace,
                "is_running": is_running,
                "progress": execution.progress
            });

            info!("Returning execution trace for running execution");
            return Json(response).into_response();
        }

        // For completed executions with trace data, use the in-memory trace
        if !execution.trace.is_empty() {
            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "agent_type": execution.agent,
                "interface": "web",
                "status": format!("{:?}", execution.status).to_lowercase(),
                "final_status": execution.status,
                "trace": execution.trace,
                "is_running": false,
                "progress": execution.progress
            });

            info!("Returning execution trace for completed execution");
            return Json(response).into_response();
        }
    }

    // If no active execution found, look for sessions in database
    info!(
        "No active execution found for benchmark: {}, checking database sessions",
        benchmark_id
    );

    // Get most recent session for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!("Found session for benchmark: {}", benchmark_id);

                // Get the session log which contains execution trace
                info!(
                    "DEBUG: Attempting to get session log for session: {}",
                    session.session_id
                );

                match state.db.get_session_log(&session.session_id).await {
                    Ok(Some(log_content)) => {
                        info!("DEBUG: Got session log content");

                        let log_preview = if log_content.len() > 200 {
                            format!("{}...", &log_content[..200])
                        } else {
                            log_content.clone()
                        };

                        info!("DEBUG: Session log preview: {}", log_preview);

                        let response = json!({
                            "benchmark_id": benchmark_id,
                            "execution_id": session.session_id,
                            "agent_type": session.agent_type,
                            "interface": "web",
                            "status": format!("{:?}", session.status).to_lowercase(),
                            "final_status": session.status,
                            "trace": log_content,
                            "is_running": false,
                            // Note: SessionInfo doesn't have progress field, using default
                            "progress": 0
                        });

                        info!("DEBUG: Successfully created execution trace response");
                        return Json(response).into_response();
                    }
                    Ok(None) => {
                        warn!("No session log found for session: {}", session.session_id);

                        let response = json!({
                            "benchmark_id": benchmark_id,
                            "execution_id": session.session_id,
                            "trace": "",
                            "is_running": false,
                            "message": "No session log found for execution"
                        });

                        info!("DEBUG: Successfully created no-log response");
                        return Json(response).into_response();
                    }
                    Err(e) => {
                        warn!("Failed to get session log: {}", e);

                        let response = json!({
                            "benchmark_id": benchmark_id,
                            "trace": "",
                            "is_running": false,
                            "message": format!("Database error: {}", e)
                        });

                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
                    }
                }
            } else {
                info!("No sessions found for benchmark: {}", benchmark_id);

                let response = json!({
                    "benchmark_id": benchmark_id,
                    "trace": "",
                    "is_running": false,
                    "message": "No execution history found for this benchmark"
                });

                return Json(response).into_response();
            }
        }
        Err(e) => {
            warn!("Failed to get sessions for benchmark: {}", e);

            let response = json!({
                "benchmark_id": benchmark_id,
                "trace": "",
                "is_running": false,
                "message": format!("Database error: {}", e)
            });

            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }
}
