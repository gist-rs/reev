use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) fn handle_sol_transfer(key_map: &HashMap<String, String>) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '001-SOL-TRANSFER' id. Generating instruction with code.");
    let from_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let to_pubkey = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .context("RECIPIENT_WALLET_PUBKEY not found in key_map")?;
    let from = Pubkey::from_str(from_pubkey).context("Failed to parse from_pubkey")?;
    let to = Pubkey::from_str(to_pubkey).context("Failed to parse to_pubkey")?;
    let lamports = 100_000_000;
    let instruction = system_instruction::transfer(&from, &to, lamports);
    info!("[reev-agent] Generated instruction: {instruction:?}");
    Ok(instruction.into())
}
