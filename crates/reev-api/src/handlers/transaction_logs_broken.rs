//! Transaction logs handler for benchmark transaction information

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reev_db::writer::DatabaseWriterTrait;
use reev_types::execution::ExecutionStatus;

use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::types::ApiState;

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Getting transaction logs for benchmark: {}", benchmark_id);

    // Database-only approach: get most recent session for this benchmark
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: None,
        interface: Some("web".to_string()),
        status: None,
        limit: Some(1), // Get most recent session
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!("Found session for transaction logs: {}", session.session_id);

                // Check format parameter: yaml or plain (yaml is default)
                let format_param = params
                    .get("format")
                    .map_or("yaml".to_string(), |v| v.clone());

                // Check show_cu parameter: true or false (false is default)
                let show_cu_param = params
                    .get("show_cu")
                    .map_or("false".to_string(), |v| v.clone());
                let show_cu = show_cu_param == "true";

                let response = json!({
                    "benchmark_id": benchmark_id,
                    "execution_id": session.session_id,
                    "agent_type": session.agent_type,
                    "interface": session.interface,
                    "status": session.status,
                    "is_running": session.status == "running" || session.status == "queued",
                    "score": session.score
                });

                Json(response).into_response()
            } else {
                info!("No sessions found for benchmark: {}", benchmark_id);

                let response = json!({
                    "benchmark_id": benchmark_id,
                    "execution_id": null,
                    "session_log_available": false,
                    "is_running": false,
                    "message": "No execution history found for this benchmark"
                });

                Json(response).into_response()
            }
        }
        Err(e) => {
            warn!("Failed to get sessions for benchmark: {}", e);

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": null,
                "session_log_available": false,
                "is_running": false,
                "message": format!("Database error: {}", e)
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
