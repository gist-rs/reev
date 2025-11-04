//! Dynamic Flow Handlers
//!
//! This module provides API endpoints for executing dynamic flows through REST API.
//! It integrates with reev-orchestrator to provide same functionality available via CLI.

use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use reev_types::{ExecutionResponse, ExecutionStatus};
use serde_json::json;
use tracing::info;

use crate::types::ApiState;

/// Execute a dynamic flow (direct mode - zero file I/O)
pub async fn execute_dynamic_flow(
    State(_state): State<ApiState>,
    Json(request): Json<crate::types::DynamicFlowRequest>,
) -> impl IntoResponse {
    info!(
        prompt = %request.prompt,
        wallet = %request.wallet,
        agent = %request.agent,
        "Executing direct flow"
    );

    // Generate execution ID
    let execution_id = format!(
        "direct-{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );

    // TODO: Implement actual execution using reev-orchestrator
    // Currently returning mock response to avoid thread safety issues
    Json(ExecutionResponse {
        execution_id,
        status: ExecutionStatus::Completed,
        duration_ms: 0,
        result: None,
        error: None,
        logs: Vec::new(),
        tool_calls: Vec::new(),
    })
    .into_response()
}

/// Execute a dynamic flow with recovery
pub async fn execute_recovery_flow(
    State(_state): State<ApiState>,
    Json(request): Json<crate::types::RecoveryFlowRequest>,
) -> impl IntoResponse {
    info!(
        prompt = %request.prompt,
        wallet = %request.wallet,
        agent = ?request.agent,
        "Executing recovery flow"
    );

    // Generate execution ID
    let execution_id = format!(
        "recovery-{}",
        uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );

    // TODO: Implement actual recovery execution using reev-orchestrator
    // Currently returning mock response to avoid thread safety issues
    Json(ExecutionResponse {
        execution_id,
        status: ExecutionStatus::Completed,
        duration_ms: 0,
        result: None,
        error: None,
        logs: Vec::new(),
        tool_calls: Vec::new(),
    })
    .into_response()
}

/// Get recovery metrics
pub async fn get_recovery_metrics(State(_state): State<ApiState>) -> impl IntoResponse {
    info!("Getting recovery metrics");

    // TODO: Implement actual recovery metrics collection
    let metrics = json!({
        "total_flows": 0,
        "successful_flows": 0,
        "failed_flows": 0,
        "recovered_flows": 0,
        "average_recovery_time_ms": 0,
        "success_rate": 0.0
    });

    Json(metrics).into_response()
}
