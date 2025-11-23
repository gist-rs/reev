//! Tests for executor module

use reev_core::executor::Executor;
use reev_core::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;

#[tokio::test]
async fn test_execute_simple_swap_flow() {
    let executor = Executor::new().unwrap();

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
    .with_step(step)
    .with_ground_truth(ground_truth);

    // Create a mock wallet context
    let mut context = WalletContext::new("11111111111111111111111111111112".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL

    // Execute the flow
    let result = executor.execute_flow(&flow, &context).await.unwrap();

    // Verify the result
    println!(
        "Result flow_id: {}, expected flow_id: {}",
        result.flow_id, flow.flow_id
    );
    println!("Result step_results len: {}", result.step_results.len());
    if !result.step_results.is_empty() {
        println!("First step ID: {}", result.step_results[0].step_id);
    }

    assert_eq!(result.flow_id, flow.flow_id);
    assert_eq!(result.user_prompt, flow.user_prompt);
    assert!(result.success);
    assert_eq!(result.step_results.len(), 1);
    assert_eq!(result.step_results[0].step_id, "swap");
}
