//! Flow Plan Execution with Ping-Pong Handler
//!
//! This module provides the handler for executing flow plans using ping-pong coordination.

// use reev_core::{ContextResolver, Executor}; // TODO: Uncomment when implementing migration
use reev_types::execution::ToolCallSummary;
use std::sync::Arc;
use tracing::info;

// use super::extract_transaction_details::extract_transaction_details; // TODO: Uncomment when implementing migration

/// Execute a flow plan using ping-pong coordination
#[allow(dead_code)]
pub async fn execute_flow_plan_with_ping_pong(
    flow_plan: &reev_types::flow::DynamicFlowPlan,
    agent_type: &str,
    // TODO: Use database when implementing migration
    _database: Option<Arc<reev_db::writer::DatabaseWriter>>,
) -> Vec<ToolCallSummary> {
    // TODO: Use tool_calls when implementing migration
    let _tool_calls: Vec<ToolCallSummary> = Vec::new();
    // TODO: Use execution_start_time when implementing migration
    let _execution_start_time = chrono::Utc::now();

    info!(
        "[PingPongExecution] Starting ping-pong execution for flow plan: {}",
        flow_plan.flow_id
    );
    info!(
        "[PingPongExecution] Agent type: {}, Steps: {}",
        agent_type,
        flow_plan.steps.len()
    );

    // TODO: Use orchestrator gateway for ping-pong execution with shared database if available
    // let gateway = match database {
    //     Some(db) => match OrchestratorGateway::with_database(db).await {
    //         Ok(gateway) => {
    //             info!("Using shared database for ping-pong execution");
    //             gateway
    //         }
    //         Err(e) => {
    //             error!("Failed to create gateway with shared database: {}, falling back to separate DB", e);
    //             match OrchestratorGateway::new().await {
    //                 Ok(gateway) => gateway,
    //                 Err(e) => {
    //                     error!("Failed to create gateway fallback: {}", e);
    //                     return vec![];
    //                 }
    //             }
    //         }
    //     },
    //     None => match OrchestratorGateway::new().await {
    //         Ok(gateway) => gateway,
    //         Err(e) => {
    //             error!("Failed to create gateway for ping-pong execution: {}", e);
    //             return vec![];
    //         }
    //     },
    // };
    // TODO: Execute flow with ping-pong using the gateway
    // let step_results = match gateway
    //     .execute_flow_with_ping_pong(flow_plan, agent_type)
    //     .await
    // {
    //     Ok(results) => {
    //         info!(
    //             "[PingPongExecution] ✅ Flow execution completed: {} step results",
    //             results.len()
    //         );
    //         results
    //     }
    //     Err(e) => {
    //         error!("[PingPongExecution] ❌ Flow execution failed: {}", e);
    //         // Return empty tool calls on execution failure
    //         return vec![];
    //     }
    // };

    // Temporarily return empty tool calls until OrchestratorGateway is replaced
    info!("[PingPongExecution] Ping-pong execution is temporarily disabled - OrchestratorGateway needs to be replaced");
    vec![]
}
