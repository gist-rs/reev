//! Test Single Step Chunks for Multi-Step Flow Generation
//!
//! This test validates that single-step chunks are properly defined
//! and can be combined for multi-step flows like "sell all SOL and lend to jup".

use reev_core::yml_schema::{YmlFlow, YmlStep, YmlToolCall};
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use std::collections::HashMap;


#[tokio::test]
async fn test_single_step_chunks_can_be_combined_for_multi_step_flow(
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create test wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        reev_types::benchmark::TokenBalance {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            balance: 5_000_000_000, // 5 SOL
            decimals: Some(9),
            symbol: Some("SOL".to_string()),
            formatted_amount: Some("5.0".to_string()),
            owner: Some("test_wallet".to_string()),
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 5_000_000_000, // 5 SOL
        total_value_usd: 750.0,     // $150 per SOL
        token_balances,
        token_prices: HashMap::new(), // Empty for test
    };

    // Create a multi-step flow that matches the expected structure for "sell all SOL and lend to jup"
    let flow_id = uuid::Uuid::new_v4().to_string();
    let mut multi_step_flow = YmlFlow::new(
        flow_id.clone(),
        "sell all SOL and lend to jup".to_string(),
        reev_core::yml_schema::YmlWalletInfo::new(
            wallet_context.owner.clone(),
            wallet_context.sol_balance,
        )
        .with_total_value(wallet_context.total_value_usd),
    )
    .with_refined_prompt("swap 4.99 SOL to USDC then lend to Jupiter".to_string());

    // Step 1: Swap
    let swap_step = YmlStep::new(
        format!("{}_swap", uuid::Uuid::new_v4()),
        "swap 4.99 SOL to USDC".to_string(),
        "".to_string(), // Empty context for first step
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
    .with_refined_prompt("swap 4.99 SOL to USDC".to_string());

    // Step 2: Lend with context from previous step
    let lend_step = YmlStep::new(
        format!("{}_lend", uuid::Uuid::new_v4()),
        "lend 708.58 USDC (value 4.99 SOL) to Jupiter lend".to_string(),
        "swapped 4.99 SOL to 708.58 USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
    .with_refined_prompt("lend 708.58 USDC (value 4.99 SOL) to Jupiter lend".to_string());

    // Add steps to multi-step flow
    multi_step_flow.steps.push(swap_step);
    multi_step_flow.steps.push(lend_step);

    // Write YML to file
    let yml_content = serde_yaml::to_string(&multi_step_flow)?;

    // Verify YML structure matches what we expect
    let yaml: serde_yaml::Value = serde_yaml::from_str(&yml_content)?;

    // Check flow structure
    assert_eq!(yaml.get("flow_id").unwrap().as_str().unwrap(), flow_id);
    assert_eq!(
        yaml.get("user_prompt").unwrap().as_str().unwrap(),
        "sell all SOL and lend to jup"
    );
    assert_eq!(
        yaml.get("refined_prompt").unwrap().as_str().unwrap(),
        "swap 4.99 SOL to USDC then lend to Jupiter"
    );

    // Check steps
    let steps = yaml.get("steps").unwrap().as_sequence().unwrap();
    assert_eq!(steps.len(), 2);

    // Check first step
    let step1 = steps[0].as_mapping().unwrap();
    let step1_prompt = step1.get("refined_prompt").unwrap().as_str().unwrap();
    assert!(step1_prompt.contains("swap"));
    assert!(step1_prompt.contains("4.99"));
    assert!(step1_prompt.contains("SOL"));
    assert_eq!(step1.get("context").unwrap().as_str().unwrap(), "");

    // Check second step
    let step2 = steps[1].as_mapping().unwrap();
    let step2_context = step2.get("context").unwrap().as_str().unwrap();
    let step2_prompt = step2.get("refined_prompt").unwrap().as_str().unwrap();
    assert!(step2_context.contains("swapped 4.99 SOL"));
    assert!(step2_prompt.contains("lend"));
    assert!(step2_prompt.contains("708.58"));

    println!("Generated expected YML structure for 'sell all SOL and lend to jup':");
    println!("{yml_content}");

    // Verify the structure matches the expected format:
    // ```
    // steps:
    //   - step_id: "{uuidv7}_swap"
    //     refined_prompt: "swap 4.99 SOL to USDC"
    //     context:
    //   - step_id: "{uuidv7}_lend"
    //     wallet_context: "wall context and token price go here"
    //     previous_context: "swapped 4.99 SOL to 708.58 USDC"
    //     refined_prompt: "lend 708.58 USDC (value 4.99 SOL) to Jupiter lend"
    // ```

    assert!(yml_content.contains("swap 4.99 SOL to USDC"));
    assert!(yml_content.contains("lend 708.58 USDC"));
    assert!(yml_content.contains("swapped 4.99 SOL to 708.58 USDC"));

    Ok(())
}
