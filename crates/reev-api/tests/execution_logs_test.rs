//! Tests for execution logs parser and handler

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use reev_api::handlers::execution_logs::get_execution_trace;
use reev_db::writer::DatabaseWriterTrait;
use reev_types::{ExecutionState, ExecutionStatus};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

/// Test execution trace generation with session data format
#[tokio::test]
async fn test_execution_trace_from_session_data() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create pooled database directly to ensure we use the same connection
    let db_config = reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy()));
    let pooled_db = reev_lib::db::PooledDatabaseWriter::new(db_config, 5).await?;

    // Create test execution state with session data format
    let mut execution_state = ExecutionState::new(
        "test-execution-1".to_string(),
        "001-sol-transfer".to_string(),
        "deterministic".to_string(),
    );

    // Session data format - what we expect from completed executions
    let session_data = json!({
        "session_id": "test-execution-1",
        "benchmark_id": "001-sol-transfer",
        "agent_type": "deterministic",
        "success": true,
        "score": 0.95,
        "steps": [
            {
                "action": [
                    {
                        "program_id": "11111111111111111111111111111111",
                        "accounts": [
                            {"pubkey": "source_account", "is_signer": true, "is_writable": true},
                            {"pubkey": "dest_account", "is_signer": false, "is_writable": true}
                        ],
                        "data": "base58_encoded_data"
                    }
                ],
                "observation": {
                    "last_transaction_status": "success",
                    "last_transaction_error": ""
                }
            },
            {
                "action": [
                    {
                        "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                        "accounts": [
                            {"pubkey": "token_account", "is_signer": false, "is_writable": true}
                        ],
                        "data": "token_instruction_data"
                    }
                ],
                "observation": {
                    "last_transaction_status": "success",
                    "last_transaction_error": ""
                }
            }
        ]
    });

    execution_state.complete(session_data);
    println!(
        "Storing execution state with ID: {}",
        execution_state.execution_id
    );
    pooled_db.store_execution_state(&execution_state).await?;
    println!("Stored execution state successfully");

    // Create API state with the same database instance
    let api_state = reev_api::types::ApiState {
        db: pooled_db,
        agent_configs: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        benchmark_executor: std::sync::Arc::new(
            reev_api::services::PooledBenchmarkExecutor::new_with_default(std::sync::Arc::new(
                reev_lib::db::PooledDatabaseWriter::new(
                    reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy())),
                    5,
                )
                .await?,
            )),
        ),
    };

    // Test the handler with execution_id parameter
    let mut params = HashMap::new();
    params.insert("execution_id".to_string(), "test-execution-1".to_string());

    let response = get_execution_trace(
        State(api_state),
        Path("001-sol-transfer".to_string()),
        Query(params),
    )
    .await
    .into_response();

    // Get status before moving response
    let status = response.status();
    println!("Response status: {status}");

    // Get body for both debug and verification
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;

    // Print response body if not 200
    if status != 200 {
        let body_str = String::from_utf8_lossy(&body);
        println!("Response body: {body_str}");
        panic!("Expected status 200, got {status}");
    }

    // Verify response
    assert_eq!(status, 200);
    let response_json: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(response_json["benchmark_id"], "001-sol-transfer");
    assert_eq!(response_json["execution_id"], "test-execution-1");
    assert_eq!(response_json["agent_type"], "deterministic");
    assert_eq!(response_json["status"], "completed");
    assert_eq!(response_json["is_running"], false);
    assert_eq!(response_json["progress"], 1.0);

    // Check that trace contains ASCII tree format
    let trace = response_json["trace"].as_str().unwrap();
    println!("Actual trace content:\n{trace}");

    // Check expected content
    assert!(trace.contains("🌊")); // ASCII tree marker (wave symbol)
    assert!(trace.contains("001-sol-transfer"));
    assert!(trace.contains("deterministic"));
    assert!(trace.contains("SUCCESS"));

    Ok(())
}

/// Test execution trace with error case
#[tokio::test]
async fn test_execution_trace_with_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create pooled database directly to ensure we use the same connection
    let db_config = reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy()));
    let pooled_db = reev_lib::db::PooledDatabaseWriter::new(db_config, 5).await?;

    // Create execution state with error
    let mut execution_state = ExecutionState::new(
        "test-execution-error".to_string(),
        "001-sol-transfer".to_string(),
        "deterministic".to_string(),
    );

    execution_state.update_status(ExecutionStatus::Failed);
    execution_state.set_error("Transaction failed: insufficient funds".to_string());

    // Set result_data to indicate failure without overriding status
    execution_state.result_data = Some(serde_json::json!({
        "success": false,
        "score": 0.0,
        "error": "Transaction failed: insufficient funds",
        "steps": [] // No steps due to failure
    }));

    pooled_db.store_execution_state(&execution_state).await?;

    // Create API state with the same database instance
    let api_state = reev_api::types::ApiState {
        db: pooled_db,
        agent_configs: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        benchmark_executor: std::sync::Arc::new(
            reev_api::services::PooledBenchmarkExecutor::new_with_default(std::sync::Arc::new(
                reev_lib::db::PooledDatabaseWriter::new(
                    reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy())),
                    5,
                )
                .await?,
            )),
        ),
    };

    // Test the handler with execution_id parameter
    let mut params = HashMap::new();
    params.insert(
        "execution_id".to_string(),
        "test-execution-error".to_string(),
    );

    let response = get_execution_trace(
        State(api_state),
        Path("001-sol-transfer".to_string()),
        Query(params),
    )
    .await
    .into_response();

    // Verify response
    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let response_json: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(response_json["status"], "failed");
    assert_eq!(response_json["is_running"], false);

    // Check error message in trace
    let trace = response_json["trace"].as_str().unwrap();
    assert!(trace.contains("❌") || trace.contains("FAILED"));

    Ok(())
}

