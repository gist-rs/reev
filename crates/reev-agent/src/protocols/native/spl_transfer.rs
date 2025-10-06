//! Native SPL transfer protocol handler
//!
//! This module provides the real Solana protocol integration for SPL token transfer operations.

use anyhow::Result;
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::collections::HashMap;

/// Handle SPL token transfer operation using Solana SPL token instructions.
/// This is the real protocol handler that contains the actual SPL transfer logic.
pub async fn handle_spl_transfer(
    source_pubkey: Pubkey,
    destination_pubkey: Pubkey,
    authority_pubkey: Pubkey,
    amount: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    // Create the SPL transfer instruction
    let instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        &[],
        amount,
    )?;

    // Convert to RawInstruction format
    let raw_instruction = instruction_to_raw(instruction);

    Ok(vec![raw_instruction])
}

/// Convert a solana_sdk::Instruction to our RawInstruction format
pub fn instruction_to_raw(instruction: Instruction) -> RawInstruction {
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
