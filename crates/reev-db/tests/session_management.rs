//! Test unified session management functionality
//!
//! This test verifies that the new session management system works correctly
//! for both TUI and Web interfaces, ensuring consistent database writes.

use reev_db::types::{SessionFilter, SessionInfo, SessionResult};
use reev_db::{DatabaseConfig, DatabaseWriter};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

#[tokio::test]
async fn test_session_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // Use temporary directory for test database
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_sessions.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());
    let db = DatabaseWriter::new(config).await?;

    // Test session creation
    let session_id = uuid::Uuid::new_v4().to_string();
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let session = SessionInfo {
        session_id: session_id.clone(),
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "test-agent".to_string(),
        interface: "tui".to_string(),
        start_time: start_time as i64,
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    };

    // Create session
    db.create_session(&session).await?;
    println!("✅ Session created: {session_id}");

    // Test session retrieval
    let filter = SessionFilter {
        benchmark_id: Some("test-benchmark".to_string()),
        agent_type: None,
        interface: None,
        status: None,
        limit: None,
    };

    let sessions = db.list_sessions(&filter).await?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, session_id);
    assert_eq!(sessions[0].interface, "tui");
    println!("✅ Session retrieved correctly");

    // Test session completion
    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let result = SessionResult {
        end_time: end_time as i64,
        score: 0.85,
        final_status: "completed".to_string(),
    };

    db.complete_session(&session_id, &result).await?;
    println!("✅ Session completed");

    // Test log storage
    let log_content = r#"{
        "session_id": "test-session",
        "events": [
            {"timestamp": 1698765432, "event_type": "prompt", "data": "test prompt"},
            {"timestamp": 1698765435, "event_type": "tool_call", "data": "test tool"}
        ]
    }"#;

    db.store_complete_log(&session_id, log_content).await?;
    println!("✅ Log stored successfully");

    // Test log retrieval
    let retrieved_log = db.get_session_log(&session_id).await?;
    assert!(retrieved_log.contains("test prompt"));
    assert!(retrieved_log.contains("test tool"));
    println!("✅ Log retrieved successfully");

    // Test Web interface session
    let web_session_id = uuid::Uuid::new_v4().to_string();
    let web_session = SessionInfo {
        session_id: web_session_id.clone(),
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "web-agent".to_string(),
        interface: "web".to_string(),
        start_time: start_time as i64 + 10,
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    };

    db.create_session(&web_session).await?;

    // Test filtering by interface
    let web_filter = SessionFilter {
        benchmark_id: None,
        agent_type: None,
        interface: Some("web".to_string()),
        status: None,
        limit: None,
    };

    let web_sessions = db.list_sessions(&web_filter).await?;
    assert_eq!(web_sessions.len(), 1);
    assert_eq!(web_sessions[0].interface, "web");
    println!("✅ Web interface session works correctly");

    // Test filtering by agent
    let agent_filter = SessionFilter {
        benchmark_id: None,
        agent_type: Some("test-agent".to_string()),
        interface: None,
        status: None,
        limit: None,
    };

    let agent_sessions = db.list_sessions(&agent_filter).await?;
    assert_eq!(agent_sessions.len(), 1);
    assert_eq!(agent_sessions[0].agent_type, "test-agent");
    println!("✅ Agent filtering works correctly");

    // Test database will be cleaned up automatically when temp_dir goes out of scope
    println!("✅ Test database cleaned up");

    Ok(())
}

#[tokio::test]
async fn test_interface_consistency() -> Result<(), Box<dyn std::error::Error>> {
    // Test that TUI and Web interfaces create identical database records
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_interface_consistency.db");
    let config = DatabaseConfig::new(db_path.to_string_lossy());
    let db = DatabaseWriter::new(config).await?;

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Create TUI session
    let tui_session = SessionInfo {
        session_id: uuid::Uuid::new_v4().to_string(),
        benchmark_id: "consistency-test".to_string(),
        agent_type: "deterministic".to_string(),
        interface: "tui".to_string(),
        start_time,
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    };

    // Create Web session (identical except interface)
    let web_session = SessionInfo {
        session_id: uuid::Uuid::new_v4().to_string(),
        benchmark_id: "consistency-test".to_string(),
        agent_type: "deterministic".to_string(),
        interface: "web".to_string(),
        start_time: start_time + 1, // Slightly different time
        end_time: None,
        status: "running".to_string(),
        score: None,
        final_status: None,
    };

    db.create_session(&tui_session).await?;
    db.create_session(&web_session).await?;

    // Complete both sessions with identical results
    let result = SessionResult {
        end_time: start_time + 100,
        score: 0.75,
        final_status: "completed".to_string(),
    };

    db.complete_session(&tui_session.session_id, &result)
        .await?;
    db.complete_session(&web_session.session_id, &result)
        .await?;

    // Store identical logs for both
    let log_content = r#"{
        "session_id": "consistency-test",
        "events": [
            {"timestamp": 1698765432, "event_type": "prompt", "data": "Swap 1 SOL to USDC"},
            {"timestamp": 1698765435, "event_type": "tool_call", "data": "jupiter_quote"},
            {"timestamp": 1698765440, "event_type": "transaction", "data": "tx_executed"},
            {"timestamp": 1698765445, "event_type": "result", "data": "success"}
        ]
    }"#;

    db.store_complete_log(&tui_session.session_id, log_content)
        .await?;
    db.store_complete_log(&web_session.session_id, log_content)
        .await?;

    // Verify both sessions have identical data structure
    let sessions = db.list_sessions(&SessionFilter::default()).await?;
    assert_eq!(sessions.len(), 2);

    // Both should have same score, status, and log content
    for session in &sessions {
        assert_eq!(session.score, Some(0.75));
        assert_eq!(session.final_status.as_ref().unwrap(), "completed");
        assert_eq!(session.status, "completed");

        let log = db.get_session_log(&session.session_id).await?;
        assert!(log.contains("Swap 1 SOL to USDC"));
        assert!(log.contains("jupiter_quote"));
    }

    println!(
        "✅ Interface consistency test passed - TUI and Web produce identical database records"
    );

    // Test database will be cleaned up automatically when temp_dir goes out of scope

    Ok(())
}
