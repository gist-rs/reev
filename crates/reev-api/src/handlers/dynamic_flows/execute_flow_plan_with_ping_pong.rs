//! Flow Plan Execution with Ping-Pong Handler
//!
//! This module provides the handler for executing flow plans using ping-pong coordination.

use reev_orchestrator::OrchestratorGateway;
use reev_types::execution::ToolCallSummary;
use std::sync::Arc;
use tracing::{error, info};

use super::extract_transaction_details::extract_transaction_details;

/// Execute a flow plan using ping-pong coordination
#[allow(dead_code)]
pub async fn execute_flow_plan_with_ping_pong(
    flow_plan: &reev_types::flow::DynamicFlowPlan,
    agent_type: &str,
    database: Option<Arc<reev_db::writer::DatabaseWriter>>,
) -> Vec<ToolCallSummary> {
    let mut tool_calls = Vec::new();
    let execution_start_time = chrono::Utc::now();

    info!(
        "[PingPongExecution] Starting ping-pong execution for flow plan: {}",
        flow_plan.flow_id
    );
    info!(
        "[PingPongExecution] Agent type: {}, Steps: {}",
        agent_type,
        flow_plan.steps.len()
    );

    // Use orchestrator gateway for ping-pong execution with shared database if available
    let gateway = match database {
        Some(db) => match OrchestratorGateway::with_database(db).await {
            Ok(gateway) => {
                info!("Using shared database for ping-pong execution");
                gateway
            }
            Err(e) => {
                error!("Failed to create gateway with shared database: {}, falling back to separate DB", e);
                match OrchestratorGateway::new().await {
                    Ok(gateway) => gateway,
                    Err(e) => {
                        error!("Failed to create gateway fallback: {}", e);
                        return vec![];
                    }
                }
            }
        },
        None => match OrchestratorGateway::new().await {
            Ok(gateway) => gateway,
            Err(e) => {
                error!("Failed to create gateway for ping-pong execution: {}", e);
                return vec![];
            }
        },
    };
    let step_results = match gateway
        .execute_flow_with_ping_pong(flow_plan, agent_type)
        .await
    {
        Ok(results) => {
            info!(
                "[PingPongExecution] ✅ Flow execution completed: {} step results",
                results.len()
            );
            results
        }
        Err(e) => {
            error!("[PingPongExecution] ❌ Flow execution failed: {}", e);
            // Return empty tool calls on execution failure
            return vec![];
        }
    };

    info!(
        "[PingPongExecution] Converting {} step results to tool calls",
        step_results.len()
    );

    // Execute agent based on type
    // Convert step results to tool call summaries
    for (index, step_result) in step_results.iter().enumerate() {
        let duration_ms = step_result.execution_time_ms;

        // Extract tool name from step or tool calls
        let tool_name = if !step_result.tool_calls.is_empty() {
            step_result.tool_calls[0].clone()
        } else {
            // Infer from step ID - use type-safe enum approach
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

        // Extract execution data from step output
        let output = &step_result.output;
        // Use output directly since it's already a serde_json::Value
        let (params, result_data, tool_args) = extract_transaction_details(output);

        let tool_call_summary = ToolCallSummary {
            tool_name: tool_name.to_string(),
            timestamp: execution_start_time + chrono::Duration::milliseconds(index as i64 * 2000),
            duration_ms,
            success: step_result.success,
            error: step_result.error_message.clone(),
            params: Some(params),
            result_data: Some(result_data),
            tool_args,
        };

        tool_calls.push(tool_call_summary);

        info!(
            "[PingPongExecution] Added tool call: {} ({}ms) - {}",
            tool_name,
            duration_ms,
            if step_result.success {
                "SUCCESS"
            } else {
                "FAILED"
            }
        );
    }

    // Log completion summary
    let successful_calls = tool_calls.iter().filter(|t| t.success).count();
    let failed_calls = tool_calls.len() - successful_calls;

    info!(
        "[PingPongExecution] Execution summary: {} successful, {} failed tool calls",
        successful_calls, failed_calls
    );

    tool_calls
}
