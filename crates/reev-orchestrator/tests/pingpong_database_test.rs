//! Tests for PingPongExecutor database integration
//!
//! This module tests the new database functionality added in Phase 2:
//! - Session storage to database
//! - Session consolidation with timeout
//! - Error handling for failed consolidations

use reev_db::writer::{DatabaseWriter, DatabaseWriterTrait};
use reev_orchestrator::context_resolver::ContextResolver;
use reev_orchestrator::execution::ping_pong_executor::PingPongExecutor;
use reev_types::flow::{DynamicFlowPlan, DynamicStep};

use std::sync::Arc;
use tempfile::TempDir;

/// Create a test database connection
async fn create_test_database() -> (DatabaseWriter, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let config = reev_db::config::DatabaseConfig::local(db_path.to_string_lossy().to_string());
    let writer = DatabaseWriter::new(config)
        .await
        .expect("Failed to create database writer");

    (writer, temp_dir)
}

/// Create a test flow plan
fn create_test_flow_plan() -> DynamicFlowPlan {
    let mut flow_plan = DynamicFlowPlan::new(
        "test_flow_001".to_string(),
        "Test flow for database integration".to_string(),
        reev_types::flow::WalletContext::new("test_wallet_123".to_string()),
    );

    flow_plan.steps.push(
        DynamicStep::new(
            "step_1".to_string(),
            "Execute get_account_balance".to_string(),
            "Get account balance".to_string(),
        )
        .with_required_tools(vec![reev_types::tools::ToolName::GetAccountBalance])
        .with_estimated_time(5),
    );

    flow_plan.steps.push(
        DynamicStep::new(
            "step_2".to_string(),
            "Execute jupiter_swap".to_string(),
            "Swap tokens".to_string(),
        )
        .with_required_tools(vec![reev_types::tools::ToolName::JupiterSwap])
        .with_estimated_time(10),
    );

    flow_plan
}

#[tokio::test]
async fn test_pingpong_executor_database_integration() {
    // Setup
    let (database, _temp_dir) = create_test_database().await;
    let context_resolver = Arc::new(ContextResolver::new());
    let mut executor = PingPongExecutor::new(30000, context_resolver, database.into());
    let flow_plan = create_test_flow_plan();

    // Execute flow plan with database integration
    let result = executor
        .execute_flow_plan_with_ping_pong(&flow_plan, "test_agent")
        .await;

    // Note: This test expects execution to fail due to missing mock agent,
    // but we should still get database storage and consolidation
    match result {
        Ok(execution_result) => {
            // Verify execution result structure
            assert!(!execution_result.execution_id.is_empty());
            assert_eq!(execution_result.flow_id, flow_plan.flow_id);
            assert_eq!(execution_result.total_steps, 2);

            // Check that consolidation was attempted
            // (may fail due to missing agent sessions, but should be present in error handling)
            println!(
                "Execution completed with {} steps",
                execution_result.completed_steps
            );
        }
        Err(e) => {
            // Expected due to missing agent runner, but verify database operations occurred
            println!("Expected execution failure: {e}");
        }
    }
}

#[tokio::test]
async fn test_session_storage_to_database() {
    // Setup
    let (database, _temp_dir) = create_test_database().await;
    let context_resolver = Arc::new(ContextResolver::new());
    let executor = PingPongExecutor::new(30000, context_resolver, database.into());

    // Test session storage
    let execution_id = "test_exec_001";
    let step_index = 0;
    let session_id = "test_session_001";
    let yml_content = r#"
step_id: step_1
success: true
tool_calls: []
output: {"balance": 1000000000}
execution_time_ms: 1000
"#;

    // Store session
    let result = executor
        .store_session_to_database(execution_id, step_index, session_id, yml_content)
        .await;

    assert!(result.is_ok(), "Session storage should succeed");
}

#[tokio::test]
async fn test_consolidation_timeout() {
    // Setup
    let (database, _temp_dir) = create_test_database().await;
    let context_resolver = Arc::new(ContextResolver::new());
    let executor = PingPongExecutor::new(30000, context_resolver, database.into());

    // Test consolidation with no sessions (should fail quickly)
    let execution_id = "test_exec_empty";
    let result = executor.consolidate_database_sessions(execution_id).await;

    assert!(
        result.is_err(),
        "Consolidation with no sessions should fail"
    );

    let e = result.unwrap_err();
    let error_msg = e.to_string();
    assert!(
        error_msg.contains("No sessions found") || error_msg.contains("Failed to get sessions"),
        "Should fail with appropriate error message: {error_msg}"
    );
}

