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
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "📝 No execution data available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace first (new format)
                                match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(
                                    &log_content,
                                ) {
                                    Ok(trace) => {
                                        // Create a TestResult from the trace for rendering
                                        let test_result = reev_lib::results::TestResult::new(
                                            &reev_lib::benchmark::TestCase {
                                                id: benchmark_id.clone(),
                                                description: format!(
                                                    "Execution result for {benchmark_id}"
                                                ),
                                                tags: vec!["web".to_string()],
                                                initial_state: vec![],
                                                prompt: trace.prompt.clone(),
                                                flow: None,
                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                    transaction_status: "unknown".to_string(),
                                                    final_state_assertions: vec![],
                                                    expected_instructions: vec![],
                                                    skip_instruction_validation: false,
                                                },
                                            },
                                            reev_lib::results::FinalStatus::Succeeded,
                                            session.score.unwrap_or(1.0),
                                            trace,
                                        );

                                        // Render as ASCII tree using the existing renderer
                                        let ascii_tree =
                                            reev_runner::renderer::render_result_as_tree(
                                                &test_result,
                                            );

                                        info!(
                                            "Rendered ASCII tree for benchmark: {} ({} chars) - New format",
                                            benchmark_id,
                                            ascii_tree.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            ascii_tree,
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // Try to parse as old SessionLog format and extract ExecutionTrace
                                        match serde_json::from_str::<
                                            reev_lib::session_logger::SessionLog,
                                        >(&log_content)
                                        {
                                            Ok(session_log) => {
                                                // Check if final_result contains ExecutionTrace in data field
                                                if let Some(final_result) =
                                                    &session_log.final_result
                                                {
                                                    if let Ok(trace) = serde_json::from_value::<
                                                        reev_lib::trace::ExecutionTrace,
                                                    >(
                                                        final_result.data.clone()
                                                    ) {
                                                        // Create a TestResult from the extracted trace
                                                        let test_result = reev_lib::results::TestResult::new(
                                                            &reev_lib::benchmark::TestCase {
                                                                id: benchmark_id.clone(),
                                                                description: format!(
                                                                    "Execution result for {benchmark_id}"
                                                                ),
                                                                tags: vec!["tui".to_string()],
                                                                initial_state: vec![],
                                                                prompt: trace.prompt.clone(),
                                                                flow: None,
                                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                                    transaction_status: "unknown".to_string(),
                                                                    final_state_assertions: vec![],
                                                                    expected_instructions: vec![],
                                                                    skip_instruction_validation: false,
                                                                },
                                                            },
                                                            reev_lib::results::FinalStatus::Succeeded,
                                                            final_result.score,
                                                            trace,
                                                        );

                                                        // Render as ASCII tree using the existing renderer
                                                        let ascii_tree =
                                                            reev_runner::renderer::render_result_as_tree(
                                                                &test_result,
                                                            );

                                                        info!(
                                                            "Rendered ASCII tree for benchmark: {} ({} chars) - Migrated format",
                                                            benchmark_id,
                                                            ascii_tree.len()
                                                        );
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            ascii_tree,
                                                        )
                                                            .into_response()
                                                    } else {
                                                        warn!("Failed to extract ExecutionTrace from SessionLog data");
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Legacy session format detected - unable to extract ExecutionTrace\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                        )
                                                            .into_response()
                                                    }
                                                } else {
                                                    warn!("SessionLog has no final_result");
                                                    (
                                                        StatusCode::OK,
                                                        [("Content-Type", "text/plain")],
                                                        format!("❌ Legacy session format detected - no final_result available\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                    )
                                                        .into_response()
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to parse log as both ExecutionTrace and SessionLog: {}", e);
                                                // Return raw log if not a valid format
                                                info!(
                                                    "Returning raw execution log for benchmark: {} ({} chars)",
                                                    benchmark_id,
                                                    log_content.len()
                                                );
                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    format!("❌ Unknown session format - cannot render ASCII tree\n\nRaw log:\n{}", &log_content[..log_content.len().min(1000)]),
                                                )
                                                    .into_response()
                                            }
                                        }
                                    }
                                }
                            }
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
                            Ok(log_content) => {
                                if log_content.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        [("Content-Type", "text/plain")],
                                        "❌ Execution failed - No details available".to_string(),
                                    )
                                        .into_response();
                                }

                                // Try to parse as ExecutionTrace first (new format)
                                match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(
                                    &log_content,
                                ) {
                                    Ok(trace) => {
                                        // Create a TestResult from the trace for rendering
                                        let test_result = reev_lib::results::TestResult::new(
                                            &reev_lib::benchmark::TestCase {
                                                id: benchmark_id.clone(),
                                                description: format!(
                                                    "Failed execution result for {benchmark_id}"
                                                ),
                                                tags: vec!["web".to_string()],
                                                initial_state: vec![],
                                                prompt: trace.prompt.clone(),
                                                flow: None,
                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                    transaction_status: "unknown".to_string(),
                                                    final_state_assertions: vec![],
                                                    expected_instructions: vec![],
                                                    skip_instruction_validation: false,
                                                },
                                            },
                                            reev_lib::results::FinalStatus::Failed,
                                            session.score.unwrap_or(0.0),
                                            trace,
                                        );

                                        // Render as ASCII tree using the existing renderer
                                        let ascii_tree =
                                            reev_runner::renderer::render_result_as_tree(
                                                &test_result,
                                            );

                                        info!(
                                            "Rendered ASCII tree for failed benchmark: {} ({} chars) - New format",
                                            benchmark_id,
                                            ascii_tree.len()
                                        );
                                        (
                                            StatusCode::OK,
                                            [("Content-Type", "text/plain")],
                                            format!("❌ Execution Failed\n\n{ascii_tree}"),
                                        )
                                            .into_response()
                                    }
                                    Err(_) => {
                                        // Try to parse as old SessionLog format and extract ExecutionTrace
                                        match serde_json::from_str::<
                                            reev_lib::session_logger::SessionLog,
                                        >(&log_content)
                                        {
                                            Ok(session_log) => {
                                                // Check if final_result contains ExecutionTrace in data field
                                                if let Some(final_result) =
                                                    &session_log.final_result
                                                {
                                                    if let Ok(trace) = serde_json::from_value::<
                                                        reev_lib::trace::ExecutionTrace,
                                                    >(
                                                        final_result.data.clone()
                                                    ) {
                                                        // Create a TestResult from the extracted trace
                                                        let test_result = reev_lib::results::TestResult::new(
                                                            &reev_lib::benchmark::TestCase {
                                                                id: benchmark_id.clone(),
                                                                description: format!(
                                                                    "Failed execution result for {benchmark_id}"
                                                                ),
                                                                tags: vec!["tui".to_string()],
                                                                initial_state: vec![],
                                                                prompt: trace.prompt.clone(),
                                                                flow: None,
                                                                ground_truth: reev_lib::benchmark::GroundTruth {
                                                                    transaction_status: "unknown".to_string(),
                                                                    final_state_assertions: vec![],
                                                                    expected_instructions: vec![],
                                                                    skip_instruction_validation: false,
                                                                },
                                                            },
                                                            reev_lib::results::FinalStatus::Failed,
                                                            final_result.score,
                                                            trace,
                                                        );

                                                        // Render as ASCII tree using the existing renderer
                                                        let ascii_tree =
                                                            reev_runner::renderer::render_result_as_tree(
                                                                &test_result,
                                                            );

                                                        info!(
                                                            "Rendered ASCII tree for failed benchmark: {} ({} chars) - Migrated format",
                                                            benchmark_id,
                                                            ascii_tree.len()
                                                        );
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Execution Failed\n\n{ascii_tree}"),
                                                        )
                                                            .into_response()
                                                    } else {
                                                        warn!("Failed to extract ExecutionTrace from failed SessionLog data");
                                                        (
                                                            StatusCode::OK,
                                                            [("Content-Type", "text/plain")],
                                                            format!("❌ Failed execution - Legacy session format detected, unable to extract ExecutionTrace\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                        )
                                                            .into_response()
                                                    }
                                                } else {
                                                    warn!("Failed SessionLog has no final_result");
                                                    (
                                                        StatusCode::OK,
                                                        [("Content-Type", "text/plain")],
                                                        format!("❌ Failed execution - Legacy session format detected, no final_result available\n\nRaw log preview:\n{}", &log_content[..log_content.len().min(500)]),
                                                    )
                                                        .into_response()
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to parse failed log as both ExecutionTrace and SessionLog: {}", e);
                                                // Return raw log if not a valid format
                                                info!(
                                                    "Returning raw failed execution log for benchmark: {} ({} chars)",
                                                    benchmark_id,
                                                    log_content.len()
                                                );
                                                (
                                                    StatusCode::OK,
                                                    [("Content-Type", "text/plain")],
                                                    format!("❌ Failed execution - Unknown session format, cannot render ASCII tree\n\nRaw log:\n{}", &log_content[..log_content.len().min(1000)]),
                                                )
                                                    .into_response()
                                            }
                                        }
                                    }
                                }
                            }
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
                            "❓ Unknown execution status".to_string(),
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
                    "📭 No execution data found".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("Content-Type", "text/plain")],
                "❌ Database error".to_string(),
            )
                .into_response()
        }
    }
}

