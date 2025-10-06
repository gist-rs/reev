use crate::protocols::native::handle_spl_transfer as protocol_handle_spl_transfer;
use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) async fn handle_spl_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[reev-agent] Matched '002-SPL-TRANSFER' id. Calling centralized SPL transfer handler.");
    let source_ata_str = key_map
        .get("USER_USDC_ATA")
        .context("USER_USDC_ATA not found in key_map")?;
    let dest_ata_str = key_map
        .get("RECIPIENT_USDC_ATA")
        .context("RECIPIENT_USDC_ATA not found in key_map")?;
    let authority_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let source_pubkey =
        Pubkey::from_str(source_ata_str).context("Failed to parse source ATA pubkey")?;
    let destination_pubkey =
        Pubkey::from_str(dest_ata_str).context("Failed to parse destination ATA pubkey")?;
    let authority_pubkey =
        Pubkey::from_str(authority_str).context("Failed to parse authority pubkey")?;
    let amount = 15_000_000; // 15 USDC

    // Call the protocol handler
    let instructions = protocol_handle_spl_transfer(
        source_pubkey,
        destination_pubkey,
        authority_pubkey,
        amount,
        key_map,
    )
    .await?;

    info!(
        "[reev-agent] Successfully received {} instructions. Responding to runner.",
        instructions.len()
    );

    Ok(instructions)
}
