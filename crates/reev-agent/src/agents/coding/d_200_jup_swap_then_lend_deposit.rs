use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use crate::protocols::jupiter::swap::handle_jupiter_swap;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
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
    let output_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let swap_amount = 100_000_000; // 0.1 SOL
    let slippage_bps = 500; // 5%

    let swap_instructions = handle_jupiter_swap(
        user_pubkey,
        input_mint,
        output_mint,
        swap_amount,
        slippage_bps,
        key_map,
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
    let deposit_amount = 9_000_000; // ~9 USDC (accounting for slippage and fees)
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    let lend_instructions =
        handle_jupiter_lend_deposit(user_pubkey, usdc_mint, deposit_amount, key_map).await?;

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
