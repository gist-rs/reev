use anyhow::Result;
use reev_db::writer::DatabaseWriterTrait;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use reev_types::{ExecutionState, ExecutionStatus};
use std::collections::HashMap;

/// Test database operations in isolation to identify corruption issue
#[tokio::test]
async fn test_database_operations_isolation() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🧪 Testing database operations in isolation");

    // Setup fresh in-memory database - guaranteed clean state
    let db_config = DatabaseConfig::new("sqlite::memory:");
    let db = PooledDatabaseWriter::new(db_config, 1).await?;

    let execution_id = "test-isolation-123";
    let benchmark_id = "001-sol-transfer";
    let agent = "glm-4.6";

    println!(
        "📋 Test parameters: execution_id={execution_id}, agent={agent}"
    );

    // Create initial execution state
    let initial_state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent: agent.to_string(),
        status: ExecutionStatus::Queued,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        progress: Some(0.0),
        error_message: None,
        result_data: None,
        metadata: HashMap::new(),
    };

    println!("💾 Storing initial state (Queued)...");
    db.store_execution_state(&initial_state).await?;
    println!("✅ Initial state stored successfully");

    // Verify initial state
    let retrieved = db.get_execution_state(execution_id).await?;
    assert!(
        retrieved.is_some(),
        "Should be able to retrieve initial state"
    );
    println!("✅ Initial state verified: {:?}", retrieved.unwrap().status);

    // Create completed execution state
    let completed_state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent: agent.to_string(),
        status: ExecutionStatus::Completed,
        created_at: initial_state.created_at, // Keep same created_at
        updated_at: chrono::Utc::now(),
        progress: Some(1.0),
        error_message: None,
        result_data: Some(serde_json::json!({
            "success": true,
            "score": 1.0,
            "status": "Succeeded",
            "execution_time_ms": 37000,
            "data": {
                "prompt": "Test prompt",
                "steps": [
                    {
                        "action": [{"program_id": "11111111111111111111111111111111"}],
                        "observation": {"last_transaction_status": "Success"}
                    }
                ]
            }
        })),
        metadata: HashMap::new(),
    };

    println!("💾 Storing completed state...");
    db.store_execution_state(&completed_state).await?;
    println!("✅ Completed state stored successfully");

    // Verify final state
    let final_retrieved = db.get_execution_state(execution_id).await?;
    assert!(
        final_retrieved.is_some(),
        "Should be able to retrieve final state"
    );

    let final_state = final_retrieved.unwrap();
    assert_eq!(final_state.status, ExecutionStatus::Completed);
    assert_eq!(final_state.progress, Some(1.0));
    assert!(final_state.result_data.is_some());

    if let Some(result_data) = &final_state.result_data {
        assert_eq!(result_data["success"], true);
        assert_eq!(result_data["score"], 1.0);
        println!(
            "✅ Final state verified: success={}, score={}",
            result_data["success"], result_data["score"]
        );
    }

    println!("🎉 Database isolation test completed successfully!");

    // No cleanup needed for in-memory database

    Ok(())
}

/// Test with file-based database to check for corruption
#[tokio::test]
async fn test_database_file_operations() -> Result<()> {
    println!("🧪 Testing file-based database operations");

    let db_path = "test_file_isolation.db";
    let _ = std::fs::remove_file(db_path); // Clean up any existing file

    let db_config = DatabaseConfig::new(db_path);
    let db = PooledDatabaseWriter::new(db_config, 1).await?;

    let execution_id = "test-file-456";

    // Store single state
    let state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: "001-sol-transfer".to_string(),
        agent: "glm-4.6".to_string(),
        status: ExecutionStatus::Completed,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        progress: Some(1.0),
        error_message: None,
        result_data: Some(serde_json::json!({
            "success": true,
            "score": 1.0
        })),
        metadata: HashMap::new(),
    };

    db.store_execution_state(&state).await?;

    // Retrieve and verify
    let retrieved = db.get_execution_state(execution_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().status, ExecutionStatus::Completed);

    println!("✅ File-based database test successful");

    // Cleanup
    let _ = std::fs::remove_file(db_path);

    Ok(())
}
