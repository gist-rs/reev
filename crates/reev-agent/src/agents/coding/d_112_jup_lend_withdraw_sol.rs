use crate::protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `112-JUP-LEND-WITHDRAW-SOL` benchmark.
///
/// This agent calls the centralized Jupiter lend withdraw handler, which fetches
/// instructions from the Jupiter API and prepares the `surfpool` environment.
pub(crate) async fn handle_jup_lend_withdraw_sol(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '112-JUP-LEND-WITHDRAW-SOL' id. Calling centralized Jupiter lend withdraw handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let asset_mint = native_mint::ID;
    let amount = 100_000_000; // 0.1 SOL

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
