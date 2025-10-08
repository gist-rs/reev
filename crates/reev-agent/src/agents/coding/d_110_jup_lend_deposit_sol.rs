use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use anyhow::{Context, Result};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_token;
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
    info!("[reev-agent] Matched '110-jup-lend-deposit-sol' id. Calling centralized Jupiter lend deposit handler.");

    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    let wsol_mint = spl_token::native_mint::ID;
    let amount = 100_000_000; // 0.1 SOL

    // --- Jupiter Lend Instruction ---
    // The handler performs account pre-loading and returns the complete set of
    // instructions needed for the transaction, including SOL wrapping.
    info!("[reev-agent] Getting Jupiter lend/deposit instructions...");
    let jupiter_instructions =
        handle_jupiter_lend_deposit(user_pubkey, wsol_mint, amount, key_map).await?;

    // --- Convert all instructions ---
    info!("[reev-agent] Converting Jupiter instructions...");
    let all_instructions: Vec<RawInstruction> = jupiter_instructions;

    info!(
        "[reev-agent] Successfully prepared {} total instructions. Responding to runner.",
        all_instructions.len()
    );

    Ok(all_instructions)
}
