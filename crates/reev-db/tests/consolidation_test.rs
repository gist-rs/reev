//! Test consolidation database methods
//!
//! This test verifies that the consolidation database methods work correctly
//! for the PingPongExecutor integration.

use reev_db::shared::performance::{ConsolidationMetadata, SessionLog};
use reev_db::writer::DatabaseWriterTrait;
use reev_db::{DatabaseConfig, DatabaseWriter};

#[tokio::test]
async fn test_consolidation_database_methods() -> Result<(), Box<dyn std::error::Error>> {
    // Create test database
    let config = DatabaseConfig::new("sqlite::memory:");
    let db = DatabaseWriter::new(config).await?;

    // Test data
    let execution_id = "test_execution_001";
    let consolidated_id = "consolidated_001";
    let step_content = r#"{"session_id": "test_session", "events": []}"#;

    // Test 1: Store step session
    println!("Testing store_step_session...");
    db.store_step_session(execution_id, 1, step_content).await?;
    println!("✅ Step session stored successfully");

    // Test 2: Get sessions for consolidation
    println!("Testing get_sessions_for_consolidation...");

    let sessions = db.get_sessions_for_consolidation(execution_id).await?;
    assert!(!sessions.is_empty(), "Should have at least one session");
    println!("✅ Retrieved {} sessions for consolidation", sessions.len());

    // Test 3: Store consolidated session
    println!("Testing store_consolidated_session...");
    let metadata =
        ConsolidationMetadata::with_values(Some(0.85), Some(5), Some(100.0), Some(30000));

    let consolidated_content = r#"{"consolidated": true, "steps": []}"#;
    db.store_consolidated_session(
        consolidated_id,
        execution_id,
        consolidated_content,
        &metadata,
    )
    .await?;
    println!("✅ Consolidated session stored successfully");

    // Test 4: Get consolidated session
    println!("Testing get_consolidated_session...");
    let retrieved_content = db.get_consolidated_session(consolidated_id).await?;
    assert!(
        retrieved_content.is_some(),
        "Should retrieve consolidated content"
    );
    assert_eq!(
        retrieved_content.unwrap(),
        consolidated_content,
        "Retrieved content should match stored content"
    );
    println!("✅ Consolidated session retrieved successfully");

    // Test 5: Transaction operations
    println!("Testing transaction operations...");
    db.begin_transaction(execution_id).await?;
    println!("✅ Transaction begun");

    db.store_step_session(execution_id, 2, step_content).await?;
    println!("✅ Step session stored in transaction");

    db.commit_transaction(execution_id).await?;
    println!("✅ Transaction committed");

    // Test 6: Transaction rollback
    println!("Testing transaction rollback...");
    db.begin_transaction(execution_id).await?;

    // This should be rolled back
    db.store_step_session(execution_id, 3, "this will be rolled back")
        .await?;

    db.rollback_transaction(execution_id).await?;
    println!("✅ Transaction rolled back");

    // Verify rollback worked
    let sessions_after_rollback = db.get_sessions_for_consolidation(execution_id).await?;
    let found_rolled_back = sessions_after_rollback
        .iter()
        .any(|s| s.content.contains("this will be rolled back"));
    assert!(!found_rolled_back, "Rolled back session should not exist");
    println!("✅ Transaction rollback verified");

    println!("\n🎉 All consolidation database methods working correctly!");

    Ok(())
}

#[tokio::test]
async fn test_consolidation_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Test ConsolidationMetadata creation and serialization
    let metadata =
        ConsolidationMetadata::with_values(Some(0.92), Some(10), Some(95.5), Some(45000));

    // Test serialization
    let json = serde_json::to_string(&metadata)?;
    println!("✅ ConsolidationMetadata serializes correctly: {json}");

    // Test deserialization
    let deserialized: ConsolidationMetadata = serde_json::from_str(&json)?;
    assert_eq!(deserialized.avg_score, Some(0.92));
    assert_eq!(deserialized.total_tools, Some(10));
    assert_eq!(deserialized.success_rate, Some(95.5));
    assert_eq!(deserialized.execution_duration_ms, Some(45000));
    println!("✅ ConsolidationMetadata deserializes correctly");

    // Test default creation
    let default_metadata = ConsolidationMetadata::new();
    assert_eq!(default_metadata.avg_score, None);
    assert_eq!(default_metadata.total_tools, None);
    assert_eq!(default_metadata.success_rate, None);
    assert_eq!(default_metadata.execution_duration_ms, None);
    println!("✅ ConsolidationMetadata::new() works correctly");

    Ok(())
}

#[tokio::test]
async fn test_session_log_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Test SessionLog creation and serialization
    let session_log = SessionLog {
        session_id: "session_001".to_string(),
        execution_id: "execution_001".to_string(),
        content: r#"{"test": "content"}"#.to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        status: "completed".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&session_log)?;
    println!("✅ SessionLog serializes correctly: {json}");

    // Test deserialization
    let deserialized: SessionLog = serde_json::from_str(&json)?;
    assert_eq!(deserialized.session_id, "session_001");
    assert_eq!(deserialized.execution_id, "execution_001");
    assert_eq!(deserialized.content, r#"{"test": "content"}"#);
    assert_eq!(deserialized.timestamp, "2024-01-01T00:00:00Z");
    assert_eq!(deserialized.status, "completed");
    println!("✅ SessionLog deserializes correctly");

    Ok(())
}
