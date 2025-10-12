use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use crate::protocols::jupiter::swap::handle_jupiter_swap;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use reev_lib::constants::{usdc_mint, FIVE_PERCENT, SOL_SWAP_AMOUNT, USDC_LEND_AMOUNT_LARGE};
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `200-JUP-SWAP-THEN-LEND-DEPOSIT` flow benchmark.
///
/// This agent orchestrates a two-step DeFi workflow:
/// 1. Swap 0.5 SOL to USDC using Jupiter with the best rate
/// 2. Deposit all received USDC into Jupiter lending to start earning yield
///
/// The agent acts as an oracle by calling the centralized Jupiter handlers for both operations.
/// It returns the complete set of instructions needed for the entire flow.
#[allow(dead_code)] // Flow system handles this benchmark through individual steps
pub(crate) async fn handle_jup_swap_then_lend_deposit(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '200-jup-swap-then-lend-deposit' id. Starting deterministic flow.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    // Step 1: Swap 0.5 SOL to USDC using Jupiter
    info!("[reev-agent] Step 1: Swapping 0.5 SOL to USDC");
    let input_mint = native_mint::ID;
    let output_mint = usdc_mint();
    let swap_amount = SOL_SWAP_AMOUNT; // 0.1 SOL
    let slippage_bps = FIVE_PERCENT; // 5%

    let swap_instructions = handle_jupiter_swap(
        user_pubkey,
        input_mint,
        output_mint,
        swap_amount,
        slippage_bps,
    )
    .await?;

    info!(
        "[reev-agent] Step 1 completed: {} swap instructions generated",
        swap_instructions.len()
    );

    // Step 2: Deposit received USDC into Jupiter lending
    info!("[reev-agent] Step 2: Depositing USDC into Jupiter lending");

    // For lending, we use the USDC mint and deposit the expected amount from the swap
    // Note: In a real scenario, we'd calculate the exact amount received from the swap
    // For deterministic purposes, we estimate ~0.5 SOL worth of USDC (accounting for slippage)
    let deposit_amount = USDC_LEND_AMOUNT_LARGE; // ~9 USDC (accounting for slippage and fees)
    let usdc_mint = usdc_mint();

    let lend_instructions =
        handle_jupiter_lend_deposit(user_pubkey, usdc_mint, deposit_amount).await?;

    info!(
        "[reev-agent] Step 2 completed: {} lending instructions generated",
        lend_instructions.len()
    );

    // Combine all instructions for the complete flow
    let mut all_instructions = Vec::new();
    all_instructions.extend(swap_instructions);
    all_instructions.extend(lend_instructions);

    info!(
        "[reev-agent] Flow completed: {} total instructions generated for swap + lend flow",
        all_instructions.len()
    );

    Ok(all_instructions)
}
