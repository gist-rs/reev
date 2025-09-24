use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, transaction::Transaction};
use solana_system_interface::instruction as system_instruction;
use std::collections::HashMap;

/// A helper struct for deserializing the parameters required for a SOL transfer.
#[derive(Deserialize, Debug)]
struct SolTransferParams {
    from_pubkey: String,
    to_pubkey: String,
    lamports: u64,
}

/// Creates a native SOL transfer instruction.
///
/// This is a pure function that returns the instruction, which can then be
/// embedded in a transaction.
///
/// # Arguments
/// * `from_pubkey`: The public key of the account that will send the SOL.
/// * `to_pubkey`: The public key of the account that will receive the SOL.
/// * `lamports`: The amount of lamports to transfer.
///
/// # Returns
/// An `Instruction` object for the transfer.
pub fn create_instruction(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64) -> Instruction {
    system_instruction::transfer(from_pubkey, to_pubkey, lamports)
}

/// Builds a native SOL transfer transaction.
///
/// This function is responsible for creating the transaction object, but not for
/// signing or sending it. That responsibility lies with the `SolanaEnv`.
///
/// # Arguments
/// * `params`: The parameters for the action, taken from the `AgentAction`.
/// * `pubkey_map`: The map of placeholder strings to real `Pubkey`s.
///
/// # Returns
/// A `Transaction` object ready to be signed and sent.
pub fn build_transaction(
    params: &HashMap<String, Value>,
    pubkey_map: &HashMap<String, Pubkey>,
) -> Result<Transaction> {
    let transfer_params: SolTransferParams = serde_json::from_value(serde_json::to_value(params)?)
        .context("Failed to deserialize SOL transfer parameters")?;

    let from_pubkey = pubkey_map
        .get(&transfer_params.from_pubkey)
        .context(format!(
            "Placeholder '{}' not found in pubkey map",
            transfer_params.from_pubkey
        ))?;
    let to_pubkey = pubkey_map.get(&transfer_params.to_pubkey).context(format!(
        "Placeholder '{}' not found in pubkey map",
        transfer_params.to_pubkey
    ))?;

    let ix = create_instruction(from_pubkey, to_pubkey, transfer_params.lamports);

    // For a simple transfer, the `from_pubkey` is also the fee payer.
    Ok(Transaction::new_with_payer(&[ix], Some(from_pubkey)))
}
