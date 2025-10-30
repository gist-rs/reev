//! Flow log handlers
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::{info, warn};

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
            let mut active_executions = Vec::new();

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

            info!(
                "Found {} executions for benchmark: {}",
                active_executions.len(),
                benchmark_id
            );

            Json(json!({
                "benchmark_id": benchmark_id,
                "sessions": active_executions
            }))
            .into_response()
        }
        Err(e) => {
            warn!("Failed to get sessions from database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error",
                    "message": format!("Failed to retrieve sessions: {}", e),
                    "sessions": []
                })),
            )
                .into_response()
        }
    }
}
