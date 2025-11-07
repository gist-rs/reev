//! Deterministic coding agent for the `115-jup-lend-mint-usdc` benchmark.
//!
//! This agent uses the Jupiter lend_mint protocol handler to fetch mint instructions
//! and returns them in the expected format for the framework.

use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

/// Handles the deterministic logic for the `115-jup-lend-mint-usdc` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter lend_mint handler,
/// which fetches the actual mint instructions from the Jupiter API and prepares
/// them for execution in the simulated environment.
#[allow(dead_code)]
pub async fn handle_jupiter_mint(
    asset: &Pubkey,
    shares: u64,
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[d_115_jup_lend_mint_usdc] Handling Jupiter mint via lend_mint protocol");
    info!(
        "[d_115_jup_lend_mint_usdc] Asset: {}, Shares: {}",
        asset, shares
    );

    // Call the centralized lend_mint protocol handler
    let raw_instructions =
        reev_protocols::jupiter::execute_jupiter_lend_mint(asset, shares, key_map)
            .await
            .context("Failed to execute Jupiter lend mint")?;

    info!(
        "[d_115_jup_lend_mint_usdc] Successfully received {} instructions",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
