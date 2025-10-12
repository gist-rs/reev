use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use reev_lib::constants::{usdc_mint, USDC_LEND_AMOUNT};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `111-JUP-LEND-DEPOSIT-USDC` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter lend deposit handler.
/// This handler calls the public Jupiter API to get the deposit instructions
/// and pre-loads all required accounts into the local `surfpool` fork.
pub(crate) async fn handle_jup_lend_deposit_usdc(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '111-jup-lend-deposit-usdc' id. Calling centralized Jupiter lend deposit handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let asset_mint = usdc_mint();
    let amount = USDC_LEND_AMOUNT; // 10 USDC

    // The handler performs account pre-loading and returns the complete set of
    // instructions needed for the transaction.
    let instructions = handle_jupiter_lend_deposit(user_pubkey, asset_mint, amount).await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
