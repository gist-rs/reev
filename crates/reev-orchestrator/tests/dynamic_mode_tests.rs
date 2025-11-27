//! Tests for dynamic_mode module

use reev_orchestrator::gateway::OrchestratorGateway;
use reev_types::flow::WalletContext;
use serial_test::serial;
use std::io::Write;
use tempfile::NamedTempFile;

#[tokio::test]
#[serial]
async fn test_should_use_database_flow() {
    // Set test mode to avoid requiring ZAI_API_KEY
    std::env::set_var("REEV_TEST_MODE", "true");
    // Use test method that doesn't require ZAI_API_KEY
    let gateway = OrchestratorGateway::new_for_test(None).await.unwrap();

    // Test 1: Dynamic flow should use database
    let dynamic_yml = r#"
flow_id: "test_dynamic"
flow_type: "dynamic"
user_prompt: "Test dynamic flow"
description: "Test dynamic flow"
steps:
  - step_id: "step1"
    agent: "test_agent"
    prompt_template: "Test prompt"
    description: "Test step description"
    required_tools: []
    estimated_time_seconds: 30
    critical: true
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(dynamic_yml.as_bytes()).unwrap();
    let dynamic_path = temp_file.path().to_path_buf();

    let should_use_db = gateway
        .should_use_database_flow(&dynamic_path)
        .await
        .unwrap();
    assert!(should_use_db, "Dynamic flow should use database");

    // Test 2: Static flow should use file-based
    let static_yml = r#"
flow_id: "test_static"
user_prompt: "Test static flow"
description: "Test static flow"
steps:
  - step_id: "step1"
    agent: "test_agent"
    prompt_template: "Test prompt"
    description: "Test step description"
    required_tools: []
    estimated_time_seconds: 30
    critical: true
"#;

    let mut temp_file2 = NamedTempFile::new().unwrap();
    temp_file2.write_all(static_yml.as_bytes()).unwrap();
    let static_path = temp_file2.path().to_path_buf();

    let should_use_db = gateway
        .should_use_database_flow(&static_path)
        .await
        .unwrap();
    assert!(!should_use_db, "Static flow should use file-based");
}

#[tokio::test]
#[serial]
async fn test_dynamic_flow_with_database_routing() {
    // This test verifies that the database routing logic works
    // Full end-to-end test would require actual agent execution
    // Set test mode to avoid requiring ZAI_API_KEY
    std::env::set_var("REEV_TEST_MODE", "true");
    // Use test method that doesn't require ZAI_API_KEY
    let gateway = OrchestratorGateway::new_for_test(None).await.unwrap();

    let dynamic_yml = r#"
flow_id: "test_dynamic_flow"
flow_type: "dynamic"
user_prompt: "Test dynamic flow for database routing"
description: "Test dynamic flow for database routing"
atomic_mode: "Strict"
steps:
  - step_id: "balance_check"
    agent: "glm-4.6-coding"
    prompt_template: "Get account balance"
    description: "Check account balance"
    required_tools: ["GetAccountBalance"]
    estimated_time_seconds: 10
    critical: true
initial_state:
  - name: "wallet_address"
    value: "test_wallet_address"
context:
  owner: "test_wallet"
  sol_balance: 1000000000
  token_balances: {}
  token_prices: {}
  total_value_usd: 100.0
metadata:
  created_at: "2024-01-01T00:00:00Z"
  category: "test"
  complexity_score: 1
  tags: ["test"]
  version: "1.0"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(dynamic_yml.as_bytes()).unwrap();
    let yml_path = temp_file.path().to_path_buf();

    // Verify routing decision works
    let should_use_db = gateway.should_use_database_flow(&yml_path).await.unwrap();
    assert!(
        should_use_db,
        "Dynamic flow with flow_type should use database"
    );
}

#[test]
fn test_amount_extraction() {
    assert_eq!(
        reev_orchestrator::dynamic_mode::extract_amount("swap 1.5 SOL to USDC"),
        "1.5 SOL"
    );
    assert_eq!(
        reev_orchestrator::dynamic_mode::extract_amount("use 50% of SOL"),
        "50%"
    );
    assert_eq!(
        reev_orchestrator::dynamic_mode::extract_amount("lend 100 USDC"),
        "100 USDC"
    );
    assert_eq!(
        reev_orchestrator::dynamic_mode::extract_amount("check balance"),
        ""
    );
}

#[test]
fn test_request_validation() {
    let mut context = WalletContext::new("test_wallet".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL in lamports

    // Valid request
    assert!(reev_orchestrator::dynamic_mode::validate_user_request("swap 1 SOL", &context).is_ok());

    // Empty request
    assert!(reev_orchestrator::dynamic_mode::validate_user_request("", &context).is_err());

    // Too long request
    let long_prompt = "a".repeat(1001);
    assert!(
        reev_orchestrator::dynamic_mode::validate_user_request(&long_prompt, &context).is_err()
    );

    // Blocked keyword
    assert!(
        reev_orchestrator::dynamic_mode::validate_user_request("drain wallet", &context).is_err()
    );
}

#[test]
fn test_empty_wallet_validation() {
    let empty_context = WalletContext::new("empty_wallet".to_string());

    // Empty wallet should fail
    assert!(
        reev_orchestrator::dynamic_mode::validate_user_request("swap 1 SOL", &empty_context)
            .is_err()
    );
}
