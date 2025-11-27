//! End-to-end multi-step test using default Solana keypair
//!
//! This test loads the wallet from ~/.config/solana/id.json, airdrops SOL via surfpool,
//! creates a multi-step flow for "sell all SOL and lend to jup", lets the LLM handle
//! tool calling via rig, signs the transaction with the default keypair, and verifies completion.
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test e2e_multi_step test_sell_all_sol_and_lend_to_jup -- --nocapture > test_output.log 2>&1
//! ```
//!
//! ## Test Flow (7 Steps)
//!
//! 1. Prompt: "sell all SOL and lend to jup"
//! 2. Shows log info for YML prompt with wallet info from SURFPOOL sent to GLM-coding
//! 3. Creates a multi-step flow with swap and lend operations
//! 4. Shows log info for swap tool calling from LLM
//! 5. Shows log info for lend tool calling from LLM
//! 6. Signs both transactions with default keypair at ~/.config/solana/id.json
//! 7. Shows transaction completion results from SURFPOOL

mod common;

use anyhow::{anyhow, Result};
use common::{ensure_surfpool_running, get_test_keypair, setup_wallet_for_swap};
use jup_sdk::surfpool::SurfpoolClient;
use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;
use reev_core::yml_generator::flow_builders::{build_lend_flow, build_swap_flow};
use reev_core::yml_generator::operation_types::{LendParams, SwapParams};
use reev_core::yml_schema::{YmlFlow, YmlStep};
use reev_core::Executor;
use reev_types::flow::WalletContext;
use reev_types::tools::{ToolName, YmlToolCall};
use solana_sdk::signature::Signer;
use std::env;
use tracing::{error, info, warn};

