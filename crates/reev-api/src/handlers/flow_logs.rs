//! Flow log handlers
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_types::ExecutionStatus;
use serde_json::json;
use tracing::{error, info};

/// Get flow logs for a benchmark
pub async fn get_flow_log(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting session logs for benchmark: {}", benchmark_id);

    // Database-only approach: get all sessions for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: Some("web".to_string()),
        status: None,
        limit: None,
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            for session in sessions {
                let is_running = session.status == "running" || session.status == "queued";
                active_executions.push(json!({
                    "session_id": session.session_id,
                    "agent_type": session.agent_type,
                    "interface": session.interface,
                    "status": session.status,
                    "is_running": is_running,
                    "progress": if is_running { 0.5 } else { 1.0 },
                    "score": session.score,
                }));
            }
        }
        Err(e) => {
            warn!("Failed to get sessions from database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "Database error",
                    "message": format!("Failed to retrieve sessions: {}", e),
                    "sessions": []
                }),
            )
                .into_response();
        }
    }
    drop(executions);

    // If there are active executions, return them
    if !active_executions.is_empty() {
        info!(
            "Found {} active executions for benchmark: {}",
            active_executions.len(),
            benchmark_id
        );
        return Json(json!({
            "benchmark_id": benchmark_id,
            "sessions": active_executions
        }))
        .into_response();
    }

    // If no active executions, look for completed sessions
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: None,
        status: None,
        limit: None,
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if sessions.is_empty() {
                info!(
                    "No sessions or active executions found for benchmark: {}",
                    benchmark_id
                );
                Json(json!({"message": "No sessions found", "sessions": []})).into_response()
            } else {
                // Get logs for each session
                let mut session_logs = Vec::new();
                for session in sessions {
                    match state.db.get_session_log(&session.session_id).await {
                        Ok(log_content) => {
                            session_logs.push(json!({
                                "session_id": session.session_id,
                                "agent_type": session.agent_type,
                                "interface": session.interface,
                                "status": session.status,
                                "score": session.score,
                                "final_status": session.final_status,
                                "log_content": log_content,
                                "is_running": false
                            }));
                        }
                        Err(e) => {
                            error!(
                                "Failed to get log for session {}: {}",
                                session.session_id, e
                            );
                            session_logs.push(json!({
                                "session_id": session.session_id,
                                "agent_type": session.agent_type,
                                "interface": session.interface,
                                "status": session.status,
                                "score": session.score,
                                "final_status": session.final_status,
                                "error": format!("Failed to retrieve log: {}", e),
                                "is_running": false
                            }));
                        }
                    }
                }

                info!(
                    "Found {} completed sessions for benchmark: {}",
                    session_logs.len(),
                    benchmark_id
                );
                Json(json!({
                    "benchmark_id": benchmark_id,
                    "sessions": session_logs
                }))
                .into_response()
            }
        }
        Err(e) => {
            error!(
                "Failed to get sessions for benchmark {}: {}",
                benchmark_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
