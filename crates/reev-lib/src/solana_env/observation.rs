use crate::{
    agent::AgentObservation,
    benchmark::{AddressDerivation, GroundTruth},
    solana_env::environment::SolanaEnv,
};
use anyhow::Result;
use serde_json::json;
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as SplTokenAccount;
use std::{collections::HashMap, str::FromStr, thread, time::Duration};
use tracing::info;

pub(crate) fn get_observation(
    env: &mut SolanaEnv,
    ground_truth: &GroundTruth,
    last_tx_status: &str,
    last_tx_error: Option<String>,
    last_tx_logs: Vec<String>,
) -> Result<AgentObservation> {
    let mut account_states = HashMap::new();

    // --- 1. Ensure all derived addresses are in the environment's key_map ---
    for assertion in &ground_truth.final_state_assertions {
        if let Some(derivation) = assertion.address_derivation() {
            let placeholder = assertion.pubkey();
            // Only derive and insert if the placeholder is not already mapped.
            if !env.pubkey_map.contains_key(placeholder) {
                if let AddressDerivation::AssociatedTokenAccount { owner, mint } = derivation {
                    if let Some(owner_pubkey) = env.pubkey_map.get(owner) {
                        let mint_pubkey = Pubkey::from_str(mint)?;
                        let derived_ata = get_associated_token_address(owner_pubkey, &mint_pubkey);
                        info!(
                            "Mapping derived ATA {} to placeholder '{}'",
                            derived_ata, placeholder
                        );
                        // Add the derived address to the environment's persistent map
                        env.pubkey_map.insert(placeholder.to_string(), derived_ata);
                    }
                }
            }
        }
    }

    // --- 2. Fetch all account states ---
    info!(pubkeys_to_fetch = ?env.pubkey_map, "Fetching accounts for observation");
    for i in 0..3 {
        for (name, pubkey) in &env.pubkey_map {
            // If we already have the account, skip it.
            if account_states.contains_key(name) {
                continue;
            }

            if let Ok(account) = env.rpc_client.get_account(pubkey) {
                let mut state = json!({
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "data_len": account.data.len(),
                });

                if account.owner == spl_token::ID && account.data.len() == SplTokenAccount::LEN {
                    if let Ok(token_account) = SplTokenAccount::unpack(&account.data) {
                        if let Some(obj) = state.as_object_mut() {
                            obj.insert("mint".to_string(), json!(token_account.mint.to_string()));
                            obj.insert(
                                "token_account_owner".to_string(),
                                json!(token_account.owner.to_string()),
                            );
                            obj.insert(
                                "amount".to_string(),
                                json!(token_account.amount.to_string()),
                            );
                        }
                    }
                }
                account_states.insert(name.clone(), state);
            }
        }

        // If we have all accounts, break early.
        if account_states.len() == env.pubkey_map.len() {
            info!("Successfully fetched all accounts.");
            break;
        }

        if i < 2 {
            info!(
                "Missing {} accounts, retrying in 500ms...",
                env.pubkey_map.len() - account_states.len()
            );
            thread::sleep(Duration::from_millis(500));
        }
    }

    let final_key_map = env
        .pubkey_map
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string()))
        .collect();

    Ok(AgentObservation {
        account_states,
        key_map: final_key_map,
        last_transaction_status: last_tx_status.to_string(),
        last_transaction_error: last_tx_error,
        last_transaction_logs: last_tx_logs,
    })
}