/// Render TestResult as ASCII tree
pub async fn render_ascii_tree(Json(test_result): Json<serde_json::Value>) -> impl IntoResponse {
    info!("Rendering ASCII tree for TestResult");

    // Parse the TestResult from JSON
    let test_result: reev_lib::results::TestResult = match serde_json::from_value(test_result) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to parse TestResult: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid TestResult format").into_response();
        }
    };

    // Render as ASCII tree
    let ascii_tree = reev_runner::renderer::render_result_as_tree(&test_result);

    info!("Successfully rendered ASCII tree");
    (StatusCode::OK, [("Content-Type", "text/plain")], ascii_tree).into_response()
}

/// Parse YML to TestResult
pub async fn parse_yml_to_testresult(yml_content: String) -> impl IntoResponse {
    info!("Parsing YML to TestResult");
    info!("YML content length: {} chars", yml_content.len());
    info!(
        "YML content preview: {}",
        &yml_content[..yml_content.len().min(200)]
    );

    // Log the first few lines to understand the format
    let lines: Vec<&str> = yml_content.lines().take(5).collect();
    info!("YML first 5 lines: {:?}", lines);

    // Parse YML to TestResult object
    let test_result: reev_lib::results::TestResult = match serde_yaml::from_str(&yml_content) {
        Ok(result) => {
            info!("Successfully parsed YML to TestResult");
            result
        }
        Err(e) => {
            error!("Failed to parse YML to TestResult: {}", e);
            error!(
                "YML content that failed: {}",
                &yml_content[..yml_content.len().min(500)]
            );
            return (StatusCode::BAD_REQUEST, format!("Invalid YML format: {e}")).into_response();
        }
    };

    info!("Successfully parsed YML to TestResult");
    Json(test_result).into_response()
}

