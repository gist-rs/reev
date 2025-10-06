use crate::protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `113-JUP-LEND-WITHDRAW-USDC` benchmark.
///
/// This agent calls the centralized Jupiter lend withdraw handler, which fetches
/// instructions from the Jupiter API and prepares the `surfpool` environment.
pub(crate) async fn handle_jup_lend_withdraw_usdc(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '113-JUP-LEND-WITHDRAW-USDC' id. Calling centralized Jupiter lend withdraw handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let asset_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount = 10_000_000; // 10 USDC

    // The handler performs account pre-loading and returns the complete set of
    // instructions needed for the transaction.
    let instructions =
        handle_jupiter_lend_withdraw(user_pubkey, asset_mint, amount, key_map).await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
