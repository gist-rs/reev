//! Transaction log handlers
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs_demo(
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Generating demo transaction logs with visualization");

    // Check format parameter: yaml or plain (yaml is default)
    let format_param = params
        .get("format")
        .map_or("yaml".to_string(), |v| v.clone());
    let use_yaml = format_param == "yaml";

    // Check show_cu parameter: true or false (false is default)
    let show_cu_param = params
        .get("show_cu")
        .map_or("false".to_string(), |v| v.clone());
    let show_cu = show_cu_param == "true";

    let demo_logs = if use_yaml {
        // Generate YAML format demo
        let mock_logs = vec![
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
            "Program log: CreateIdempotent".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
            "Program log: Instruction: GetAccountDataSize".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
            "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        ];

        match crate::services::generate_transaction_logs_yaml(&mock_logs, show_cu) {
            Ok(yaml_logs) => yaml_logs,
            Err(e) => {
                error!("Failed to generate YAML logs: {}", e);
                format!("Error generating YAML tree: {e}")
            }
        }
    } else {
        // Generate plain format demo
        let mock_logs = vec![
            "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]".to_string(),
            "Program log: CreateIdempotent".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".to_string(),
            "Program log: Instruction: GetAccountDataSize".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1569 of 997595 compute units".to_string(),
            "Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=".to_string(),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".to_string(),
        ];

        let mut plain = String::new();
        plain.push_str("Step 1:\n");
        for log in &mock_logs {
            plain.push_str(&format!("  {log}\n"));
        }
        plain
    };

    (
        StatusCode::OK,
        Json(json!({
            "benchmark_id": "demo-jupiter-swap",
            "transaction_logs": demo_logs,
            "format": if use_yaml { "yaml" } else { "plain" },
            "message": "Demo transaction logs generated successfully"
        })),
    )
        .into_response()
}

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("Getting transaction logs for benchmark: {}", benchmark_id);

    // First check for active executions (like execution trace does)
    let executions = state.executions.lock().await;
    let mut found_execution = None;

    for (execution_id, execution) in executions.iter() {
        if execution.benchmark_id == benchmark_id {
            found_execution = Some((execution_id.clone(), execution.clone()));
            break;
        }
    }

    drop(executions); // Release lock before processing

    if let Some((_execution_id, execution)) = found_execution {
        let is_running = execution.status == ExecutionStatus::Running
            || execution.status == ExecutionStatus::Pending;

        info!(
            "Found execution for benchmark: {} (status: {:?})",
            benchmark_id, execution.status
        );

        // Handle running executions like execution trace - return raw trace or loading message
        if is_running {
            // Check format parameter: yaml or plain (yaml is default)
            let format_param = params
                .get("format")
                .map_or("yaml".to_string(), |v| v.clone());

            // Check show_cu parameter: true or false (false is default)
            let show_cu_param = params
                .get("show_cu")
                .map_or("false".to_string(), |v| v.clone());
            let show_cu = show_cu_param == "true";

            let transaction_logs = if execution.trace.trim().is_empty() {
                "🔄 Loading transaction logs...\n\n⏳ Execution in progress - please wait"
                    .to_string()
            } else {
                // Try to parse and extract transaction logs from running execution
                match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&execution.trace) {
                    Ok(trace) => {
                        // Create a TestResult from the trace to use existing extraction logic
                        let test_result = reev_lib::results::TestResult::new(
                            &reev_lib::benchmark::TestCase {
                                id: benchmark_id.clone(),
                                description: format!("Transaction logs for {benchmark_id}"),
                                tags: vec!["api".to_string()],
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
                            1.0, // Default score for running execution
                            trace,
                        );

                        // Use appropriate transaction log extraction
                        let logs = if format_param == "yaml" {
                            match crate::services::generate_transaction_logs_yaml(
                                &test_result
                                    .trace
                                    .steps
                                    .iter()
                                    .flat_map(|step| &step.observation.last_transaction_logs)
                                    .cloned()
                                    .collect::<Vec<_>>(),
                                show_cu,
                            ) {
                                Ok(yaml_logs) => yaml_logs,
                                Err(e) => {
                                    error!("Failed to generate YAML logs from execution: {}", e);
                                    format!("Error generating YAML tree: {e}")
                                }
                            }
                        } else {
                            crate::services::generate_transaction_logs(&test_result)
                        };

                        // Add status indicator for running executions
                        if !logs.trim().is_empty() {
                            format!("{logs}\n\n⏳ Execution in progress - logs may be incomplete")
                        } else {
                            logs
                        }
                    }
                    Err(_) => {
                        // Failed to parse - show raw trace with processing message
                        format!("🔄 Processing transaction logs...\n\n⚠️ Unable to parse execution trace - still running\n\nRaw trace preview:\n{}",
                            &execution.trace[..execution.trace.len().min(500)])
                    }
                }
            };

            info!(
                "Returning transaction logs from running execution for benchmark: {} ({} chars, format: {}, show_cu: {})",
                benchmark_id,
                transaction_logs.len(),
                format_param,
                show_cu
            );

            return (
                StatusCode::OK,
                Json(json!({
                    "benchmark_id": benchmark_id,
                    "transaction_logs": transaction_logs,
                    "format": format_param,
                    "show_cu": show_cu,
                    "message": "Transaction logs from active execution (may be incomplete)",
                    "is_running": true
                })),
            )
                .into_response();
        }

        // For completed executions with trace data, use the in-memory trace
        if !execution.trace.is_empty() {
            if let Ok(trace) =
                serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&execution.trace)
            {
                // Create a TestResult from the trace to use existing extraction logic
                let test_result = reev_lib::results::TestResult::new(
                    &reev_lib::benchmark::TestCase {
                        id: benchmark_id.clone(),
                        description: format!("Transaction logs for {benchmark_id}"),
                        tags: vec!["api".to_string()],
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
                    1.0,
                    trace,
                );

                // Check format parameter: yaml or plain (yaml is default)
                let format_param = params
                    .get("format")
                    .map_or("yaml".to_string(), |v| v.clone());
                let use_yaml = format_param == "yaml";

                // Check show_cu parameter: true or false (false is default)
                let show_cu_param = params
                    .get("show_cu")
                    .map_or("false".to_string(), |v| v.clone());
                let show_cu = show_cu_param == "true";

                // Use appropriate transaction log extraction
                let transaction_logs = if use_yaml {
                    match crate::services::generate_transaction_logs_yaml(
                        &test_result
                            .trace
                            .steps
                            .iter()
                            .flat_map(|step| &step.observation.last_transaction_logs)
                            .cloned()
                            .collect::<Vec<_>>(),
                        show_cu,
                    ) {
                        Ok(yaml_logs) => yaml_logs,
                        Err(e) => {
                            error!("Failed to generate YAML logs from execution: {}", e);
                            format!("Error generating YAML tree: {e}")
                        }
                    }
                } else {
                    crate::services::generate_transaction_logs(&test_result)
                };

                info!(
                    "Extracted transaction logs from completed execution for benchmark: {} ({} chars, format: {}, show_cu: {})",
                    benchmark_id,
                    transaction_logs.len(),
                    if use_yaml { "yaml" } else { "plain" },
                    show_cu
                );

                return (
                    StatusCode::OK,
                    Json(json!({
                        "benchmark_id": benchmark_id,
                        "transaction_logs": transaction_logs,
                        "format": if use_yaml { "yaml" } else { "plain" },
                        "show_cu": show_cu,
                        "message": "Transaction logs from completed execution",
                        "is_running": false
                    })),
                )
                    .into_response();
            }
        }
    } else {
        info!(
            "No execution found in memory for benchmark: {}",
            benchmark_id
        );
    }

    // Get the most recent session for this benchmark
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

                // Get the session log which contains the execution trace
                info!(
                    "DEBUG: Attempting to get session log for session: {}",
                    session.session_id
                );

                // Use same pattern as execution log - direct database access
                match state.db.get_session_log(&session.session_id).await {
                    Ok(Some(log_content)) => {
                        info!("DEBUG: Retrieved session log content (length: {} chars) for session: {}", log_content.len(), session.session_id);

                        // Log first 100 chars of content for debugging
                        let log_preview = if log_content.len() > 100 {
                            format!("{}...", &log_content[..100])
                        } else {
                            log_content.clone()
                        };
                        info!("DEBUG: Session log preview: {}", log_preview);

                        // Try to parse as ExecutionTrace and extract transaction logs
                        match serde_json::from_str::<reev_lib::trace::ExecutionTrace>(&log_content)
                        {
                            Ok(trace) => {
                                info!("DEBUG: Successfully parsed session log as ExecutionTrace");
                                // Create a TestResult from the trace to use existing extraction logic
                                let test_result = reev_lib::results::TestResult::new(
                                    &reev_lib::benchmark::TestCase {
                                        id: benchmark_id.clone(),
                                        description: format!("Transaction logs for {benchmark_id}"),
                                        tags: vec!["api".to_string()],
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

                                // Check format parameter: yaml or plain (yaml is default)
                                let format_param = params
                                    .get("format")
                                    .map_or("yaml".to_string(), |v| v.clone());
                                let use_yaml = format_param == "yaml";

                                // Check show_cu parameter: true or false (false is default)
                                let show_cu_param = params
                                    .get("show_cu")
                                    .map_or("false".to_string(), |v| v.clone());
                                let show_cu = show_cu_param == "true";

                                // Use appropriate transaction log extraction
                                let transaction_logs = if use_yaml {
                                    match crate::services::generate_transaction_logs_yaml(
                                        &test_result
                                            .trace
                                            .steps
                                            .iter()
                                            .flat_map(|step| {
                                                &step.observation.last_transaction_logs
                                            })
                                            .cloned()
                                            .collect::<Vec<_>>(),
                                        show_cu,
                                    ) {
                                        Ok(yaml_logs) => yaml_logs,
                                        Err(e) => {
                                            error!("Failed to generate YAML logs: {}", e);
                                            format!("Error generating YAML tree: {e}")
                                        }
                                    }
                                } else {
                                    crate::services::generate_transaction_logs(&test_result)
                                };

                                info!(
                                    "Extracted transaction logs for benchmark: {} ({} chars, format: {}, show_cu: {})",
                                    benchmark_id,
                                    transaction_logs.len(),
                                    if use_yaml { "yaml" } else { "plain" },
                                    show_cu
                                );

                                if transaction_logs.trim().is_empty() {
                                    return (
                                        StatusCode::OK,
                                        Json(json!({
                                            "benchmark_id": benchmark_id,
                                            "transaction_logs": "",
                                            "format": format_param,
                                            "show_cu": show_cu,
                                            "message": "No transaction logs available",
                                            "is_running": false
                                        })),
                                    )
                                        .into_response();
                                }

                                (
                                    StatusCode::OK,
                                    Json(json!({
                                        "benchmark_id": benchmark_id,
                                        "format": format_param,
                                        "show_cu": show_cu,
                                        "message": "Transaction logs extracted successfully",
                                        "transaction_logs": transaction_logs,
                                        "is_running": false
                                    })),
                                )
                                    .into_response()
                            }
                            Err(e) => {
                                warn!("Failed to parse log as ExecutionTrace: {}", e);
                                info!("DEBUG: Returning fallback response for invalid ExecutionTrace data");

                                info!("DEBUG: Successfully created fallback JSON response");
                                let response = (
                                    StatusCode::OK,
                                    Json(json!({
                                        "benchmark_id": benchmark_id,
                                        "transaction_logs": "",
                                        "format": if params.get("format").is_some_and(|f| f == "tree") { "tree" } else { "plain" },
                                        "show_cu": params.get("show_cu").unwrap_or(&"false".to_string()) == "true",
                                        "message": "No valid ExecutionTrace data found",
                                        "is_running": false
                                    })),
                                );
                                info!("DEBUG: Returning fallback response");
                                response.into_response()
                            }
                        }
                    }
                    Ok(None) => {
                        warn!("No session log content found");
                        info!("DEBUG: Returning empty response for missing session log");
                        let response = (
                            StatusCode::OK,
                            Json(json!({
                                "benchmark_id": benchmark_id,
                                "transaction_logs": "",
                                "format": if params.get("format").is_some_and(|f| f == "tree") { "tree" } else { "plain" },
                                "show_cu": params.get("show_cu").unwrap_or(&"false".to_string()) == "true",
                                "message": "No session log content found",
                                "is_running": false
                            })),
                        );
                        info!("DEBUG: Returning empty response");
                        response.into_response()
                    }
                    Err(e) => {
                        warn!("Failed to get session log: {}", e);
                        info!("DEBUG: Returning error response for failed session log retrieval");
                        info!("DEBUG: Successfully created error JSON response");
                        let response = (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "Failed to retrieve session log"
                            })),
                        );
                        info!("DEBUG: Returning error response");
                        response.into_response()
                    }
                }
            } else {
                info!("No sessions found for benchmark: {}", benchmark_id);
                (
                    StatusCode::OK,
                    Json(json!({
                        "benchmark_id": benchmark_id,
                        "transaction_logs": "",
                        "format": if params.get("format").is_some_and(|f| f == "tree") { "tree" } else { "plain" },
                        "show_cu": params.get("show_cu").unwrap_or(&"false".to_string()) == "true",
                        "message": format!("No sessions found for benchmark: {}", benchmark_id),
                        "is_running": false
                    })),
                ).into_response()
            }
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error"
                })),
            )
                .into_response()
        }
    }
}
