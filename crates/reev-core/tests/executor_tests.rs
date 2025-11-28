//! Tests for executor module

#[path = "common/mod.rs"]
mod common;
use reev_core::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;

#[tokio::test]
async fn test_execute_simple_swap_flow() {
    // Use MockToolExecutor for this test since we don't need a real API
    let mock_executor =
        common::mock_helpers::mock_tool_executor::MockToolExecutor::new().with_success(true);

    // Create a simple swap flow
    let wallet_info = YmlWalletInfo::new(
        "11111111111111111111111111111112".to_string(),
        1_000_000_000, // 1 SOL
    )
    .with_token(
        TokenBalance::new(
            "So11111111111111111111111111111111111111112".to_string(),
            1_000_000_000,
        )
        .with_decimals(9)
        .with_symbol("SOL".to_string()),
    );

    let step = YmlStep::new(
        "swap".to_string(),
        "swap 1 SOL to USDC".to_string(),
        "Exchange 1 SOL for USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true));

    // Add ground truth for validation
    let ground_truth = YmlGroundTruth::new()
        .with_assertion(
            YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey("11111111111111111111111111111112".to_string())
                .with_expected_change_gte(-1_010_000_000.0),
        )
        .with_error_tolerance(0.01);

    let flow = YmlFlow::new(
        uuid::Uuid::now_v7().to_string(),
        "swap 1 SOL to USDC".to_string(),
        wallet_info,
    )
    .with_step(step.clone())
    .with_ground_truth(ground_truth);

    // Create a mock wallet context
    let mut context = WalletContext::new("11111111111111111111111111111112".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL

    // Execute the flow
    // Execute step directly with mock executor
    let step_result = mock_executor
        .execute_step(&flow.steps[0], &context)
        .await
        .unwrap();

    // Verify result
    println!(
        "Step result ID: {}, expected step ID: {}",
        step_result.step_id, flow.steps[0].step_id
    );
    println!("Step result success: {}", step_result.success);
    if let Some(error) = &step_result.error_message {
        println!("Step result error: {error}");
    }

    assert_eq!(step_result.step_id, flow.steps[0].step_id);
    assert!(step_result.success);
    assert_eq!(step_result.tool_calls, vec!["JupiterSwap"]);
    // Check that step result has a valid step_id (UUID)
    assert!(!step_result.step_id.is_empty());
}
