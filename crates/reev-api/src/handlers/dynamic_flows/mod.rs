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

/// Execute a dynamic flow (direct mode - zero file I/O)
#[instrument(skip_all, fields(
    prompt = %request.prompt,
    wallet = %request.wallet,
    agent = %request.agent,
    execution_mode = "direct"
))]
pub async fn execute_dynamic_flow(
    State(_state): State<ApiState>,
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
    let _agent = request.agent.clone();

    if request.shared_surfpool {
        // Bridge mode - use temporary YML file execution
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

                info!(
                    flow_id = %flow_plan.flow_id,
                    steps = flow_plan.steps.len(),
                    yml_path = %yml_path,
                    duration_ms,
                    "Bridge flow execution completed successfully"
                );

                Json(ExecutionResponse {
                    execution_id,
                    status: ExecutionStatus::Completed,
                    duration_ms,
                    result: Some(json!({
                        "flow_id": flow_plan.flow_id,
                        "steps_generated": flow_plan.steps.len(),
                        "execution_mode": "bridge",
                        "yml_file": yml_path,
                        "prompt_processed": request.prompt
                    })),
                    error: None,
                    logs: vec![
                        format!(
                            "Generated {} steps for bridge execution",
                            flow_plan.steps.len()
                        ),
                        format!("Created temporary YML file: {}", yml_path),
                    ],
                    tool_calls: vec![],
                })
                .into_response()
            }
            Ok(Err(e)) => {
                error!(error = %e, "Failed to process bridge flow request");

                Json(ExecutionResponse {
                    execution_id,
                    status: ExecutionStatus::Failed,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    result: None,
                    error: Some(format!("Failed to generate bridge flow plan: {e}")),
                    logs: vec![format!("Error: {}", e)],
                    tool_calls: vec![],
                })
                .into_response()
            }
            Err(e) => {
                error!(error = %e, "Bridge task execution failed");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error during bridge flow execution",
                        "execution_id": execution_id,
                        "details": format!("Task failed: {}", e)
                    })),
                )
                    .into_response()
            }
        }
    } else {
        // Direct mode - generate flow plan only
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
            Ok(Ok((flow_plan, _yml_path))) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;

                info!(
                    flow_id = %flow_plan.flow_id,
                    steps = flow_plan.steps.len(),
                    duration_ms,
                    "Direct flow execution completed successfully"
                );

                Json(ExecutionResponse {
                    execution_id,
                    status: ExecutionStatus::Completed,
                    duration_ms,
                    result: Some(json!({
                        "flow_id": flow_plan.flow_id,
                        "steps_generated": flow_plan.steps.len(),
                        "execution_mode": "direct",
                        "prompt_processed": request.prompt
                    })),
                    error: None,
                    logs: vec![format!(
                        "Generated {} steps for direct execution",
                        flow_plan.steps.len()
                    )],
                    tool_calls: vec![],
                })
                .into_response()
            }
            Ok(Err(e)) => {
                error!(error = %e, "Failed to process direct flow request");

                Json(ExecutionResponse {
                    execution_id,
                    status: ExecutionStatus::Failed,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    result: None,
                    error: Some(format!("Failed to generate direct flow plan: {e}")),
                    logs: vec![format!("Error: {}", e)],
                    tool_calls: vec![],
                })
                .into_response()
            }
            Err(e) => {
                error!(error = %e, "Direct task execution failed");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Internal server error during direct flow execution",
                        "execution_id": execution_id,
                        "details": format!("Task failed: {}", e)
                    })),
                )
                    .into_response()
            }
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
