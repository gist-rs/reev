use crate::actions::{Action, MockedState};
use crate::solana_env::MockAccountData;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::Value;

/// Implements the `Action` trait for handling mocked SPL-Token transfers.
pub struct SplTokenTransferAction;

/// Helper struct for deserializing the parameters required for an SPL-Token transfer.
#[derive(Deserialize, Debug)]
struct SplTokenTransferParams {
    /// The source token account public key.
    source_pubkey: String,
    /// The destination token account public key.
    destination_pubkey: String,
    /// The public key of the owner of the source token account, who is authorizing the transfer.
    owner_pubkey: String,
    /// The amount of tokens to transfer, in the smallest unit.
    amount: u64,
}

impl Action for SplTokenTransferAction {
    /// Executes a mocked SPL-Token transfer.
    ///
    /// This function validates that both source and destination accounts are valid token accounts,
    /// that they share the same mint, that the owner is correct, and that there are sufficient
    /// funds. It then mutates the `amount` field in the respective account data.
    ///
    /// # Expected JSON Parameters:
    /// ```json
    /// {
    ///   "source_pubkey": "SENDER_TOKEN_ACCOUNT_PUBKEY",
    ///   "destination_pubkey": "RECEIVER_TOKEN_ACCOUNT_PUBKEY",
    ///   "owner_pubkey": "OWNER_OF_SENDER_TOKEN_ACCOUNT_PUBKEY",
    ///   "amount": 15000000
    /// }
    /// ```
    fn execute(&self, state: &mut MockedState, params: &Value) -> Result<()> {
        let transfer_params: SplTokenTransferParams = serde_json::from_value(params.clone())
            .context("Failed to deserialize SPL-Token transfer parameters")?;

        // --- Validation Step 1: Check source account ---
        let source_account_data = state
            .get_mut(&transfer_params.source_pubkey)
            .ok_or_else(|| {
                anyhow!(
                    "Source token account {} not found.",
                    transfer_params.source_pubkey
                )
            })?
            .data
            .clone(); // Clone to avoid mutable borrow issues

        let source_token_data = if let MockAccountData::SplToken(data) = source_account_data {
            data
        } else {
            return Err(anyhow!(
                "Source account {} is not a valid SPL-Token account.",
                transfer_params.source_pubkey
            ));
        };

        if source_token_data.owner != transfer_params.owner_pubkey {
            return Err(anyhow!(
                "Owner mismatch for source account {}. Expected authority {}, but account is owned by {}.",
                transfer_params.source_pubkey,
                transfer_params.owner_pubkey,
                source_token_data.owner
            ));
        }

        if source_token_data.amount < transfer_params.amount {
            return Err(anyhow!(
                "Insufficient funds in source token account {}. Required: {}, Available: {}.",
                transfer_params.source_pubkey,
                transfer_params.amount,
                source_token_data.amount
            ));
        }

        // --- Validation Step 2: Check destination account ---
        let destination_account_data = state
            .get(&transfer_params.destination_pubkey)
            .ok_or_else(|| {
                anyhow!(
                    "Destination token account {} not found.",
                    transfer_params.destination_pubkey
                )
            })?
            .data
            .clone();

        let destination_token_data =
            if let MockAccountData::SplToken(data) = destination_account_data {
                data
            } else {
                return Err(anyhow!(
                    "Destination account {} is not a valid SPL-Token account.",
                    transfer_params.destination_pubkey
                ));
            };

        // --- Validation Step 3: Check if mints match ---
        if source_token_data.mint != destination_token_data.mint {
            return Err(anyhow!(
                "Mint mismatch between source ({}) and destination ({}).",
                source_token_data.mint,
                destination_token_data.mint
            ));
        }

        // --- Mutation Step: Perform the transfer ---
        // Re-borrow as mutable now that all checks have passed.
        let source_account_mut = state.get_mut(&transfer_params.source_pubkey).unwrap();
        if let MockAccountData::SplToken(data) = &mut source_account_mut.data {
            data.amount -= transfer_params.amount;
        }

        let destination_account_mut = state.get_mut(&transfer_params.destination_pubkey).unwrap();
        if let MockAccountData::SplToken(data) = &mut destination_account_mut.data {
            data.amount += transfer_params.amount;
        }

        println!(
            "[SplTokenTransferAction] Transferred {} tokens from {} to {}.",
            transfer_params.amount,
            transfer_params.source_pubkey,
            transfer_params.destination_pubkey
        );

        Ok(())
    }
}
