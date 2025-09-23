use crate::actions::{Action, MockedState};
use crate::solana_env::MockAccountData;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::Value;

/// Implements the `Action` trait for handling mocked Token-2022 transfers.
/// Note: The internal logic is identical to the standard SPL-Token transfer for our mock,
/// as they share the same account data structure (`SplTokenState`).
pub struct Token2022TransferAction;

/// Helper struct for deserializing parameters. This is identical to the SPL-Token params.
#[derive(Deserialize, Debug)]
struct Token2022TransferParams {
    source_pubkey: String,
    destination_pubkey: String,
    owner_pubkey: String,
    amount: u64,
}

impl Action for Token2022TransferAction {
    /// Executes a mocked Token-2022 transfer.
    fn execute(&self, state: &mut MockedState, params: &Value) -> Result<()> {
        let transfer_params: Token2022TransferParams = serde_json::from_value(params.clone())
            .context("Failed to deserialize Token-2022 transfer parameters")?;

        // --- Validation Step 1: Check source account ---
        let source_account_data = state
            .get(&transfer_params.source_pubkey)
            .ok_or_else(|| {
                anyhow!(
                    "Source token account {} not found.",
                    transfer_params.source_pubkey
                )
            })?
            .data
            .clone();

        let source_token_data = if let MockAccountData::SplToken(data) = source_account_data {
            data
        } else {
            return Err(anyhow!(
                "Source account {} is not a valid Token-2022 account.",
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
                    "Destination account {} is not a valid Token-2022 account.",
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
        let source_account_mut = state.get_mut(&transfer_params.source_pubkey).unwrap();
        if let MockAccountData::SplToken(data) = &mut source_account_mut.data {
            data.amount -= transfer_params.amount;
        }

        let destination_account_mut = state.get_mut(&transfer_params.destination_pubkey).unwrap();
        if let MockAccountData::SplToken(data) = &mut destination_account_mut.data {
            data.amount += transfer_params.amount;
        }

        println!(
            "[Token2022TransferAction] Transferred {} tokens from {} to {}.",
            transfer_params.amount,
            transfer_params.source_pubkey,
            transfer_params.destination_pubkey
        );

        Ok(())
    }
}
