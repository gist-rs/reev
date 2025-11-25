use anyhow::Result;
// std::collections::HashMap is imported through reev_types::flow

use reev_core::execution::rig_agent::RigAgent;
use reev_core::yml_schema::YmlStep;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;

// Helper function to create a mock wallet context for tests
fn create_mock_wallet_context() -> WalletContext {
    let mut token_balances_map = std::collections::HashMap::new();
    token_balances_map.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        reev_types::benchmark::TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            balance: 20000000,
            decimals: Some(6),
            symbol: Some("USDC".to_string()),
            formatted_amount: Some("20".to_string()),
            owner: Some("5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh".to_string()),
        },
    );

    let mut token_prices = std::collections::HashMap::new();
    token_prices.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        1.0,
    );

    WalletContext {
        owner: "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh".to_string(),
        sol_balance: 4000000000,
        token_balances: token_balances_map,
        token_prices,
        total_value_usd: 6200.0, // 4 SOL + 20 USDC
    }
}

// Helper function to create a mock YML step for tests
fn create_mock_step(
    step_id: &str,
    prompt: &str,
    refined_prompt: &str,
    context: &str,
    expected_tools: Option<Vec<ToolName>>,
) -> YmlStep {
    YmlStep {
        step_id: step_id.to_string(),
        refined_prompt: refined_prompt.to_string(),
        prompt: prompt.to_string(),
        context: context.to_string(),
        expected_tool_calls: None,
        expected_tools,
        critical: Some(true),
        estimated_time_seconds: Some(10),
    }
}

#[tokio::test]
async fn test_rig_agent_handles_invalid_api_key() -> Result<()> {
    // Test that RigAgent fails gracefully with invalid API key
    let api_key = Some("invalid_api_key".to_string());
    let model_name = Some("gpt-3.5-turbo".to_string());

    let result = RigAgent::new(api_key, model_name).await;
    assert!(result.is_err());

    // Verify the error message contains relevant information
    let error_msg = match result {
        Ok(_) => "Unexpected success".to_string(),
        Err(e) => e.to_string(),
    };
    assert!(error_msg.contains("API key") || error_msg.contains("rig"));

    Ok(())
}

#[tokio::test]
async fn test_rig_agent_handles_missing_api_key() -> Result<()> {
    // Test that RigAgent fails gracefully with missing API key
    let result = RigAgent::new(None, None).await;
    assert!(result.is_err());

    // Verify the error message contains relevant information
    let error_msg = match result {
        Ok(_) => "Unexpected success".to_string(),
        Err(e) => e.to_string(),
    };
    assert!(error_msg.contains("API key") || error_msg.contains("required"));

    Ok(())
}

#[tokio::test]
async fn test_rig_agent_step_execution_structure() -> Result<()> {
    // This test verifies the structure of step execution without requiring valid API

    // We'll create a mock RigAgent that we can't actually use
    // but we can test the surrounding logic
    let wallet_context = create_mock_wallet_context();

    // Create a mock step
    let step = create_mock_step(
        "transfer_1",
        "transfer 1 SOL to address gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
        "transfer 1 SOL to address gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
        "User wants to transfer 1 SOL to the specified recipient",
        Some(vec![ToolName::SolTransfer]),
    );

    // Verify step structure
    assert_eq!(step.step_id, "transfer_1");
    assert_eq!(
        step.refined_prompt,
        "transfer 1 SOL to address gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    );
    assert_eq!(step.expected_tools, Some(vec![ToolName::SolTransfer]));
    assert_eq!(step.critical, Some(true));
    assert_eq!(step.estimated_time_seconds, Some(10));

    // Verify wallet context structure
    assert_eq!(
        wallet_context.owner,
        "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh"
    );
    assert_eq!(wallet_context.sol_balance, 4000000000);
    assert_eq!(wallet_context.total_value_usd, 6200.0);

    Ok(())
}

#[test]
fn test_tool_name_display() -> Result<()> {
    // Test that ToolName implements Display correctly
    let sol_transfer = ToolName::SolTransfer;
    let jupiter_swap = ToolName::JupiterSwap;
    let jupiter_lend = ToolName::JupiterLendEarnDeposit;

    assert_eq!(sol_transfer.to_string(), "SolTransfer");
    assert_eq!(jupiter_swap.to_string(), "JupiterSwap");
    assert_eq!(jupiter_lend.to_string(), "JupiterLendEarnDeposit");

    Ok(())
}

#[test]
fn test_wallet_context_structure() -> Result<()> {
    // Test WalletContext structure and methods
    let mut wallet_context = WalletContext::new("test_owner".to_string());

    // Verify initial state
    assert_eq!(wallet_context.owner, "test_owner");
    assert_eq!(wallet_context.sol_balance, 0);
    assert_eq!(wallet_context.total_value_usd, 0.0);
    assert!(wallet_context.token_balances.is_empty());
    assert!(wallet_context.token_prices.is_empty());

    // Add some data
    wallet_context.sol_balance = 1000000000; // 1 SOL
    wallet_context.add_token_price(
        "So11111111111111111111111111111111111111112".to_string(),
        150.0,
    );

    // Test methods
    assert_eq!(wallet_context.sol_balance_sol(), 1.0);
    assert_eq!(
        wallet_context.get_token_price("So11111111111111111111111111111111111111112"),
        Some(150.0)
    );

    // Calculate total value
    wallet_context.calculate_total_value();
    assert_eq!(wallet_context.total_value_usd, 150.0);

    Ok(())
}

#[test]
fn test_yml_step_builder_methods() -> Result<()> {
    // Test YmlStep builder methods
    let step = YmlStep::new(
        "test_step".to_string(),
        "Original prompt".to_string(),
        "Step context".to_string(),
    )
    .with_refined_prompt("Refined prompt".to_string())
    .with_expected_tools(vec![ToolName::SolTransfer])
    .with_critical(true)
    .with_estimated_time(30);

    assert_eq!(step.step_id, "test_step");
    assert_eq!(step.prompt, "Original prompt");
    assert_eq!(step.refined_prompt, "Refined prompt");
    assert_eq!(step.context, "Step context");
    assert_eq!(step.expected_tools, Some(vec![ToolName::SolTransfer]));
    assert_eq!(step.critical, Some(true));
    assert_eq!(step.estimated_time_seconds, Some(30));

    Ok(())
}

#[test]
fn test_workflow_types() -> Result<()> {
    // Test that workflow-related types are correctly defined

    // Test AtomicMode
    use reev_types::flow::AtomicMode;

    assert_eq!(AtomicMode::Strict.as_str(), "strict");
    assert_eq!(AtomicMode::Lenient.as_str(), "lenient");
    assert_eq!(AtomicMode::Conditional.as_str(), "conditional");

    // Test BenchmarkSource
    use reev_types::flow::BenchmarkSource;

    let static_source = BenchmarkSource::StaticFile {
        path: "test.yml".to_string(),
    };
    let dynamic_source = BenchmarkSource::DynamicFlow {
        prompt: "transfer SOL".to_string(),
        wallet: "test_wallet".to_string(),
    };

    assert_eq!(static_source.get_prompt(), None);
    assert_eq!(static_source.get_wallet(), None);
    assert!(!static_source.is_dynamic());

    assert_eq!(dynamic_source.get_prompt(), Some("transfer SOL"));
    assert_eq!(dynamic_source.get_wallet(), Some("test_wallet"));
    assert!(dynamic_source.is_dynamic());

    Ok(())
}
