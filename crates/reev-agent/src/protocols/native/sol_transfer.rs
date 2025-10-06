//! Native SOL transfer protocol handler
//!
//! This module provides the real Solana protocol integration for SOL transfer operations.

use anyhow::Result;
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::collections::HashMap;

/// Handle native SOL transfer operation using Solana system instructions.
/// This is the real protocol handler that contains the actual SOL transfer logic.
pub async fn handle_sol_transfer(
    from_pubkey: Pubkey,
    to_pubkey: Pubkey,
    lamports: u64,
    _key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    // Create the system transfer instruction
    let instruction =
        solana_system_interface::instruction::transfer(&from_pubkey, &to_pubkey, lamports);

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
