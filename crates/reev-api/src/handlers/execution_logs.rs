use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reev_types::execution::{ExecutionState, ExecutionStatus};

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
            || execution.status == ExecutionStatus::Queued;

        info!(
            "Found execution for benchmark: {} (status: {:?})",
            benchmark_id, execution.status
        );

        // Handle running executions like execution trace - return raw trace or loading message
        if is_running {
            let trace_data = execution
                .metadata
                .get("trace")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let execution_trace = if trace_data.trim().is_empty() {
                "🔄 Loading execution trace...\n\n⏳ Execution in progress - please wait"
                    .to_string()
            } else {
                // Return the raw execution trace
                trace_data.to_string()
            };

            // For running executions, try to format if trace has content, otherwise show loading
            if !trace_data.trim().is_empty() {
                match format_execution_trace(trace_data, execution_id.clone()) {
                    Ok(execution_state) => {
                        info!("Successfully formatted execution trace for running execution");
                        return Json(execution_state).into_response();
                    }
                    Err(e) => {
                        warn!(
                            "Failed to format running execution trace: {}, returning raw",
                            e
                        );
                        // Fall through to raw response
                    }
                }
            }

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

        // For completed executions with trace data, format it using the ASCII tree formatter
        let trace_data = execution
            .metadata
            .get("trace")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !trace_data.is_empty() {
            match format_execution_trace(trace_data, execution_id.clone()) {
                Ok(execution_state) => {
                    info!("Successfully formatted execution trace for completed execution");
                    return Json(execution_state).into_response();
                }
                Err(e) => {
                    warn!(
                        "Failed to format completed execution trace: {}, returning raw",
                        e
                    );

                    // Fallback to raw response if formatting fails
                    let response = json!({
                        "benchmark_id": benchmark_id,
                        "execution_id": execution_id,
                        "agent_type": execution.agent,
                        "interface": "web",
                        "status": format!("{:?}", execution.status).to_lowercase(),
                        "final_status": execution.status,
                        "trace": execution.metadata.get("trace").cloned().unwrap_or(serde_json::Value::String(String::new())),
                        "is_running": false,
                        "progress": execution.progress
                    });

                    info!("Returning raw execution trace for completed execution (fallback)");
                    return Json(response).into_response();
                }
            }
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

                        // Format execution trace using the same function as benchmarks.rs
                        match format_execution_trace(&log_content, session.session_id.clone()) {
                            Ok(execution_state) => {
                                info!("Successfully formatted execution trace for session");
                                Json(execution_state).into_response()
                            }
                            Err(e) => {
                                warn!("Failed to format execution trace: {}, returning raw", e);

                                // Fallback to raw JSON response if formatting fails
                                let response = json!({
                                    "benchmark_id": benchmark_id,
                                    "execution_id": session.session_id,
                                    "agent_type": session.agent_type,
                                    "interface": "web",
                                    "status": format!("{:?}", session.status).to_lowercase(),
                                    "final_status": session.status,
                                    "trace": log_content,
                                    "is_running": false,
                                    "progress": 0
                                });

                                info!("DEBUG: Successfully created execution trace response (raw fallback)");
                                Json(response).into_response()
                            }
                        }
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
                        Json(response).into_response()
                    }
                    Err(e) => {
                        warn!("Failed to get session log: {}", e);

                        let response = json!({
                            "benchmark_id": benchmark_id,
                            "trace": "",
                            "is_running": false,
                            "message": format!("Database error: {}", e)
                        });

                        (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
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

                Json(response).into_response()
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

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Helper function to format execution trace from log content
fn format_execution_trace(
    log_content: &str,
    execution_id: String,
) -> Result<ExecutionState, Box<dyn std::error::Error + Send + Sync>> {
    // Parse the flow log and format it as readable trace
    match serde_json::from_str::<serde_json::Value>(log_content) {
        Ok(parsed) => {
            let mut formatted_trace = String::new();

            if let Some(prompt) = parsed.get("prompt").and_then(|v| v.as_str()) {
                formatted_trace.push_str(&format!("📝 Prompt: {prompt}\n\n"));
            }

            if let Some(steps) = parsed.get("steps").and_then(|v| v.as_array()) {
                for (i, step) in steps.iter().enumerate() {
                    formatted_trace.push_str(&format!("✓ Step {}\n", i + 1));

                    if let Some(action) = step.get("action") {
                        formatted_trace.push_str("   ├─ ACTION:\n");
                        if let Some(action_array) = action.as_array() {
                            for action_item in action_array {
                                if let Some(program_id) = action_item.get("program_id") {
                                    formatted_trace
                                        .push_str(&format!("      Program ID: {program_id}\n"));
                                }
                                if let Some(accounts) = action_item.get("accounts") {
                                    if let Some(accounts_array) = accounts.as_array() {
                                        formatted_trace.push_str("      Accounts:\n");
                                        for (idx, account) in accounts_array.iter().enumerate() {
                                            if let Some(pubkey) = account.get("pubkey") {
                                                let is_signer = account
                                                    .get("is_signer")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false);
                                                let is_writable = account
                                                    .get("is_writable")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false);
                                                let icon =
                                                    if is_signer { "🖋️" } else { "🖍️" };
                                                let arrow = if is_writable { "➕" } else { "➖" };
                                                formatted_trace.push_str(&format!(
                                                    "      [{idx}] {icon} {arrow} {pubkey}\n"
                                                ));
                                            }
                                        }
                                    }
                                }
                                if let Some(data) = action_item.get("data") {
                                    formatted_trace
                                        .push_str(&format!("      Data (Base58): {data}\n"));
                                }
                            }
                        }
                    }

                    if let Some(observation) = step.get("observation") {
                        formatted_trace.push_str("   └─ OBSERVATION: ");
                        if let Some(status) = observation.get("last_transaction_status") {
                            formatted_trace.push_str(&format!("{status}\n"));
                        }
                        if let Some(error) = observation.get("last_transaction_error") {
                            if !error.as_str().unwrap_or("").is_empty() {
                                formatted_trace.push_str(&format!("   Error: {error}\n"));
                            }
                        }
                    }
                    formatted_trace.push('\n');
                }
            }

            // Add final success message
            formatted_trace.push_str("✅ Execution completed - Full trace displayed above\n");

            // Extract benchmark_id from the trace if available
            let benchmark_id = parsed
                .get("benchmark_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let mut execution_state = ExecutionState::new(
                execution_id,
                benchmark_id,
                "deterministic".to_string(), // Default agent
            );
            execution_state.status = ExecutionStatus::Completed;
            execution_state.progress = Some(1.0);
            execution_state.add_metadata("trace", serde_json::Value::String(formatted_trace));
            Ok(execution_state)
        }
        Err(e) => Err(format!("Failed to parse execution trace: {e}").into()),
    }
}
