use crate::actions::{Action, MockedState};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::Value;

/// A struct that implements the `Action` trait for handling native SOL transfers.
pub struct SolTransferAction;

/// A helper struct for deserializing the parameters required for a SOL transfer.
#[derive(Deserialize, Debug)]
struct SolTransferParams {
    from_pubkey: String,
    to_pubkey: String,
    lamports: u64,
}

impl Action for SolTransferAction {
    /// Executes a mocked SOL transfer.
    ///
    /// This function validates the parameters, checks for sufficient funds, and
    /// updates the balances of the sender and receiver accounts in the mocked state.
    ///
    /// # Expected JSON Parameters:
    /// ```json
    /// {
    ///   "from_pubkey": "SENDER_PUBKEY_STRING",
    ///   "to_pubkey": "RECEIVER_PUBKEY_STRING",
    ///   "lamports": 100000000
    /// }
    /// ```
    fn execute(&self, state: &mut MockedState, params: &Value) -> Result<()> {
        let transfer_params: SolTransferParams = serde_json::from_value(params.clone())
            .context("Failed to deserialize SOL transfer parameters")?;

        // --- Validation Step 1: Check if sender account exists and has enough funds ---
        let sender = state
            .get_mut(&transfer_params.from_pubkey)
            .ok_or_else(|| anyhow!("Source account {} not found.", transfer_params.from_pubkey))?;

        if sender.lamports < transfer_params.lamports {
            return Err(anyhow!(
                "Insufficient funds in account {}. Required: {}, Available: {}.",
                transfer_params.from_pubkey,
                transfer_params.lamports,
                sender.lamports
            ));
        }

        // --- Validation Step 2: Check if receiver account exists ---
        if !state.contains_key(&transfer_params.to_pubkey) {
            return Err(anyhow!(
                "Destination account {} not found.",
                transfer_params.to_pubkey
            ));
        }

        // --- Mutation Step: Perform the transfer ---
        // This is done atomically after all checks have passed.
        let sender = state.get_mut(&transfer_params.from_pubkey).unwrap();
        sender.lamports -= transfer_params.lamports;

        let receiver = state.get_mut(&transfer_params.to_pubkey).unwrap();
        receiver.lamports += transfer_params.lamports;

        println!(
            "[SolTransferAction] Transferred {} lamports from {} to {}.",
            transfer_params.lamports, transfer_params.from_pubkey, transfer_params.to_pubkey
        );

        Ok(())
    }
}
