use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use std::collections::HashMap;

/// A helper struct for deserializing the parameters required for an SPL token transfer.
#[derive(Deserialize, Debug)]
struct SplTransferParams {
    /// The source token account pubkey (must be owned by `authority_pubkey`).
    from_pubkey: String,
    /// The destination token account pubkey.
    to_pubkey: String,
    /// The pubkey of the account authorized to sign for the transfer.
    authority_pubkey: String,
    /// The amount of tokens to transfer, in the smallest denomination.
    amount: u64,
}

/// Builds an SPL token transfer transaction.
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
    let transfer_params: SplTransferParams = serde_json::from_value(serde_json::to_value(params)?)
        .context("Failed to deserialize SPL transfer parameters")?;

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

    let authority_pubkey = pubkey_map
        .get(&transfer_params.authority_pubkey)
        .context(format!(
            "Placeholder '{}' not found in pubkey map",
            transfer_params.authority_pubkey
        ))?;

    // This function requires the `spl-token` crate.
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        from_pubkey,
        to_pubkey,
        authority_pubkey,
        &[authority_pubkey], // The authority is the only signer required.
        transfer_params.amount,
    )?;

    // The authority of the token account is also the fee payer for the transaction.
    Ok(Transaction::new_with_payer(&[ix], Some(authority_pubkey)))
}
