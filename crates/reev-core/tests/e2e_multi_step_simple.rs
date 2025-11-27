//! End-to-end multi-step flow creation test
//!
//! This test validates that our system can properly create a multi-step flow
//! for "sell all SOL and lend to jup" by manually combining single-step flows.

mod common;

use anyhow::Result;
use common::{ensure_surfpool_running, get_test_keypair, setup_wallet_for_swap};
use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::yml_schema::{YmlFlow, YmlStep, YmlToolCall, YmlWalletInfo};
use reev_types::tools::ToolName;
use solana_sdk::signature::Signer;
use tracing::{info, warn};

/// Create a multi-step flow for "sell all SOL and lend to jup"
async fn create_sell_all_sol_and_lend_flow() -> Result<YmlFlow> {
    info!("\n🚀 Creating multi-step flow: sell all SOL and lend to jup");

    // Step 1: Get wallet keypair and context
    let keypair = get_test_keypair()?;
    let pubkey = keypair.pubkey();

    // Set up context resolver
    let context_resolver = ContextResolver::new(SolanaEnvironment {
        rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
    });

    info!("📍 Wallet: {}", pubkey);

    // Resolve wallet context
    let wallet_context = context_resolver
        .resolve_wallet_context(&pubkey.to_string())
        .await?;

    info!(
        "📊 Wallet context: {} SOL, ${:.2} total value",
        wallet_context.sol_balance_sol(),
        wallet_context.total_value_usd
    );

    // Step 2: Create swap step (first operation in multi-step flow)
    let sol_balance_sol = wallet_context.sol_balance_sol();
    let gas_reserve_sol = 0.05; // Reserve 0.05 SOL for gas and fees
    let swap_amount_sol = if sol_balance_sol > gas_reserve_sol {
        sol_balance_sol - gas_reserve_sol
    } else {
        // If balance is very low, use half for swap
        sol_balance_sol / 2.0
    };
    info!("🔄 Swap amount: {} SOL", swap_amount_sol);

    // Create swap step
    let swap_step = YmlStep::new(
        "step_1_swap".to_string(),
        format!("swap {} SOL to USDC", swap_amount_sol),
        "Swap SOL to USDC using Jupiter DEX".to_string(),
    )
    .with_tool_call(YmlToolCall {
        tool_name: ToolName::JupiterSwap,
        critical: true,
        expected_parameters: None,
    })
    .with_critical(true);

    // Step 3: Create lend step (second operation in multi-step flow)
    let usdc_price_estimate = 150.0; // $150 per SOL estimate
    let expected_usdc_amount = swap_amount_sol * usdc_price_estimate;
    info!("💵 Expected USDC from swap: {}", expected_usdc_amount);

    // Create lend step
    let lend_step = YmlStep::new(
        "step_2_lend".to_string(),
        format!("lend {} USDC to jupiter", expected_usdc_amount),
        format!(
            "Deposit USDC from previous swap into Jupiter lending. Expected amount: {:.2} USDC",
            expected_usdc_amount
        ),
    )
    .with_tool_call(YmlToolCall {
        tool_name: ToolName::JupiterLendEarnDeposit,
        critical: true,
        expected_parameters: None,
    })
    .with_critical(true);

    // Step 4: Create a multi-step flow with both steps
    let flow_id = format!("multi-step-sell-all-lend-{}", uuid::Uuid::new_v4());
    let mut multi_step_flow = YmlFlow::new(
        flow_id.clone(),
        "sell all SOL and lend to jup".to_string(),
        YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
            .with_total_value(wallet_context.total_value_usd),
    )
    .with_refined_prompt("swap SOL to USDC then lend to jupiter".to_string());

    // Add tokens to wallet info
    let mut wallet_info = multi_step_flow.subject_wallet_info.clone();
    for token in wallet_context.token_balances.values() {
        wallet_info = wallet_info.with_token(token.clone());
    }
    multi_step_flow.subject_wallet_info = wallet_info;

    // Add both steps to multi-step flow
    multi_step_flow.steps.push(swap_step);
    multi_step_flow.steps.push(lend_step);

    info!("📋 Multi-step flow created:");
    info!("   Flow ID: {}", flow_id);
    info!("   Steps: {}", multi_step_flow.steps.len());
    for (i, step) in multi_step_flow.steps.iter().enumerate() {
        info!("   Step {}: {}", i + 1, step.step_id);
    }

    // Verify multi-step flow structure
    assert_eq!(multi_step_flow.steps.len(), 2);
    assert_eq!(multi_step_flow.user_prompt, "sell all SOL and lend to jup");
    assert_eq!(
        multi_step_flow.refined_prompt,
        "swap SOL to USDC then lend to jupiter"
    );

    // Verify first step (swap)
    let first_step = &multi_step_flow.steps[0];
    assert!(first_step.step_id.contains("swap"));
    assert!(first_step.refined_prompt.contains("swap"));
    assert!(first_step
        .refined_prompt
        .contains(&format!("{}", swap_amount_sol)));
    assert!(first_step.expected_tool_calls.is_some());

    // Verify second step (lend)
    let second_step = &multi_step_flow.steps[1];
    assert!(second_step.step_id.contains("lend"));
    assert!(second_step.refined_prompt.contains("lend"));
    assert!(second_step
        .refined_prompt
        .contains(&format!("{}", expected_usdc_amount as i64)));
    assert!(second_step.expected_tool_calls.is_some());

    info!("✅ Multi-step flow validation passed!");
    Ok(multi_step_flow)
}

#[tokio::test]
async fn test_multi_step_flow_creation() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("🧪 Starting multi-step flow creation test");

    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Get initial wallet setup
    let keypair = get_test_keypair()?;
    let pubkey = keypair.pubkey();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    let (initial_sol_balance, _) = setup_wallet_for_swap(&pubkey, &surfpool_client).await?;
    info!("🚀 Initial wallet setup complete");

    // Create multi-step flow
    let multi_step_flow = create_sell_all_sol_and_lend_flow().await?;

    // Serialize to YAML for inspection
    let yml_content = serde_yaml::to_string(&multi_step_flow)?;
    info!("📄 Generated YML:");
    info!("{}", yml_content);

    // Verify it's valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yml_content)?;
    assert!(parsed.is_mapping());

    info!("✅ Multi-step flow creation test completed successfully!");
    Ok(())
}
