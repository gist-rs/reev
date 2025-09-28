use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use spl_token;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) fn handle_spl_transfer(key_map: &HashMap<String, String>) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '002-SPL-TRANSFER' id. Generating instruction with code.");
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
    let amount = 15_000_000;
    let instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[],
        amount,
    )
    .context("Failed to create SPL transfer instruction")?;
    info!("[reev-agent] Generated instruction: {instruction:?}");
    Ok(instruction.into())
}
