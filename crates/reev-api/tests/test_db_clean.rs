use anyhow::Result;
use reev_db::writer::DatabaseWriterTrait;
use reev_db::{DatabaseConfig, PooledDatabaseWriter};
use reev_types::{ExecutionState, ExecutionStatus};
use std::collections::HashMap;

/// Test with guaranteed clean database to isolate corruption issue
#[tokio::test]
async fn test_completely_fresh_database() -> Result<()> {
    println!("🧪 Testing with completely fresh database");

    // Use unique in-memory database name to guarantee fresh state
    let db_name = format!("sqlite::memory:?fresh={}", uuid::Uuid::new_v4());
    let db_config = DatabaseConfig::new(&db_name);
    let db = PooledDatabaseWriter::new(db_config, 1).await?;

    let execution_id = "fresh-test-123";
    let benchmark_id = "001-sol-transfer";
    let agent = "glm-4.6";

    println!(
        "📋 Fresh test: execution_id={execution_id}, agent={agent}"
    );

    // Create and store only one state (no duplicate)
    let state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: benchmark_id.to_string(),
        agent: agent.to_string(),
        status: ExecutionStatus::Completed, // Skip Queued -> avoid duplicate issue
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        progress: Some(1.0),
        error_message: None,
        result_data: Some(serde_json::json!({
            "success": true,
            "score": 1.0,
            "status": "Succeeded"
        })),
        metadata: HashMap::new(),
    };

    println!("💾 Storing single state (Completed)...");
    db.store_execution_state(&state).await?;
    println!("✅ State stored successfully");

    // Verify it was stored
    let retrieved = db.get_execution_state(execution_id).await?;
    assert!(retrieved.is_some(), "Should retrieve stored state");

    let final_state = retrieved.unwrap();
    assert_eq!(final_state.status, ExecutionStatus::Completed);
    assert_eq!(final_state.progress, Some(1.0));
    assert!(final_state.result_data.is_some());

    if let Some(result_data) = &final_state.result_data {
        assert_eq!(result_data["success"], true);
        assert_eq!(result_data["score"], 1.0);
        println!("✅ Final state verified");
    }

    println!("🎉 Fresh database test passed!");

    Ok(())
}

/// Test database with individual operations to pinpoint corruption
#[tokio::test]
async fn test_individual_database_operations() -> Result<()> {
    println!("🧪 Testing individual database operations");

    let db_config = DatabaseConfig::new("sqlite::memory:");
    let db = PooledDatabaseWriter::new(db_config, 1).await?;

    let execution_id = "individual-test-456";

    // Test 1: Simple INSERT only
    println!("🔧 Test 1: Simple INSERT");
    let simple_state = ExecutionState {
        execution_id: execution_id.to_string(),
        benchmark_id: "test".to_string(),
        agent: "test".to_string(),
        status: ExecutionStatus::Queued,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        progress: Some(0.0),
        error_message: None,
        result_data: None,
        metadata: HashMap::new(),
    };

    db.store_execution_state(&simple_state).await?;
    println!("✅ Simple INSERT successful");

    // Test 2: Simple retrieval
    println!("🔧 Test 2: Simple retrieval");
    let retrieved = db.get_execution_state(execution_id).await?;
    assert!(retrieved.is_some(), "Should retrieve inserted state");
    println!("✅ Simple retrieval successful");

    // Test 3: Simple verification only
    println!("🔧 Test 3: Final verification");

    let final_retrieved = db.get_execution_state(execution_id).await?;
    assert!(final_retrieved.is_some());
    assert_eq!(final_retrieved.unwrap().status, ExecutionStatus::Completed);
    println!("✅ Final state verification successful");

    println!("🎉 Individual operations test passed!");

    Ok(())
}
