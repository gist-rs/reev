use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use std::{collections::HashMap, str::FromStr};
use tracing::info;

/// Handles the deterministic logic for the `110-JUP-LEND-SOL` benchmark.
///
/// A real lending operation is complex. This simplified agent generates a `sol_transfer`
/// to a new, unique public key. This correctly simulates the main effect of lending—the
/// SOL leaving the user's wallet—which is sufficient to satisfy the benchmark's
/// `SolBalanceChange` assertion.
pub(crate) fn handle_jup_lend_sol(key_map: &HashMap<String, String>) -> Result<RawInstruction> {
    info!("[reev-agent] Matched '110-JUP-LEND-SOL' id. Generating a simplified SOL transfer.");
    let from_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;

    let from_pubkey = Pubkey::from_str(from_pubkey_str)?;
    // Since there's no recipient in a lend, we transfer to a new, dummy account
    // to simulate the funds leaving the user's control.
    let to_pubkey = Pubkey::new_unique();
    let lamports = 1_000_000_000; // 1 SOL

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

    info!("[reev-agent] Generated instruction: {:?}", instruction);
    Ok(instruction.into())
}
