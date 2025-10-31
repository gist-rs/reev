//! Transaction log parser for converting blockchain transaction data to ASCII format
//!
//! This module provides functionality to parse blockchain transaction logs from execution data
//! and convert them into human-readable ASCII tree formats. It supports:
//! - Transaction logs from completed executions
//! - Structured blockchain transaction data
//! - Program call hierarchies with proper nesting
//!
//! The parser formats transaction logs with visual indicators for:
//! - Program calls and instructions
//! - Account operations
//! - Compute unit usage
//! - Success/failure status

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reev_db::writer::DatabaseWriterTrait;

use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::handlers::parsers::transaction_logs::TransactionLogParser;
use crate::types::ApiState;

/// Get transaction logs for a benchmark
pub async fn get_transaction_logs(
    State(state): State<ApiState>,
    Path(benchmark_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!(
        "🔍 [get_transaction_logs] Request received - benchmark_id: {}, query_params: {:?}",
        benchmark_id, params
    );

    // Check if specific execution_id is requested
    let target_execution_id = params.get("execution_id");

    // Use provided execution_id or find most recent execution for this benchmark
    let execution_id = if let Some(exec_id) = target_execution_id {
        exec_id.clone()
    } else {
        // Find most recent execution for this benchmark
        match state
            .db
            .list_execution_states_by_benchmark(&benchmark_id)
            .await
        {
            Ok(executions) => {
                if let Some(latest_execution) = executions.first() {
                    info!(
                        "📋 [get_transaction_logs] Using latest execution_id for benchmark {}: {} (total executions: {})",
                        benchmark_id, latest_execution.execution_id, executions.len()
                    );
                    latest_execution.execution_id.clone()
                } else {
                    warn!(
                        "❌ [get_transaction_logs] No executions found for benchmark: {}",
                        benchmark_id
                    );
                    return Json(json!({
                        "benchmark_id": benchmark_id,
                        "execution_id": null,
                        "is_running": false,
                        "message": "No execution history found for this benchmark",
                        "transaction_logs": "📝 No transaction logs available"
                    }))
                    .into_response();
                }
            }
            Err(e) => {
                warn!(
                    "❌ [get_transaction_logs] Failed to list executions for benchmark {}: {}",
                    benchmark_id, e
                );
                return Json(json!({
                    "benchmark_id": benchmark_id,
                    "execution_id": null,
                    "is_running": false,
                    "message": format!("Database error: {}", e),
                    "transaction_logs": "❌ Database error occurred"
                }))
                .into_response();
            }
        }
    };

    info!(
        "🎯 [get_transaction_logs] Looking up execution state for execution_id: {} (target_execution_id: {:?})",
        execution_id, target_execution_id
    );

    // Get execution state from database
    match state.db.get_execution_state(&execution_id).await {
        Ok(Some(updated_state)) => {
            let db_status = updated_state.status.clone();

            info!(
                "📊 [get_transaction_logs] Found execution state - execution_id: {}, status: {:?}, agent: {}",
                execution_id, db_status, updated_state.agent
            );

            // Extract transaction logs from result_data using proper ASCII tree parser
            let transaction_logs = if let Some(result_data) = &updated_state.result_data {
                info!("🔍 [get_transaction_logs] Parsing transaction logs from result_data");
                let parser = TransactionLogParser::new();
                match parser.generate_from_result_data(result_data) {
                    Ok(logs) => {
                        info!("✅ [get_transaction_logs] Successfully generated {} chars of transaction logs", logs.len());
                        logs
                    }
                    Err(e) => {
                        error!(
                            "❌ [get_transaction_logs] Failed to generate transaction logs: {}",
                            e
                        );
                        format!("❌ Failed to parse transaction logs: {e}")
                    }
                }
            } else {
                "📝 No transaction data available".to_string()
            };

            // Return execution results with ASCII trace
            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "agent_type": updated_state.agent,
                "interface": "web",
                "status": format!("{db_status:?}").to_lowercase(),
                "final_status": db_status,
                "transaction_logs": transaction_logs,
                "is_running": false,
                "progress": 1.0,
                "result": updated_state.result_data.clone()
            });

            Json(response).into_response()
        }
        Ok(None) => {
            warn!(
                "❌ [get_transaction_logs] No execution state found for execution_id: {} (benchmark: {})",
                execution_id, benchmark_id
            );

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "error": "Execution not found",
                "message": format!("No execution found with ID: {}", execution_id),
                "transaction_logs": "",
                "is_running": false,
                "progress": 0.0
            });

            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            warn!(
                "❌ [get_transaction_logs] Database lookup failed for execution_id: {} (benchmark: {}): {}",
                execution_id, benchmark_id, e
            );

            let response = json!({
                "benchmark_id": benchmark_id,
                "execution_id": execution_id,
                "error": "Database error",
                "message": format!("Failed to get execution: {}", e),
                "transaction_logs": "",
                "is_running": false,
                "progress": 0.0
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Fallback formatter for simple cases (kept for compatibility but not used in main flow)
fn format_transaction_log_simple(log_str: &str) -> String {
    let trimmed = log_str.trim();

    // Add icons for different types of transaction logs
    if trimmed.contains("invoke [") {
        if trimmed.contains("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            format!(
                "  🪙 Token Program {}",
                &trimmed[trimmed.find("invoke [").unwrap() + "invoke [".len()..]
            )
        } else if trimmed.contains("11111111111111111111111111111111") {
            format!(
                "  🔧 System Program {}",
                &trimmed[trimmed.find("invoke [").unwrap() + "invoke [".len()..]
            )
        } else {
            format!(
                "  📦 Program {}",
                &trimmed[trimmed.find("invoke [").unwrap() + "invoke [".len()..]
            )
        }
    } else if trimmed.contains("success") {
        format!(
            "  ✅ {}",
            &trimmed[trimmed.find("success").unwrap() + "success".len()..]
        )
    } else if trimmed.contains("compute units") {
        format!("  ⚡ {trimmed}")
    } else if trimmed.contains("Program log:") {
        format!(
            "  📝 {}",
            &trimmed[trimmed.find("Program log:").unwrap() + "Program log:".len()..]
        )
    } else if trimmed.contains("Program return:") {
        format!(
            "  ↩️ {}",
            &trimmed[trimmed.find("Program return:").unwrap() + "Program return:".len()..]
        )
    } else {
        format!("  {trimmed}")
    }
}