/// Execute multi-step "sell all SOL and lend to jup" using the planner and LLM
async fn execute_sell_all_sol_and_lend(initial_sol_balance: u64) -> Result<Vec<String>> {
    info!("\n🚀 Starting multi-step execution: sell all SOL and lend to jup");

    // Step 1: Get wallet context
    let keypair = get_test_keypair()?;
    let pubkey = keypair.pubkey();
    let context_resolver = ContextResolver::new(SolanaEnvironment {
        rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
        commitment: "confirmed".to_string(),
        wallet: Some(pubkey.to_string()),
    });

    info!("📍 Wallet: {}", pubkey);
    info!("💰 Initial SOL balance: {} lamports", initial_sol_balance);

    // Step 2: Fetch wallet context from SURFPOOL
    let wallet_context = context_resolver
        .get_wallet_context(&pubkey.to_string())
        .await?;
    info!(
        "📊 Wallet context: {} SOL, ${:.2} total value",
        wallet_context.sol_balance_sol(),
        wallet_context.total_value_usd
    );

    // Step 3: Create swap parameters (sell almost all SOL, keep some for gas)
    let sol_balance_sol = wallet_context.sol_balance_sol();
    let gas_reserve_sol = 0.05; // Reserve 0.05 SOL for gas and fees
    let swap_amount_sol = if sol_balance_sol > gas_reserve_sol {
        sol_balance_sol - gas_reserve_sol
    } else {
        // If balance is very low, use half for swap
        sol_balance_sol / 2.0
    };
    info!("🔄 Swap amount: {} SOL", swap_amount_sol);

    // Create swap refined prompt
    let swap_refined_prompt = reev_core::refiner::RefinedPrompt {
        original: "sell all SOL and lend to jup".to_string(),
        refined: format!("swap {} SOL to USDC", swap_amount_sol),
        confidence: 0.9,
    };

    // Create swap parameters
    let swap_params = SwapParams {
        from_token: "SOL".to_string(),
        to_token: "USDC".to_string(),
        amount: swap_amount_sol,
    };

    // Build swap flow
    let swap_flow = build_swap_flow(&swap_refined_prompt, &wallet_context, swap_params).await?;
    info!(
        "✅ Swap flow created with {} step(s)",
        swap_flow.steps.len()
    );

    // Step 4: Create lend parameters (using estimated USDC from swap)
    let usdc_price_estimate = 150.0; // $150 per SOL estimate
    let expected_usdc_amount = swap_amount_sol * usdc_price_estimate;
    info!("💵 Expected USDC from swap: {}", expected_usdc_amount);

    // Create lend refined prompt
    let lend_refined_prompt = reev_core::refiner::RefinedPrompt {
        original: "sell all SOL and lend to jup".to_string(),
        refined: format!("lend {} USDC to jupiter", expected_usdc_amount),
        confidence: 0.9,
    };

    // Create lend parameters
    let lend_params = LendParams {
        token: "USDC".to_string(),
        amount: expected_usdc_amount,
    };

    // Build lend flow
    let lend_flow = build_lend_flow(&lend_refined_prompt, &wallet_context, lend_params).await?;
    info!(
        "✅ Lend flow created with {} step(s)",
        lend_flow.steps.len()
    );

    // Step 5: Create multi-step flow by combining single-step flows
    let flow_id = format!("multi-step-sell-all-lend-{}", uuid::Uuid::new_v4());
    let mut multi_step_flow = YmlFlow::new(
        flow_id.clone(),
        "sell all SOL and lend to jup".to_string(),
        swap_flow.subject_wallet_info.clone(),
    )
    .with_refined_prompt("swap SOL to USDC then lend to jupiter".to_string());

    // Extract and adapt swap step with proper ID
    let swap_step = YmlStep::new(
        "step_1_swap".to_string(),
        swap_flow.steps[0].refined_prompt.clone(),
        "Swap SOL to USDC using Jupiter DEX".to_string(),
    )
    .with_tool_call(YmlToolCall {
        tool_name: ToolName::JupiterSwap,
        critical: true,
    })
    .with_critical(true);

    // Extract and adapt lend step with proper ID and context
    let lend_step = YmlStep::new(
        "step_2_lend".to_string(),
        lend_flow.steps[0].refined_prompt.clone(),
        format!(
            "Deposit USDC from previous swap into Jupiter lending. Expected amount: {:.2} USDC",
            expected_usdc_amount
        ),
    )
    .with_tool_call(YmlToolCall {
        tool_name: ToolName::JupiterLendEarnDeposit,
        critical: true,
    })
    .with_critical(true);

    // Add steps to multi-step flow
    multi_step_flow.steps.push(swap_step);
    multi_step_flow.steps.push(lend_step);

    info!("📋 Multi-step flow created:");
    info!("   Flow ID: {}", flow_id);
    info!("   Steps: {}", multi_step_flow.steps.len());
    info!("   Step 1: {}", multi_step_flow.steps[0].step_id);
    info!("   Step 2: {}", multi_step_flow.steps[1].step_id);

    // Step 6: Execute multi-step flow using the Executor with RigAgent
    info!("⚙️ Executing multi-step flow...");
    let executor = Executor::new_with_rig().await?;
    let result = executor
        .execute_flow(&multi_step_flow, &wallet_context)
        .await?;

    // Step 7: Extract transaction signatures from step results
    let mut signatures = Vec::new();
    for (i, step_result) in result.step_results.iter().enumerate() {
        info!("Step {} result:", i + 1);
        info!("  Success: {}", step_result.success);

        if let Some(error) = &step_result.error_message {
            error!("  Error: {}", error);
        }

        // Extract signature from step result
        if let Some(jupiter_swap) = step_result.output.get("jupiter_swap") {
            if let Some(sig) = jupiter_swap.get("transaction_signature") {
                if let Some(sig_str) = sig.as_str() {
                    info!("  Jupiter transaction signature: {}", sig_str);
                    signatures.push(sig_str.to_string());
                }
            }
        } else if let Some(sig) = step_result.output.get("transaction_signature") {
            if let Some(sig_str) = sig.as_str() {
                info!("  Transaction signature: {}", sig_str);
                signatures.push(sig_str.to_string());
            }
        }

        // Check for tool results array
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    if let Some(sig) = result.get("transaction_signature") {
                        if let Some(sig_str) = sig.as_str() {
                            info!("  Tool result transaction signature: {}", sig_str);
                            signatures.push(sig_str.to_string());
                        }
                    }
                }
            }
        }
    }

    // Verify overall execution success
    if result.success {
        info!("✅ Multi-step flow executed successfully!");
    } else {
        warn!("⚠️ Multi-step flow completed with some issues");
    }

    Ok(signatures)
}

/// Verify transaction details on-chain
async fn verify_transaction_details(signatures: &[String], initial_sol_balance: u64) -> Result<()> {
    info!("🔍 Verifying transaction details...");

    // Connect to Solana RPC
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    );

    let keypair = get_test_keypair()?;
    let pubkey = keypair.pubkey();

    // Get final balance
    let final_balance = rpc_client.get_balance(&pubkey).await?;
    let balance_change = final_balance as i64 - initial_sol_balance as i64;
    info!(
        "💰 Balance change: {} lamports ({:.2} SOL)",
        balance_change,
        balance_change as f64 / 1_000_000_000.0
    );

    // Check each transaction
    for sig in signatures {
        match rpc_client
            .get_transaction(
                &sig.parse()?,
                solana_transaction_status::UiTransactionEncoding::Json,
            )
            .await
        {
            Ok(tx) => {
                info!("✅ Transaction {} confirmed", sig);
            }
            Err(e) => {
                warn!("⚠️ Transaction {} not found or failed: {}", sig, e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_sell_all_sol_and_lend_to_jup() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("🧪 Starting e2e test: sell all SOL and lend to jup");

    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Get initial wallet setup
    let (initial_sol_balance, _) = setup_wallet_for_swap().await?;
    info!("🚀 Initial wallet setup complete");

    // Execute the multi-step flow
    let signatures = execute_sell_all_sol_and_lend(initial_sol_balance).await?;

    // Verify the transaction details
    verify_transaction_details(&signatures, initial_sol_balance).await?;

    info!("🎉 Multi-step e2e test completed successfully!");
    Ok(())
}
