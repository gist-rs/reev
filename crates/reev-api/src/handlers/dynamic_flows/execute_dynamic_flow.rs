//! Dynamic Flow Execution Handler
//!
//! This module provides the main handler for executing dynamic flows through REST API.

use anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use reev_db::writer::DatabaseWriterTrait;
use reev_orchestrator::OrchestratorGateway;
use reev_types::{ExecutionResponse, ExecutionStatus};
use serde_json::json;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, instrument};

use super::execute_flow_plan_with_ping_pong::execute_flow_plan_with_ping_pong;
use crate::types::ApiState;

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
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            error!("Failed to create tokio runtime: {}", e);
            anyhow::anyhow!("Runtime creation failed: {e}")
        })?;

        rt.block_on(async {
            let gateway = OrchestratorGateway::new().await.map_err(|e| {
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

            info!(
                flow_id = %flow_plan.flow_id,
                steps = flow_plan.steps.len(),
                execution_mode = execution_mode,
                duration_ms,
                "Dynamic flow execution completed successfully"
            );

            // Execute flow plan with ping-pong coordination and capture actual tool calls
            let real_tool_calls = execute_flow_plan_with_ping_pong(&flow_plan, &agent_type).await;

            // Store session log with tool calls for API visualization
            let session_log_content = json!({
                "session_id": &flow_plan.flow_id,
                "benchmark_id": "dynamic-flow",
                "agent_type": &agent_type,
                "tool_calls": &real_tool_calls,
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

            // Store in database for flow visualization (using flow_id)
            if let Err(e) = state
                .db
                .store_session_log(&flow_plan.flow_id, &session_log_content)
                .await
            {
                error!("Failed to store dynamic flow session log: {}", e);
            }

            // Also store with execution_id for easier lookup by users
            let execution_log_content = json!({
                "session_id": &execution_id,
                "benchmark_id": "dynamic-flow",
                "agent_type": &agent_type,
                "tool_calls": &real_tool_calls,
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

            // Store with execution_id for direct lookup
            if let Err(e) = state
                .db
                .store_session_log(&execution_id, &execution_log_content)
                .await
            {
                error!("Failed to store execution session log: {}", e);
            }

            // Note: OTEL data is handled by agent execution directly
            // No additional conversion needed for flow visualization

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