/// Test execution trace with missing execution_id
#[tokio::test]
async fn test_execution_trace_missing_execution_id() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create pooled database directly to ensure we use the same connection
    let db_config = reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy()));
    let pooled_db = reev_lib::db::PooledDatabaseWriter::new(db_config, 5).await?;

    // Create API state with the same database instance
    let api_state = reev_api::types::ApiState {
        db: pooled_db,
        agent_configs: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        benchmark_executor: std::sync::Arc::new(
            reev_api::services::PooledBenchmarkExecutor::new_with_default(std::sync::Arc::new(
                reev_lib::db::PooledDatabaseWriter::new(
                    reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy())),
                    5,
                )
                .await?,
            )),
        ),
    };

    // Test the handler without execution_id parameter
    let params = HashMap::new(); // No execution_id

    let response = get_execution_trace(
        State(api_state),
        Path("001-sol-transfer".to_string()),
        Query(params),
    )
    .await
    .into_response();

    // Verify error response
    assert_eq!(response.status(), 400);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let response_json: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(response_json["error"], "execution_id parameter is required");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("Please use GET /api/v1/benchmarks/{id}"));

    Ok(())
}

/// Test execution trace with non-existent execution_id
#[tokio::test]
async fn test_execution_trace_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    // Create pooled database directly to ensure we use the same connection
    let db_config = reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy()));
    let pooled_db = reev_lib::db::PooledDatabaseWriter::new(db_config, 5).await?;

    // Create API state with the same database instance
    let api_state = reev_api::types::ApiState {
        db: pooled_db,
        agent_configs: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        benchmark_executor: std::sync::Arc::new(
            reev_api::services::PooledBenchmarkExecutor::new_with_default(std::sync::Arc::new(
                reev_lib::db::PooledDatabaseWriter::new(
                    reev_db::DatabaseConfig::new(format!("sqlite:{}", db_path.to_string_lossy())),
                    5,
                )
                .await?,
            )),
        ),
    };

    // Test the handler with non-existent execution_id
    let mut params = HashMap::new();
    params.insert(
        "execution_id".to_string(),
        "non-existent-execution".to_string(),
    );

    let response = get_execution_trace(
        State(api_state),
        Path("001-sol-transfer".to_string()),
        Query(params),
    )
    .await
    .into_response();

    // Verify 404 response
    assert_eq!(response.status(), 404);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let response_json: serde_json::Value = serde_json::from_slice(&body)?;

    assert_eq!(response_json["error"], "Execution not found");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("No execution found"));

    Ok(())
}

/// Helper function to generate mock session data for testing
/// Helper function to generate mock session data for testing
#[allow(dead_code)]
fn create_mock_session_data(execution_id: &str, benchmark_id: &str) -> serde_json::Value {
    json!({
        "session_id": execution_id,
        "benchmark_id": benchmark_id,
        "agent_type": "deterministic",
        "success": true,
        "score": 0.85,
        "steps": [
            {
                "action": [
                    {
                        "program_id": "11111111111111111111111111111111",
                        "accounts": [
                            {"pubkey": "source_pubkey", "is_signer": true, "is_writable": true},
                            {"pubkey": "dest_pubkey", "is_signer": false, "is_writable": true},
                            {"pubkey": "system_program", "is_signer": false, "is_writable": false}
                        ],
                        "data": "transfer_instruction_data"
                    }
                ],
                "observation": {
                    "last_transaction_status": "success",
                    "last_transaction_error": ""
                }
            },
            {
                "action": [
                    {
                        "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                        "accounts": [
                            {"pubkey": "token_mint", "is_signer": false, "is_writable": false},
                            {"pubkey": "source_token", "is_signer": false, "is_writable": true},
                            {"pubkey": "dest_token", "is_signer": false, "is_writable": true},
                            {"pubkey": "owner", "is_signer": true, "is_writable": false}
                        ],
                        "data": "token_transfer_instruction"
                    }
                ],
                "observation": {
                    "last_transaction_status": "success",
                    "last_transaction_error": ""
                }
            }
        ]
    })
}
