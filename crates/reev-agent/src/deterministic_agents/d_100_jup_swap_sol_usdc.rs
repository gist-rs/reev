use crate::jupiter::swap::handle_jupiter_swap;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

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
    info!("[reev-agent] Matched '100-JUP-SWAP-SOL-USDC' id. Calling centralized Jupiter swap handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let input_mint = native_mint::ID;
    let output_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount = 100_000_000; // 0.1 SOL
    let slippage_bps = 500; // 5%

    // The handler performs account pre-loading and returns the complete set of
    // instructions (setup, swap, cleanup) needed for the transaction.
    let instructions = handle_jupiter_swap(
        user_pubkey,
        input_mint,
        output_mint,
        amount,
        slippage_bps,
        key_map,
    )
    .await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
