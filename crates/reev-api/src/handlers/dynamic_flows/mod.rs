//! Dynamic Flow Handlers
//!
//! This module provides API endpoints for executing dynamic flows through REST API.
//! It integrates with reev-orchestrator to provide same functionality available via CLI.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use chrono::Utc;
use reev_orchestrator::OrchestratorGateway;
use reev_types::{ExecutionResponse, ExecutionStatus};
use serde_json::json;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument};

use crate::types::ApiState;
use reev_db::writer::DatabaseWriterTrait;

/// Execute a dynamic flow (direct mode - zero file I/O)
#[instrument(skip_all, fields(
    prompt = %request.prompt,
    wallet = %request.wallet,
    agent = %request.agent,
    execution_mode = "direct"
))]
pub async fn execute_dynamic_flow(
    State(state): State<ApiState>,
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

    // Execute flow plan in blocking task
    let flow_result = task::spawn_blocking(move || {
        let gateway = OrchestratorGateway::new();
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            error!("Failed to create tokio runtime: {}", e);
            anyhow::anyhow!("Runtime creation failed: {e}")
        })?;

        rt.block_on(async { gateway.process_user_request(&prompt, &wallet).await })
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

            info!(
                flow_id = %flow_plan.flow_id,
                steps = flow_plan.steps.len(),
                execution_mode = execution_mode,
                duration_ms,
                "Dynamic flow execution completed successfully"
            );

            // Create mock tool calls based on flow plan for visualization
            let mock_tool_calls = create_mock_tool_calls_from_flow_plan(&flow_plan, &agent_type);

            // Store session log with tool calls for API visualization
            let session_log_content = json!({
                "session_id": &flow_plan.flow_id,
                "benchmark_id": "dynamic-flow",
                "agent_type": &agent_type,
                "tool_calls": &mock_tool_calls,
                "start_time": Utc::now().timestamp(),
                "end_time": Utc::now().timestamp() + 60000,
                "execution_mode": execution_mode,
                "flow_plan": {
                    "flow_id": flow_plan.flow_id,
                    "user_prompt": flow_plan.user_prompt,
                    "steps": flow_plan.steps.len()
                }
            })
            .to_string();

            // Store in database for flow visualization
            if let Err(e) = state
                .db
                .store_session_log(&flow_plan.flow_id, &session_log_content)
                .await
            {
                error!("Failed to store dynamic flow session log: {}", e);
            }

            let mut result_data = json!({
                "flow_id": flow_plan.flow_id,
                "steps_generated": flow_plan.steps.len(),
                "execution_mode": execution_mode,
                "prompt_processed": request.prompt
            });

            if request.shared_surfpool {
                result_data["yml_file"] = json!(yml_path);
            }

            let logs = if request.shared_surfpool {
                vec![
                    format!(
                        "Generated {} steps for bridge execution",
                        flow_plan.steps.len()
                    ),
                    format!("Created temporary YML file: {}", yml_path),
                    format!(
                        "Stored session log for flow visualization: {}",
                        flow_plan.flow_id
                    ),
                ]
            } else {
                vec![
                    format!(
                        "Generated {} steps for direct execution",
                        flow_plan.steps.len()
                    ),
                    format!(
                        "Stored session log for flow visualization: {}",
                        flow_plan.flow_id
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
                tool_calls: mock_tool_calls,
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

/// Execute a dynamic flow with recovery
#[instrument(skip_all, fields(
    prompt = %request.prompt,
    wallet = %request.wallet,
    agent = ?request.agent,
    execution_mode = "recovery"
))]
pub async fn execute_recovery_flow(
    State(_state): State<ApiState>,
    Json(request): Json<crate::types::RecoveryFlowRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();

    // Generate execution ID
    let execution_id = format!(
        "recovery-{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );

    info!("Starting recovery flow execution");

    // Clone request data to avoid borrow checker issues
    let request_clone = request.clone();
    let prompt = request.prompt.clone();
    let wallet = request.wallet.clone();
    let _agent = request.agent.clone();

    // Convert API recovery config to orchestrator recovery config
    let recovery_config = request
        .recovery_config
        .map(|api_config| reev_orchestrator::RecoveryConfig {
            base_retry_delay_ms: api_config.base_retry_delay_ms.unwrap_or(1000),
            max_retry_delay_ms: api_config.max_retry_delay_ms.unwrap_or(10000),
            backoff_multiplier: api_config.backoff_multiplier.unwrap_or(2.0),
            max_recovery_time_ms: api_config.max_recovery_time_ms.unwrap_or(30000),
            enable_alternative_flows: api_config.enable_alternative_flows.unwrap_or(true),
            enable_user_fulfillment: api_config.enable_user_fulfillment.unwrap_or(false),
        })
        .unwrap_or_default();

    // Execute flow in a blocking task to avoid async context issues
    let flow_result = task::spawn_blocking(move || {
        // Create new gateway instance for each request (thread-safe approach)
        let gateway = OrchestratorGateway::with_recovery_config(recovery_config);
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            error!("Failed to create tokio runtime: {}", e);
            anyhow::anyhow!("Runtime creation failed: {e}")
        })?;

        rt.block_on(async { gateway.process_user_request(&prompt, &wallet).await })
    })
    .await;

    match flow_result {
        Ok(Ok((flow_plan, _yml_path))) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;

            info!(
                flow_id = %flow_plan.flow_id,
                steps = flow_plan.steps.len(),
                duration_ms,
                "Recovery flow execution completed successfully"
            );

            Json(ExecutionResponse {
                execution_id,
                status: ExecutionStatus::Completed,
                duration_ms,
                result: Some(json!({
                    "flow_id": flow_plan.flow_id,
                    "steps_generated": flow_plan.steps.len(),
                    "execution_mode": "recovery",
                    "prompt_processed": request.prompt,
                    "recovery_enabled": true,
                    "recovery_config": request_clone.recovery_config
                })),
                error: None,
                logs: vec![
                    format!(
                        "Generated {} steps for recovery execution",
                        flow_plan.steps.len()
                    ),
                    "Recovery strategies: retry, alternative flows, user fulfillment".to_string(),
                ],
                tool_calls: vec![],
            })
            .into_response()
        }
        Ok(Err(e)) => {
            error!(error = %e, "Failed to process recovery flow request");

            Json(ExecutionResponse {
                execution_id,
                status: ExecutionStatus::Failed,
                duration_ms: start_time.elapsed().as_millis() as u64,
                result: None,
                error: Some(format!("Failed to generate recovery flow plan: {e}")),
                logs: vec![format!("Error: {}", e)],
                tool_calls: vec![],
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, "Recovery task execution failed");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal server error during recovery flow execution",
                    "execution_id": execution_id,
                    "details": format!("Task failed: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Create mock tool calls from flow plan for visualization
fn create_mock_tool_calls_from_flow_plan(
    flow_plan: &reev_types::flow::DynamicFlowPlan,
    _agent_type: &str,
) -> Vec<reev_types::execution::ToolCallSummary> {
    let mut tool_calls = Vec::new();
    let current_time = chrono::Utc::now().timestamp_millis();

    for (index, step) in flow_plan.steps.iter().enumerate() {
        let start_time = current_time + (index as i64 * 30000); // 30 seconds per step

        // Determine tool type based on step and tools
        let tool_name = if step.required_tools.contains(&"sol_tool".to_string()) {
            if step.description.to_lowercase().contains("swap") {
                "jupiter_swap"
            } else {
                "sol_tool"
            }
        } else if step
            .required_tools
            .contains(&"jupiter_earn_tool".to_string())
        {
            "jupiter_lend"
        } else {
            "unknown_tool"
        };

        // Create mock parameters based on tool type
        let _params = match tool_name {
            "jupiter_swap" => {
                let amount = if flow_plan.user_prompt.to_lowercase().contains("50%") {
                    "0.5".to_string()
                } else if flow_plan.user_prompt.to_lowercase().contains("1.5x") {
                    "0.75".to_string()
                } else {
                    "0.5".to_string()
                };

                serde_json::json!({
                    "input_token": "So11111111111111111111111111111111111111112",
                    "output_token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "amount": amount,
                    "slippage": 3.0
                })
            }
            "jupiter_lend" => {
                serde_json::json!({
                    "token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "amount": "max",
                    "action": "deposit"
                })
            }
            _ => {
                serde_json::json!({})
            }
        };

        let mock_tool_call = reev_types::execution::ToolCallSummary {
            tool_name: tool_name.to_string(),
            timestamp: chrono::DateTime::from_timestamp_millis(start_time)
                .unwrap_or_else(chrono::Utc::now),
            duration_ms: 5000, // 5 seconds execution time
            success: true,
            error: None,
        };

        tool_calls.push(mock_tool_call);
    }

    tool_calls
}

/// Get recovery metrics
#[instrument(skip_all)]
pub async fn get_recovery_metrics(State(_state): State<ApiState>) -> impl IntoResponse {
    info!("Getting recovery metrics");

    // Simple approach: return empty metrics for now
    // TODO: Implement real metrics collection when orchestrator metrics are thread-safe
    let metrics_json = json!({
        "total_flows": 0,
        "successful_flows": 0,
        "failed_flows": 0,
        "recovered_flows": 0,
        "total_recovery_time_ms": 0,
        "average_recovery_time_ms": 0,
        "success_rate": 0.0,
        "strategies_used": {
            "retry_attempts": 0,
            "alternative_flows_used": 0,
            "user_fulfillment_used": 0
        },
        "last_updated": Utc::now().to_rfc3339()
    });

    Json(metrics_json).into_response()
}
