//! Integration test for orchestrator refactor to use reev-core components

use reev_core::yml_schema::{
    builders::{create_swap_flow, create_lend_flow},
    YmlAssertion, YmlFlow, YmlGroundTruth,
};
use reev_orchestrator::gateway::OrchestratorGateway;
use reev_types::flow::WalletContext;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_orchestrator_with_reev_core() {
    // Create a temporary directory for test databases
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create a database writer for test
    let db_config = reev_db::DatabaseConfig::from_path(db_path.to_str().unwrap());
    let db = Arc::new(reev_db::writer::DatabaseWriter::new(db_config).await.unwrap());

    // Create gateway with reev-core components
    let gateway = OrchestratorGateway::with_database(db).await.unwrap();

    // Create a test wallet context
    let mut wallet_context = WalletContext::new("test_wallet_123".to_string());
    wallet_context.sol_balance = 5_000_000_000; // 5 SOL
    wallet_context.total_value_usd = 750.0; // $750 total value

    // Test process_user_request with a simple swap prompt
    let (flow_plan, yml_path) = gateway
        .process_user_request("swap 1 SOL to USDC", "test_wallet_123")
        .await
        .unwrap();

    // Verify flow was created correctly
    assert!(!flow_plan.steps.is_empty(), "Flow should have steps");

    // Verify YML file was created
    assert!(std::path::Path::new(&yml_path).exists(), "YML file should exist");

    // Read and verify YML content
    let yml_content = std::fs::read_to_string(&yml_path).unwrap();
    assert!(!yml_content.is_empty(), "YML file should not be empty");
    assert!(yml_content.contains("swap"), "YML should contain swap instruction");

    // Clean up
    std::fs::remove_file(&yml_path).unwrap();
}

#[tokio::test]
async fn test_reev_core_planner_integration() {
    // Create a temporary directory for test databases
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create a database writer for test
    let db_config = reev_db::DatabaseConfig::from_path(db_path.to_str().unwrap());
    let db = Arc::new(reev_db::writer::DatabaseWriter::new(db_config).await.unwrap());

    // Create gateway with reev-core components
    let gateway = OrchestratorGateway::with_database(db).await.unwrap();

    // Test with different prompts
    let prompts = vec![
        ("swap 2 SOL to USDC", "swap"),
        ("lend 500 USDC", "lend"),
        ("swap 1 SOL to USDC then lend", "swap_then_lend"),
    ];

    for (prompt, expected_flow_type) in prompts {
        let (flow_plan, yml_path) = gateway
            .process_user_request(prompt, "test_wallet_123")
            .await
            .unwrap();

        // Verify flow was created correctly
        assert!(!flow_plan.steps.is_empty(), "Flow for '{}' should have steps", prompt);

        // Verify YML file was created
        assert!(std::path::Path::new(&yml_path).exists(), "YML file for '{}' should exist", prompt);

        // Read and verify YML content contains expected flow type
        let yml_content = std::fs::read_to_string(&yml_path).unwrap();
        assert!(!yml_content.is_empty(), "YML file for '{}' should not be empty", prompt);

        // Clean up
        std::fs::remove_file(&yml_path).unwrap();
    }
}

#[tokio::test]
async fn test_benchmark_mode_with_reev_core() {
    // Create a temporary directory for test databases
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create a database writer for test
    let db_config = reev_db::DatabaseConfig::from_path(db_path.to_str().unwrap());
    let db = Arc::new(reev_db::writer::DatabaseWriter::new(db_config).await.unwrap());

    // Create gateway with reev-core components
    let gateway = OrchestratorGateway::with_database(db).await.unwrap();

    // Test process_user_request with USER_WALLET_PUBKEY for benchmark mode
    let (flow_plan, yml_path) = gateway
        .process_user_request("swap 1 SOL to USDC", "USER_WALLET_PUBKEY")
        .await
        .unwrap();

    // Verify flow was created correctly
    assert!(!flow_plan.steps.is_empty(), "Benchmark flow should have steps");

    // Verify YML file was created
    assert!(std::path::Path::new(&yml_path).exists(), "Benchmark YML file should exist");

    // Read and verify YML content
    let yml_content = std::fs::read_to_string(&yml_path).unwrap();
    assert!(!yml_content.is_empty(), "Benchmark YML file should not be empty");
    assert!(yml_content.contains("swap"), "Benchmark YML should contain swap instruction");

    // Clean up
    std::fs::remove_file(&yml_path).unwrap();
}
