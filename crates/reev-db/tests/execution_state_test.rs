//! Test execution state storage functionality

use reev_db::{writer::DatabaseWriterTrait, DatabaseConfig, DatabaseWriter};
use reev_types::{ExecutionState, ExecutionStatus};
use tempfile::TempDir;

async fn create_test_db() -> DatabaseWriter {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());

    DatabaseWriter::new(config)
        .await
        .expect("Failed to create test database")
}

#[tokio::test]
async fn test_basic_execution_state_storage() -> Result<(), Box<dyn std::error::Error>> {
    let db = create_test_db().await;

    // Create a simple execution state
    let mut execution_state = ExecutionState::new(
        "test-execution-1".to_string(),
        "test-benchmark".to_string(),
        "test-agent".to_string(),
    );

    execution_state.update_status(ExecutionStatus::Completed);
    execution_state.complete(serde_json::json!({
        "success": true,
        "score": 1.0,
        "test": true
    }));

    // Store the execution state
    db.store_execution_state(&execution_state).await?;

    // Retrieve the execution state
    let retrieved_state = db.get_execution_state("test-execution-1").await?;

    assert!(retrieved_state.is_some(), "Execution state should be found");

    let retrieved = retrieved_state.unwrap();
    assert_eq!(retrieved.execution_id, "test-execution-1");
    assert_eq!(retrieved.benchmark_id, "test-benchmark");
    assert_eq!(retrieved.agent, "test-agent");
    assert_eq!(retrieved.status, ExecutionStatus::Completed);
    assert!(retrieved.result_data.is_some());

    let result_data = retrieved.result_data.unwrap();
    assert_eq!(result_data["success"], true);
    assert_eq!(result_data["score"], 1.0);
    assert_eq!(result_data["test"], true);

    Ok(())
}

#[tokio::test]
async fn test_execution_state_with_error() -> Result<(), Box<dyn std::error::Error>> {
    let db = create_test_db().await;

    // Create an execution state with error
    let mut execution_state = ExecutionState::new(
        "test-execution-error".to_string(),
        "test-benchmark-error".to_string(),
        "test-agent-error".to_string(),
    );

    execution_state.update_status(ExecutionStatus::Failed);
    execution_state.set_error("Test error message".to_string());

    // Store the execution state
    db.store_execution_state(&execution_state).await?;

    // Retrieve the execution state
    let retrieved_state = db.get_execution_state("test-execution-error").await?;

    assert!(
        retrieved_state.is_some(),
        "Execution state with error should be found"
    );

    let retrieved = retrieved_state.unwrap();
    assert_eq!(retrieved.execution_id, "test-execution-error");
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert!(retrieved.error_message.is_some());
    assert_eq!(retrieved.error_message.unwrap(), "Test error message");

    Ok(())
}

#[tokio::test]
async fn test_execution_state_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let db = create_test_db().await;

    // Create an execution state with metadata
    let mut execution_state = ExecutionState::new(
        "test-execution-metadata".to_string(),
        "test-benchmark-metadata".to_string(),
        "test-agent-metadata".to_string(),
    );

    // Add some metadata
    execution_state
        .metadata
        .insert("test_key".to_string(), serde_json::json!("test_value"));
    execution_state
        .metadata
        .insert("number".to_string(), serde_json::json!(42));

    execution_state.update_status(ExecutionStatus::Running);

    // Store the execution state
    db.store_execution_state(&execution_state).await?;

    // Retrieve the execution state
    let retrieved_state = db.get_execution_state("test-execution-metadata").await?;

    assert!(
        retrieved_state.is_some(),
        "Execution state with metadata should be found"
    );

    let retrieved = retrieved_state.unwrap();
    assert_eq!(retrieved.execution_id, "test-execution-metadata");
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert_eq!(retrieved.metadata.len(), 2);
    assert_eq!(retrieved.metadata["test_key"], "test_value");
    assert_eq!(retrieved.metadata["number"], 42);

    Ok(())
}
