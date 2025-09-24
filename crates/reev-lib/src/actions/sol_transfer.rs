use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction};
use solana_system_interface::instruction as system_instruction;
use std::collections::HashMap;

/// A helper struct for deserializing the parameters required for a SOL transfer.
#[derive(Deserialize, Debug)]
struct SolTransferParams {
    from_pubkey: String,
    to_pubkey: String,
    lamports: u64,
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

    let ix = system_instruction::transfer(from_pubkey, to_pubkey, transfer_params.lamports);

    // For a simple transfer, the `from_pubkey` is also the fee payer.
    Ok(Transaction::new_with_payer(&[ix], Some(from_pubkey)))
}
