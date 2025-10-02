use crate::jupiter::lend::handle_jupiter_deposit;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `110-JUP-LEND-DEPOSIT-SOL` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter lend deposit handler.
/// This handler calls the public Jupiter API to get the deposit instructions
/// and pre-loads all required accounts into the local `surfpool` fork.
pub(crate) async fn handle_jup_lend_deposit_sol(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '110-JUP-LEND-DEPOSIT-SOL' id. Calling centralized Jupiter lend deposit handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let asset_mint = native_mint::ID;
    let amount = 100_000_000; // 0.1 SOL

    // The handler performs account pre-loading and returns the complete set of
    // instructions needed for the transaction.
    let instructions = handle_jupiter_deposit(user_pubkey, asset_mint, amount, key_map).await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
