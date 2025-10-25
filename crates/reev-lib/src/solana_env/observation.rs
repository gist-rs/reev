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
use std::{collections::HashMap, str::FromStr};
use tracing::info;

pub(crate) fn get_observation(
    env: &mut SolanaEnv,
    ground_truth: &GroundTruth,
    last_tx_status: &str,
    last_tx_error: Option<String>,
    last_tx_logs: Vec<String>,
) -> Result<AgentObservation> {
    let mut account_states = HashMap::new();
    let mut key_map: HashMap<String, Pubkey> = env.pubkey_map.clone();

    // --- 1. Ensure all derived addresses are in the environment's key_map ---
    for assertion in &ground_truth.final_state_assertions {
        if let Some(derivation) = assertion.address_derivation() {
            let placeholder = assertion.pubkey();

            // For SPL placeholders, skip processing to preserve environment addresses
            // SPECIAL HANDLING FOR SPL TRANSFERS: Never regenerate known placeholders
            let is_spl_placeholder = placeholder.contains("USDC_ATA")
                || placeholder.contains("USER_USDC_ATA")
                || (placeholder.contains("USER_WALLET_PUBKEY")
                    && env.pubkey_map.contains_key(placeholder));

            info!(
                "[observation] Processing placeholder: {}, is_spl_placeholder: {}",
                placeholder, is_spl_placeholder
            );

            if is_spl_placeholder {
                // Skip SPL placeholders - their addresses should already be in env.pubkey_map
                info!(
                    "[Observation] Skipping SPL placeholder {} to preserve environment address",
                    placeholder
                );
                continue;
            }

            // Only derive and insert if placeholder is not already mapped.
            if !env.pubkey_map.contains_key(placeholder) {
                let AddressDerivation::AssociatedTokenAccount { owner, mint } = derivation;
                if let Some(owner_pubkey) = key_map.get(owner) {
                    let mint_pubkey = Pubkey::from_str(mint)?;
                    let derived_ata = get_associated_token_address(owner_pubkey, &mint_pubkey);
                    let placeholder_str = placeholder.to_string();
                    info!(
                        "Mapping derived ATA {} to placeholder '{}'",
                        derived_ata, placeholder_str
                    );
                    // Add the derived address to the list of accounts to fetch
                    key_map.insert(placeholder_str, derived_ata);
                }
            }
        }
    }

    // --- 2. Fetch all account states ---
    for (name, pubkey) in &key_map {
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
                        obj.insert("amount".to_string(), json!(token_account.amount));
                    }
                }
            }
            account_states.insert(name.clone(), state);
        } else {
            // Account doesn't exist on-chain (0 lamports), include it from initial_state
            // This ensures recipients with 0 balance are included in context
            info!(
                "Account {} ({}) not found on-chain, including as non-existent",
                name, pubkey
            );
            let state = json!({
                "lamports": 0,
                "owner": "11111111111111111111111111111111", // System Program by default
                "executable": false,
                "data_len": 0,
                "exists": false
            });
            account_states.insert(name.clone(), state);
        }
    }

    let final_key_map = key_map
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
