//! ASCII tree rendering handlers
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tracing::{error, info, warn};

/// Get ASCII tree directly from YML TestResult in database
pub async fn get_ascii_tree_direct(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting execution log for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Get the most recent session for this benchmark and agent
    let filter = reev_db::types::SessionFilter {
        benchmark_id: Some(benchmark_id.clone()),
        agent_type: Some(agent_type.clone()),
        interface: None,
        status: None,
        limit: Some(1), // Get the most recent session
    };

    match state.db.list_sessions(&filter).await {
        Ok(sessions) => {
            if let Some(session) = sessions.first() {
                info!(
                    "Found session for benchmark: {} by agent: {} with status: {}",
                    benchmark_id,
                    agent_type,
                    session.final_status.as_deref().unwrap_or("Unknown")
                );

                // Check execution status
                match session.final_status.as_deref() {
                    Some("Running") | Some("running") => (
                        StatusCode::OK,
                        [("Content-Type", "text/plain")],
                        "⏳ Execution in progress...".to_string(),
                    )
                        .into_response(),
                    Some("Completed") | Some("Succeeded") | Some("completed")
                    | Some("succeeded") => {
                        // Get the session log which contains the full execution output
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(Some(log_content)) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "📝 No execution data available".to_string(),
                                    )
                                        .into_response();
                                }

                                // For now, return the raw log content
                                // TODO: Implement proper ASCII tree rendering
                                (
                                    StatusCode::OK,
                                    [("Content-Type", "text/plain")],
                                    format!(
                                        "📊 Execution Trace:\n\n{}",
                                        &log_content[..log_content.len().min(1000)]
                                    ),
                                )
                                    .into_response()
                            }
                            Ok(None) => (
                                StatusCode::OK,
                                [("Content-Type", "text/plain")],
                                "📝 No execution data available".to_string(),
                            )
                                .into_response(),
                            Err(e) => {
                                warn!("Failed to get session log: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve execution data".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    Some("Failed") | Some("failed") | Some("Error") | Some("error") => {
                        // Get the session log even for failed executions to show error details
                        match state.db.get_session_log(&session.session_id).await {
                            Ok(Some(log_content)) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "❌ Execution failed - No details available".to_string(),
                                    )
                                        .into_response();
                                }

                                // For now, return the raw log content
                                // TODO: Implement proper ASCII tree rendering for failed executions
                                (
                                    StatusCode::OK,
                                    [("Content-Type", "text/plain")],
                                    format!(
                                        "❌ Failed Execution:\n\n{}",
                                        &log_content[..log_content.len().min(1000)]
                                    ),
                                )
                                    .into_response()
                            }
                            Ok(None) => (
                                StatusCode::OK,
                                [("Content-Type", "text/plain")],
                                "❌ Execution failed - No details available".to_string(),
                            )
                                .into_response(),
                            Err(e) => {
                                warn!("Failed to get session log for failed execution: {}", e);
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    [("Content-Type", "text/plain")],
                                    "❌ Failed to retrieve error details".to_string(),
                                )
                                    .into_response()
                            }
                        }
                    }
                    _ => {
                        info!(
                            "Unknown session status: {} for benchmark: {}",
                            session.final_status.as_deref().unwrap_or("None"),
                            benchmark_id
                        );
                        (
                            StatusCode::OK,
                            [("Content-Type", "text/plain")],
                            format!(
                                "❓ Unknown execution status: {}",
                                session.final_status.as_deref().unwrap_or("None")
                            ),
                        )
                            .into_response()
                    }
                }
            } else {
                info!(
                    "No sessions found for benchmark: {} by agent: {}",
                    benchmark_id, agent_type
                );
                (
                    StatusCode::OK,
                    [("Content-Type", "text/plain")],
                    "📝 No execution history found".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "text/plain")],
                "❌ Failed to retrieve execution history".to_string(),
            )
                .into_response()
        }
    }
}

/// Render ASCII tree from JSON test result
pub async fn render_ascii_tree(
    State(_state): State<ApiState>,
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Rendering ASCII tree from JSON payload");

    // For now, return a simple message
    // TODO: Implement proper ASCII tree rendering from JSON
    (
        StatusCode::OK,
        [("Content-Type", "text/plain")],
        "🌳 ASCII tree rendering from JSON not yet implemented".to_string(),
    )
        .into_response()
}

/// Parse YML to TestResult
pub async fn parse_yml_to_testresult(
    State(_state): State<ApiState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("Parsing YML to TestResult");

    // For now, return a simple message
    // TODO: Implement proper YML parsing
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "YML parsing not yet implemented",
            "received": payload
        })),
    )
        .into_response()
}

/// Get ASCII tree from execution state (temporary endpoint)
pub async fn get_ascii_tree_from_state(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting ASCII tree from execution state for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    // Check if there's an active execution in memory
    let executions = state.executions.lock().await;
    let execution_key = format!("{benchmark_id}:{agent_type}");

    if let Some(execution) = executions.get(&execution_key) {
        match execution.status {
            crate::types::ExecutionStatus::Running => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                "⏳ Execution in progress...".to_string(),
            )
                .into_response(),
            crate::types::ExecutionStatus::Completed => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                format!(
                    "✅ Execution completed\n\nTrace:\n{}",
                    &execution.trace[..execution.trace.len().min(500)]
                ),
            )
                .into_response(),
            crate::types::ExecutionStatus::Failed => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                format!(
                    "❌ Execution failed\n\nError: {}\n\nTrace:\n{}",
                    execution.error.as_deref().unwrap_or("Unknown error"),
                    &execution.trace[..execution.trace.len().min(500)]
                ),
            )
                .into_response(),
            _ => (
                StatusCode::OK,
                [("Content-Type", "text/plain")],
                "📝 Execution status unknown".to_string(),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::OK,
            [("Content-Type", "text/plain")],
            "📝 No active execution found".to_string(),
        )
            .into_response()
    }
}
