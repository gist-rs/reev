use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use reev_lib::constants::{usdc_mint, EIGHT_PERCENT, SOL_SWAP_AMOUNT};
use reev_protocols::jupiter::swap::handle_jupiter_swap;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr, time::Instant};
use tracing::{error, info};

// Import enhanced OTEL logging macros
use reev_flow::{log_tool_call, log_tool_completion};

/// Handles the deterministic logic for the `100-JUP-SWAP-SOL-USDC` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter swap handler.
/// This handler performs two critical functions:
/// 1. It calls the public Jupiter API to get the best swap route, which often
///    includes setup, swap, and cleanup instructions.
/// 2. It discovers all accounts required for the full transaction and pre-loads
///    them from mainnet into the local `surfpool` fork, preventing missing account errors.
///
/// The agent returns the complete `Vec<RawInstruction>` required for the swap,
/// acknowledging that modern DeFi transactions often require multiple instructions.
pub(crate) async fn handle_jup_swap_sol_usdc(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let start_time = Instant::now();
    info!("[reev-agent] Matched '100-jup-swap-sol-usdc' id. Calling centralized Jupiter swap handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let input_mint = native_mint::ID;
    let output_mint = usdc_mint();
    let amount = SOL_SWAP_AMOUNT; // 0.1 SOL
    let slippage_bps = EIGHT_PERCENT; // 8%

    // 🎯 Add enhanced logging for deterministic agents
    let args = serde_json::json!({
        "user_pubkey": user_pubkey,
        "input_mint": input_mint,
        "output_mint": output_mint,
        "amount": amount,
        "slippage_bps": slippage_bps
    });
    log_tool_call!("deterministic_jupiter_swap", &args);

    // Execute tool logic with inline error handling
    let result = async {
        // The handler performs account pre-loading and returns the complete set of
        // instructions (setup, swap, cleanup) needed for the transaction.
        handle_jupiter_swap(user_pubkey, input_mint, output_mint, amount, slippage_bps).await
    }
    .await;

    match result {
        Ok(instructions) => {
            let execution_time = start_time.elapsed().as_millis() as u64;

            // 🎯 Add enhanced logging at SUCCESS
            let result_data = serde_json::json!({
                "instruction_count": instructions.len(),
                "user_pubkey": user_pubkey,
                "input_mint": input_mint,
                "output_mint": output_mint,
                "amount": amount,
                "slippage_bps": slippage_bps
            });
            log_tool_completion!(
                "deterministic_jupiter_swap",
                execution_time,
                &result_data,
                true
            );

            info!(
                "[reev-agent] Successfully received {} instructions. Responding to runner.",
                instructions.len()
            );
            Ok(instructions)
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            let error_data = serde_json::json!({
                "error": e.to_string(),
                "user_pubkey": user_pubkey,
                "input_mint": input_mint,
                "output_mint": output_mint,
                "amount": amount,
                "slippage_bps": slippage_bps
            });

            // 🎯 Add enhanced logging at ERROR
            log_tool_completion!(
                "deterministic_jupiter_swap",
                execution_time,
                &error_data,
                false
            );

            error!(
                "[deterministic_jupiter_swap] Tool execution failed in {}ms: {}",
                execution_time, e
            );
            Err(e)
        }
    }
}
