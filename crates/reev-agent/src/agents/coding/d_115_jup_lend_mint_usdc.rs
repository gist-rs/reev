//! Deterministic coding agent for the `115-jup-lend-mint-usdc` benchmark.
//!
//! This agent uses the Jupiter API to fetch mint instructions and returns them
//! in the expected format for the framework.

use anyhow::{Context, Result};
use jup_sdk::api::get_mint_instructions;
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

/// Handles the deterministic logic for the `115-jup-lend-mint-usdc` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter mint handler,
/// which fetches the actual mint instructions from the Jupiter API and prepares
/// them for the framework.
pub async fn handle_jupiter_mint(
    asset: &Pubkey,
    shares: u64,
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    // Get user_pubkey from key_map
    let actual_user_pubkey = if let Some(pubkey_str) = key_map.get("USER_WALLET_PUBKEY") {
        Pubkey::from_str(pubkey_str)
            .map_err(|e| anyhow::anyhow!("Invalid USER_WALLET_PUBKEY: {e}"))?
    } else {
        return Err(anyhow::anyhow!("USER_WALLET_PUBKEY not found in key_map"));
    };

    info!("[d_115_jup_lend_mint_usdc] Fetching mint instructions from Jupiter API");
    info!(
        "[d_115_jup_lend_mint_usdc] User: {}, Asset: {}, Shares: {}",
        actual_user_pubkey, asset, shares
    );

    // Call the Jupiter API to get mint instructions
    let response = get_mint_instructions(asset.to_string(), actual_user_pubkey.to_string(), shares)
        .await
        .context("Failed to get mint instructions from Jupiter API")?;

    // Convert InstructionData to RawInstruction format
    let raw_instructions: Vec<RawInstruction> = response
        .instructions
        .iter()
        .map(|inst| RawInstruction {
            program_id: inst.program_id.clone(),
            accounts: inst
                .accounts
                .iter()
                .map(|acc| reev_lib::agent::RawAccountMeta {
                    pubkey: acc.pubkey.clone(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: inst.data.clone(),
        })
        .collect();

    info!(
        "[d_115_jup_lend_mint_usdc] Successfully converted {} instructions",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
