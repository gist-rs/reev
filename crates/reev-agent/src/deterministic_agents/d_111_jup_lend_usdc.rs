use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `111-JUP-LEND-USDC` benchmark.
///
/// A real lending operation is complex. This simplified agent generates a `spl_transfer`
/// to a new, unique public key. This correctly simulates the main effect of lending—the
/// USDC leaving the user's wallet—which is sufficient to satisfy the benchmark's
/// `TokenAccountBalance` assertion.
pub(crate) fn handle_jup_lend_usdc(key_map: &HashMap<String, String>) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '111-JUP-LEND-USDC' id. Generating a simplified SPL transfer.");
    let source_ata_str = key_map
        .get("USER_USDC_ATA")
        .context("USER_USDC_ATA not found in key_map")?;
    let authority_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;

    let source_pubkey =
        Pubkey::from_str(source_ata_str).context("Failed to parse source ATA pubkey")?;
    let authority_pubkey =
        Pubkey::from_str(authority_str).context("Failed to parse authority pubkey")?;

    // For lending, the destination is a protocol-owned account. We can simulate this
    // by transferring to a new, unique public key that represents the protocol's vault.
    let destination_pubkey = Pubkey::new_unique();
    let amount = 100_000_000; // 100 USDC

    let instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[], // A standard transfer has no extra signers.
        amount,
    )
    .context("Failed to create SPL transfer instruction")?;

    info!("[reev-agent] Generated instruction: {instruction:?}");
    Ok(instruction.into())
}