/// Get ASCII tree from current execution state (temporary fix)
pub async fn get_ascii_tree_from_state(
    State(state): State<ApiState>,
    Path((benchmark_id, agent_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting ASCII tree from execution state for benchmark: {} by agent: {}",
        benchmark_id, agent_type
    );

    let executions = state.executions.lock().await;

    // Find the most recent execution for this benchmark and agent
    let mut matching_execution = None;
    for execution in executions.values() {
        if execution.benchmark_id == benchmark_id && execution.agent == agent_type {
            match matching_execution {
                None => matching_execution = Some(execution),
                Some(current) => {
                    if execution.start_time > current.start_time {
                        matching_execution = Some(execution);
                    }
                }
            }
        }
    }

    match matching_execution {
        Some(execution) => {
            info!(
                "Found execution with status: {}",
                match execution.status {
                    ExecutionStatus::Pending => "Pending",
                    ExecutionStatus::Running => "Running",
                    ExecutionStatus::Completed => "Completed",
                    ExecutionStatus::Failed => "Failed",
                }
            );

            if !execution.trace.is_empty() {
                info!(
                    "Returning ASCII tree trace ({} chars)",
                    execution.trace.len()
                );
                (
                    StatusCode::OK,
                    [("Content-Type", "text/plain")],
                    execution.trace.clone(),
                )
                    .into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    "No trace available for this execution",
                )
                    .into_response()
            }
        }
        None => {
            info!(
                "No execution found for benchmark: {} by agent: {}",
                benchmark_id, agent_type
            );
            (StatusCode::NOT_FOUND, "No execution found").into_response()
        }
    }
}
