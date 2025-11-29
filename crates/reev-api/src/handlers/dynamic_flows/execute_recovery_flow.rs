//! Recovery Flow Execution Handler
//!
//! This module provides the handler for executing recovery flows through REST API.

// TODO: Uncomment when implementing migration
// use anyhow;
use axum::{extract::State, response::IntoResponse, Json};
// use reev_core::{ContextResolver, Executor};
// use reev_types::{ExecutionResponse, ExecutionStatus};
use serde_json::json;
// use std::sync::Arc;
use std::time::Instant;
// use tokio::task;
use tracing::{info, instrument};

use crate::types::ApiState;

/// Execute a recovery flow
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
    // TODO: Use start_time when implementing migration
    let _start_time = Instant::now();

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
    // TODO: Use these variables when implementing migration
    let _request_clone = request.clone();
    let _prompt = request.prompt.clone();
    let _wallet = request.wallet.clone();
    let _agent = request.agent.clone();

    // TODO: Convert API recovery config to orchestrator recovery config
    // let recovery_config = request
    //     .recovery_config
    //     .map(|api_config| RecoveryConfig {
    //         base_retry_delay_ms: api_config.base_retry_delay_ms.unwrap_or(1000),
    //         max_retry_delay_ms: api_config.max_retry_delay_ms.unwrap_or(10000),
    //         backoff_multiplier: api_config.backoff_multiplier.unwrap_or(2.0),
    //         max_recovery_time_ms: api_config.max_recovery_time_ms.unwrap_or(30000),
    //         enable_alternative_flows: api_config.enable_alternative_flows.unwrap_or(true),
    //         enable_user_fulfillment: api_config.enable_user_fulfillment.unwrap_or(false),
    //     })
    //     .unwrap_or_default();

    // Clone database config for use in blocking task
    // TODO: Use db_config when implementing migration
    let _db_config = _state.db.config().clone();

    // TODO: Create gateway directly using blocking task for database writer creation
    // let gateway = tokio::task::spawn_blocking(move || {
    //     let rt = tokio::runtime::Runtime::new()?;
    //     rt.block_on(async {
    //         let database_writer = reev_db::writer::DatabaseWriter::new(db_config).await?;
    //         OrchestratorGateway::with_recovery_config_with_database(
    //             recovery_config,
    //             Arc::new(database_writer),
    //         )
    //         .await
    //     })
    // })
    // .await;

    // TODO: Handle gateway creation - temporarily disabled
    // let gateway = match gateway {
    //     Ok(Ok(gateway)) => gateway,
    //     Ok(Err(e)) => {
    //         error!("Failed to create gateway: {}", e);
    //         return Json(json!({
    //             "error": format!("Gateway creation failed: {}", e),
    //             "execution_id": execution_id,
    //             "execution_mode": "recovery"
    //         }))
    //         .into_response();
    //     }
    //     Err(e) => {
    //         error!("Failed to spawn blocking task: {}", e);
    //         return Json(json!({
    //             "error": format!("Task spawn failed: {}", e),
    //             "execution_id": execution_id,
    //             "execution_mode": "recovery"
    //         }))
    //         .into_response();
    //     }
    // };

    // TODO: Execute flow in a blocking task to avoid async context issues
    // let flow_result = task::spawn_blocking(move || {
    //     let rt = match tokio::runtime::Runtime::new() {
    //         Ok(rt) => rt,
    //         Err(e) => {
    //             error!("Failed to create tokio runtime: {}", e);
    //             return Err(anyhow::anyhow!("Runtime creation failed: {e}"));
    //         }
    //     };

    //     rt.block_on(async { gateway.process_user_request(&prompt, &wallet).await })
    // })
    // .await
    // .unwrap_or_else(|e| Err(anyhow::anyhow!("Task execution failed: {e}")));

    // TODO: Handle flow execution result - temporarily return error
    // match flow_result {
    //     Ok((flow_plan, _yml_path)) => {
    //         let duration_ms = start_time.elapsed().as_millis() as u64;

    //         info!(
    //             flow_id = %flow_plan.flow_id,
    //             steps = flow_plan.steps.len(),
    //             duration_ms,
    //             "Recovery flow execution completed successfully"
    //         );

    //         Json(ExecutionResponse {
    //             execution_id,
    //             status: ExecutionStatus::Completed,
    //             duration_ms,
    //             result: Some(json!({
    //                 "flow_id": flow_plan.flow_id,
    //                 "steps_generated": flow_plan.steps.len(),
    //                 "execution_mode": "recovery",
    //                 "prompt_processed": request.prompt,
    //                 "recovery_enabled": true,
    //                 "recovery_config": request_clone.recovery_config
    //             })),
    //             error: None,
    //             logs: vec![
    //                 format!(
    //                     "Generated {} steps for recovery execution",
    //                     flow_plan.steps.len()
    //                 ),
    //                 "Recovery strategies: retry, alternative flows, user fulfillment".to_string(),
    //             ],
    //             tool_calls: vec![],
    //         })
    //         .into_response()
    //     }
    //     Err(e) => {
    //         error!(error = %e, "Failed to process recovery flow request");

    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({
    //                 "error": "Internal server error during recovery flow execution",
    //                 "execution_id": execution_id,
    //                 "details": format!("Task failed: {}", e)
    //             })),
    //         )
    //             .into_response()
    //     }
    // }

    // Temporarily return an error until recovery flow execution is implemented
    Json(json!({
        "error": "Recovery flow execution is temporarily disabled - OrchestratorGateway needs to be replaced",
        "execution_id": execution_id,
        "execution_mode": "recovery"
    }))
    .into_response()
}
