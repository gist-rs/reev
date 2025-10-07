use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use anyhow::{Context, Result};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account;
use spl_token;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Converts a `solana_sdk::instruction::Instruction` to the agent's `RawInstruction` format.
fn to_raw_instruction(instruction: Instruction) -> RawInstruction {
    let accounts = instruction
        .accounts
        .into_iter()
        .map(|acc| RawAccountMeta {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    RawInstruction {
        program_id: instruction.program_id.to_string(),
        accounts,
        data: bs58::encode(instruction.data).into_string(),
    }
}

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

    // --- 1. Instructions to wrap SOL ---
    // The Jupiter program expects the user to have an initialized WSOL token account.
    // We must create it and fund it before calling the lend instruction.
    info!("[reev-agent] Creating SOL wrap instructions...");
    let wsol_ata =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    let wrap_instructions = vec![
        // Create ATA. This is idempotent, so it's safe to call even if it exists.
        spl_associated_token_account::instruction::create_associated_token_account(
            &user_pubkey,
            &user_pubkey,
            &wsol_mint,
            &spl_token::ID,
        ),
        // Transfer SOL to WSOL ATA to wrap it.
        system_instruction::transfer(&user_pubkey, &wsol_ata, amount),
        // Sync the ATA to have the correct balance for the Jupiter program.
        spl_token::instruction::sync_native(&spl_token::ID, &wsol_ata)?,
    ];

    // --- 2. Jupiter Lend Instruction ---
    // The handler performs account pre-loading and returns the complete set of
    // instructions needed for the transaction.
    info!("[reev-agent] Getting Jupiter lend/deposit instructions...");
    let mut jupiter_instructions =
        handle_jupiter_lend_deposit(user_pubkey, wsol_mint, amount, key_map).await?;

    // --- 3. Combine and convert all instructions ---
    info!("[reev-agent] Combining and converting all instructions...");
    let mut all_instructions: Vec<RawInstruction> = wrap_instructions
        .into_iter()
        .map(to_raw_instruction)
        .collect();

    all_instructions.append(&mut jupiter_instructions);

    info!(
        "[reev-agent] Successfully prepared {} total instructions. Responding to runner.",
        all_instructions.len()
    );

    Ok(all_instructions)
}
