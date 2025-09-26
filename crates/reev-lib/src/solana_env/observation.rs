use crate::{agent::AgentObservation, solana_env::SolanaEnv};
use anyhow::Result;
use serde_json::json;
use solana_program::program_pack::Pack;
use solana_sdk::signature::Signer;
use spl_token::state::Account as SplTokenAccount;
use std::collections::HashMap;

pub(crate) fn get_observation(
    env: &SolanaEnv,
    last_tx_status: &str,
    last_tx_error: Option<String>,
    last_tx_logs: Vec<String>,
) -> Result<AgentObservation> {
    let mut account_states = HashMap::new();
    let mut key_map = HashMap::new();

    for (name, keypair) in &env.keypair_map {
        key_map.insert(name.clone(), keypair.pubkey().to_string());
        if let Ok(account) = env.rpc_client.get_account(&keypair.pubkey()) {
            let mut state = json!({
                "lamports": account.lamports,
                "owner": account.owner.to_string(),
                "executable": account.executable,
                "data_len": account.data.len(),
            });

            if account.owner == spl_token::ID && account.data.len() == SplTokenAccount::LEN {
                let token_account = SplTokenAccount::unpack(&account.data)?;
                if let Some(obj) = state.as_object_mut() {
                    obj.insert("mint".to_string(), json!(token_account.mint.to_string()));
                    obj.insert(
                        "token_account_owner".to_string(),
                        json!(token_account.owner.to_string()),
                    );
                    obj.insert("amount".to_string(), json!(token_account.amount));
                }
            }
            account_states.insert(name.clone(), state);
        }
    }

    Ok(AgentObservation {
        account_states,
        key_map,
        last_transaction_status: last_tx_status.to_string(),
        last_transaction_error: last_tx_error,
        last_transaction_logs: last_tx_logs,
    })
}