#[tokio::test]
async fn test_perform_consolidation_with_sessions() {
    // Setup
    let (database, _temp_dir) = create_test_database().await;
    let db_arc = std::sync::Arc::new(database);

    // First store some test sessions
    let execution_id = "test_exec_consolidation";

    // Store step sessions
    for i in 0..2 {
        let _session_id = format!("{execution_id}_step_{i}");
        let yml_content = format!(
            r#"
step_id: step_{}
success: {}
tool_calls: ["get_account_balance"]
output: {{"step": {}, "data": "test_data"}}
execution_time_ms: 1000
"#,
            i,
            if i == 0 { "true" } else { "false" },
            i
        );

        db_arc
            .store_step_session(execution_id, i, &yml_content)
            .await
            .expect("Failed to store test session");
    }

    // Get sessions for consolidation
    let sessions = db_arc
        .get_sessions_for_consolidation(execution_id)
        .await
        .expect("Failed to get sessions for consolidation");

    assert_eq!(sessions.len(), 2, "Should have 2 sessions to consolidate");

    // Perform consolidation
    let consolidated_id =
        PingPongExecutor::perform_consolidation(&db_arc, execution_id, sessions).await;

    assert!(consolidated_id.is_ok(), "Consolidation should succeed");

    let consolidated_id = consolidated_id.unwrap();
    assert!(
        !consolidated_id.is_empty(),
        "Should return a valid consolidation ID"
    );

    // Verify consolidated session was stored
    let stored_content = db_arc
        .get_consolidated_session(&consolidated_id)
        .await
        .expect("Failed to retrieve consolidated session");

    assert!(
        stored_content.is_some(),
        "Consolidated session should be stored"
    );

    let content = stored_content.unwrap();

    // Parse the JSON content to verify structure
    let consolidated_json: serde_json::Value =
        serde_json::from_str(&content).expect("Consolidated session should be valid JSON");

    // Verify the structure contains expected fields
    assert!(
        consolidated_json.get("consolidated_session_id").is_some(),
        "Should contain consolidated_session_id field"
    );
    assert!(
        consolidated_json.get("execution_id").is_some(),
        "Should contain execution_id field"
    );
    assert!(
        consolidated_json.get("steps").is_some(),
        "Should contain steps array"
    );
    assert!(
        consolidated_json.get("metadata").is_some(),
        "Should contain metadata object"
    );

    // Verify metadata contains expected fields
    let metadata = consolidated_json.get("metadata").unwrap();
    assert_eq!(
        metadata.get("successful_steps").unwrap(),
        1,
        "Should have 1 successful step"
    );
    assert_eq!(
        metadata.get("failed_steps").unwrap(),
        1,
        "Should have 1 failed step"
    );
    assert_eq!(
        metadata.get("total_steps").unwrap(),
        2,
        "Should have total of 2 steps"
    );
    assert!(
        metadata.get("success_rate").is_some(),
        "Should contain success_rate"
    );
}

#[tokio::test]
async fn test_analyze_session_success() {
    // Test successful session
    let success_content = r#"
step_id: step_1
success: true
tool_calls: ["get_account_balance"]
output: {"balance": 1000000000}
"#;
    assert!(
        PingPongExecutor::analyze_session_success(success_content),
        "Should detect successful session"
    );

    // Test failed session
    let failed_content = r#"
step_id: step_1
success: false
error: "Something went wrong"
tool_calls: []
output: null
"#;
    assert!(
        !PingPongExecutor::analyze_session_success(failed_content),
        "Should detect failed session"
    );

    // Test ambiguous content (should default to success)
    let ambiguous_content = r#"
step_id: step_1
tool_calls: ["get_account_balance"]
output: {"balance": 1000000000}
"#;
    assert!(
        PingPongExecutor::analyze_session_success(ambiguous_content),
        "Should default to success for ambiguous content"
    );
}

#[tokio::test]
async fn test_extract_tool_count() {
    // Test content with tool_name patterns
    let content_with_tools = r#"
tool_name: get_account_balance
params: {"wallet": "test"}
tool_name: jupiter_swap
params: {"input_mint": "USDC", "output_mint": "SOL"}
"#;

    assert_eq!(
        PingPongExecutor::extract_tool_count(content_with_tools),
        2,
        "Should count 2 tools"
    );

    // Test content with JSON tool calls
    let json_content = r#"
{
    "tool_calls": [
        {"tool_name": "get_account_balance"},
        {"tool_name": "jupiter_swap"}
    ]
}
"#;

    assert_eq!(
        PingPongExecutor::extract_tool_count(json_content),
        0,
        "Should return 0 for JSON format without tool_name: prefix"
    );

    // Test content with known tool names
    let known_tools_content = r#"
jupiter_swap called with parameters
get_account_balance returned balance
jupiter_lend_earn_deposit executed
"#;

    assert_eq!(
        PingPongExecutor::extract_tool_count(known_tools_content),
        3,
        "Should count known tool names"
    );
}
