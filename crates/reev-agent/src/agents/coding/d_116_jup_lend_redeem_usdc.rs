//! Deterministic coding agent for the `116-jup-lend-redeem-usdc` benchmark.
//!
//! This agent uses the Jupiter lend_redeem protocol handler to fetch redeem instructions
//! and returns them in the expected format for the framework.

use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::info;

/// Handles the deterministic logic for the `116-jup-lend-redeem-usdc` benchmark.
///
/// This agent acts as an oracle by calling the centralized Jupiter lend_redeem handler,
/// which fetches the actual redeem instructions from the Jupiter API and prepares
/// them for execution in the simulated environment.
#[allow(dead_code)]
pub async fn handle_jupiter_redeem(
    asset: &Pubkey,
    shares: u64,
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    info!("[d_116_jup_lend_redeem_usdc] Handling Jupiter redeem via lend_redeem protocol");
    info!(
        "[d_116_jup_lend_redeem_usdc] Asset: {}, Shares: {}",
        asset, shares
    );

    // Call the centralized lend_redeem protocol handler
    let raw_instructions =
        reev_protocols::jupiter::execute_jupiter_lend_redeem(asset, shares, key_map)
            .await
            .context("Failed to execute Jupiter lend redeem")?;

    info!(
        "[d_116_jup_lend_redeem_usdc] Successfully received {} instructions",
        raw_instructions.len()
    );

    Ok(raw_instructions)
}
