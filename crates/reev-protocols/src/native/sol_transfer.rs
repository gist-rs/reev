//! Native SOL transfer protocol handler
//!
//! This module provides the real Solana protocol integration for SOL transfer operations.

use anyhow::Result;
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use reev_lib::{balance_validation::BalanceValidator, get_keypair};
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
    // Check if this is a special case for "all" sol transfer (u64::MAX)
    let actual_amount = if lamports == u64::MAX {
        // Get the keypair to access wallet balance
        let _keypair = get_keypair()?;

        // Create balance validator with empty key_map
        let balance_validator = BalanceValidator::new(HashMap::new());

        // Get current balance using pubkey string
        let current_balance = match balance_validator.get_sol_balance(&from_pubkey.to_string()) {
            Ok(balance) => balance,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get balance: {}", e.to_string()));
            }
        };

        // Calculate gas reserve (0.001 SOL for now)
        let gas_reserve = 1_000_000u64; // 0.001 SOL in lamports

        // Calculate maximum transferable amount
        if current_balance <= gas_reserve {
            return Err(anyhow::anyhow!(
                "Insufficient balance for transfer. Balance: {} SOL, required reserve: {} SOL",
                current_balance as f64 / 1_000_000_000.0,
                gas_reserve as f64 / 1_000_000_000.0
            ));
        }

        let max_transferable = current_balance - gas_reserve;

        // Return the calculated amount
        max_transferable
    } else {
        // Use the provided amount directly
        lamports
    };

    // Create the system transfer instruction
    let instruction =
        solana_system_interface::instruction::transfer(&from_pubkey, &to_pubkey, actual_amount);

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
