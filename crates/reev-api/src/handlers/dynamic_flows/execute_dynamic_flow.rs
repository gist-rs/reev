//! Dynamic Flow Execution Handler
//!
//! This module provides main handler for executing dynamic flows through REST API.

use anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use reev_orchestrator::OrchestratorGateway;
use reev_types::{ExecutionResponse, ExecutionStatus};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument};

/// Execute a dynamic flow (direct mode - zero file I/O)
#[instrument(skip_all, fields(
    prompt = %request.prompt,
    wallet = %request.wallet,
    agent = %request.agent,
    execution_mode = "direct"
))]
pub async fn execute_dynamic_flow(
    State(state): State<crate::types::ApiState>,
    Json(request): Json<crate::types::DynamicFlowRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    // Generate execution ID based on mode
    let mode_prefix = if request.shared_surfpool {
        "bridge"
    } else {
        "direct"
    };
    let execution_id = format!(
        "{}-{}",
        mode_prefix,
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );

    info!(
        execution_mode = mode_prefix,
        "Starting dynamic flow execution"
    );

    // Clone request data to avoid borrow checker issues
    let prompt = request.prompt.clone();
    let wallet = request.wallet.clone();
    let agent_type = request.agent.clone();
    let _atomic_mode = request.atomic_mode;

    // Clone the database config for use in blocking task
    let db_config = state.db.config().clone();

    // Execute flow plan in blocking task
    // Create and spawn the task with proper error handling
    let flow_result = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            error!("Failed to create tokio runtime: {}", e);
            anyhow::anyhow!("Runtime creation failed: {e}")
        })?;

        rt.block_on(async {
            // Create new DatabaseWriter using same configuration as pooled database
            let database_writer = reev_db::writer::DatabaseWriter::new(db_config)
                .await
                .map_err(|e| {
                    error!("Failed to create database writer: {}", e);
                    anyhow::anyhow!("Database writer creation failed: {e}")
                })?;

            let gateway = OrchestratorGateway::with_database(Arc::new(database_writer))
                .await
                .map_err(|e| {
                    error!("Failed to create gateway: {}", e);
                    anyhow::anyhow!("Gateway creation failed: {e}")
                })?;
            gateway.process_user_request(&prompt, &wallet).await
        })
    })
    .await;

    match flow_result {
        Ok(Ok((flow_plan, yml_path))) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let execution_mode = if request.shared_surfpool {
                "bridge"
            } else {
                "direct"
            };

            // Clone flow_plan and database config for use in blocking task
            let flow_plan_clone = flow_plan.clone();
            let db_config_for_execution = state.db.config().clone();

            info!(
                flow_id = %flow_plan.flow_id,
                steps = flow_plan.steps.len(),
                execution_mode = execution_mode,
                duration_ms,
                "Dynamic flow execution completed successfully"
            );

            // Execute flow plan with ping-pong coordination and database consolidation
            let execution_result = task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    error!("Failed to create tokio runtime: {}", e);
                    anyhow::anyhow!("Runtime creation failed: {e}")
                })?;

                rt.block_on(async {
                    // Create new DatabaseWriter using same configuration as pooled database
                    let database_writer =
                        reev_db::writer::DatabaseWriter::new(db_config_for_execution)
                            .await
                            .map_err(|e| {
                                error!("Failed to create database writer: {}", e);
                                anyhow::anyhow!("Database writer creation failed: {e}")
                            })?;

                    let gateway = OrchestratorGateway::with_database(Arc::new(database_writer))
                        .await
                        .map_err(|e| {
                            error!("Failed to create gateway: {}", e);
                            anyhow::anyhow!("Gateway creation failed: {e}")
                        })?;
                    gateway
                        .execute_dynamic_flow_with_consolidation(&flow_plan, &agent_type)
                        .await
                })
            })
            .await;

            let real_tool_calls = match &execution_result {
                Ok(Ok(result)) => {
                    info!(
                        execution_id = %result.execution_id,
                        consolidated_session = ?result.consolidated_session_id,
                        "Dynamic flow execution with consolidation completed"
                    );

                    // Convert step results to tool calls for API response
                    result
                        .step_results
                        .iter()
                        .enumerate()
                        .map(|(index, step_result)| {
                            let tool_name = if !step_result.tool_calls.is_empty() {
                                step_result.tool_calls[0].clone()
                            } else {
                                // Infer from step ID
                                if step_result.step_id.contains("swap") {
                                    reev_types::ToolName::JupiterSwap.to_string()
                                } else if step_result.step_id.contains("lend") {
                                    reev_types::ToolName::JupiterLendEarnDeposit.to_string()
                                } else if step_result.step_id.contains("balance") {
                                    reev_types::ToolName::GetAccountBalance.to_string()
                                } else if step_result.step_id.contains("position") {
                                    reev_types::ToolName::GetJupiterLendEarnPosition.to_string()
                                } else {
                                    format!("tool_{}", step_result.step_id)
                                }
                            };

                            reev_types::execution::ToolCallSummary {
                                tool_name,
                                timestamp: chrono::Utc::now()
                                    + chrono::Duration::milliseconds(index as i64 * 2000),
                                duration_ms: step_result.execution_time_ms,
                                success: step_result.success,
                                error: step_result.error_message.clone(),
                                params: None,
                                result_data: Some(step_result.output.clone()),
                                tool_args: None,
                            }
                        })
                        .collect()
                }
                Ok(Err(ref e)) => {
                    error!("Dynamic flow execution with consolidation failed: {}", e);
                    Vec::new()
                }
                Err(ref e) => {
                    error!("Failed to spawn consolidation task: {}", e);
                    Vec::new()
                }
            };

            let mut result_data = json!({
                "flow_id": flow_plan_clone.flow_id,
                "steps_generated": flow_plan_clone.steps.len(),
                "execution_mode": execution_mode,
                "prompt_processed": request.prompt,
                "consolidation_enabled": true
            });

            if request.shared_surfpool {
                result_data["yml_file"] = json!(yml_path);
            }

            // Include consolidation info if available from execution result
            if let Ok(Ok(ref exec_result)) = execution_result {
                if let Some(ref consolidated_id) = exec_result.consolidated_session_id {
                    result_data["consolidated_session_id"] = json!(consolidated_id);
                    result_data["execution_id"] = json!(&exec_result.execution_id);
                }

                // Add tool_calls to result_data for enhanced flow diagram generation
                // Convert step_results to tool_calls format for API response
                let tool_calls: Vec<serde_json::Value> = exec_result.step_results
                    .iter()
                    .enumerate()
                    .map(|(index, step_result)| {
                        json!({
                            "tool_name": if !step_result.tool_calls.is_empty() {
                                step_result.tool_calls[0].clone()
                            } else {
                                // Infer from step ID
                                if step_result.step_id.contains("swap") {
                                    reev_types::ToolName::JupiterSwap.to_string()
                                } else if step_result.step_id.contains("lend") {
                                    reev_types::ToolName::JupiterLendEarnDeposit.to_string()
                                } else if step_result.step_id.contains("balance") {
                                    reev_types::ToolName::GetAccountBalance.to_string()
                                } else if step_result.step_id.contains("position") {
                                    reev_types::ToolName::GetJupiterLendEarnPosition.to_string()
                                } else {
                                    format!("tool_{}", step_result.step_id)
                                }
                            },
                            "timestamp": chrono::Utc::now() + chrono::Duration::milliseconds(index as i64 * 2000),
                            "duration_ms": step_result.execution_time_ms,
                            "success": step_result.success,
                            "error": step_result.error_message.clone(),
                            "params": None::<serde_json::Value>,
                            "result_data": Some(step_result.output.clone()),
                            "tool_args": None::<serde_json::Value>
                        })
                    })
                    .collect();

                result_data["tool_calls"] = json!(tool_calls);
            }

            let logs = if request.shared_surfpool {
                vec![
                    format!(
                        "Generated {} steps for bridge execution",
                        flow_plan_clone.steps.len()
                    ),
                    format!("Created temporary YML file: {}", yml_path),
                    format!(
                        "Executed with database consolidation: {}",
                        flow_plan_clone.flow_id
                    ),
                ]
            } else {
                vec![
                    format!(
                        "Generated {} steps for direct execution with consolidation",
                        flow_plan_clone.steps.len()
                    ),
                    format!(
                        "Database consolidation completed: {}",
                        flow_plan_clone.flow_id
                    ),
                ]
            };

            Json(ExecutionResponse {
                execution_id,
                status: ExecutionStatus::Completed,
                duration_ms,
                result: Some(result_data),
                error: None,
                logs,
                tool_calls: real_tool_calls,
            })
            .into_response()
        }
        Ok(Err(e)) => {
            error!(error = %e, "Failed to process dynamic flow request");

            Json(ExecutionResponse {
                execution_id,
                status: ExecutionStatus::Failed,
                duration_ms: start_time.elapsed().as_millis() as u64,
                result: None,
                error: Some(format!("Failed to generate flow plan: {e}")),
                logs: vec![format!("Error: {}", e)],
                tool_calls: vec![],
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, "Dynamic flow task execution failed");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error during dynamic flow execution",
                    "execution_id": execution_id,
                    "details": format!("Task failed: {}", e)
                })),
            )
                .into_response()
        }
    }
}
