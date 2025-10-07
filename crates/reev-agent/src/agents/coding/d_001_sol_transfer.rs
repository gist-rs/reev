use crate::protocols::native::handle_sol_transfer as protocol_handle_sol_transfer;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) async fn handle_sol_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '001-sol-transfer' id. Calling centralized SOL transfer handler.");

    let from_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let to_pubkey = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .context("RECIPIENT_WALLET_PUBKEY not found in key_map")?;
    let from = Pubkey::from_str(from_pubkey).context("Failed to parse from_pubkey")?;
    let to = Pubkey::from_str(to_pubkey).context("Failed to parse to_pubkey")?;
    let lamports = 100_000_000; // 0.1 SOL

    // Call the protocol handler
    let instructions = protocol_handle_sol_transfer(from, to, lamports, key_map).await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
