use crate::{
    agent::{AgentAction, AgentObservation},
    env::Step,
    solana_env::{observation, SolanaEnv},
};
use anyhow::Result;
use serde_json::json;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_transaction_status::UiTransactionEncoding;
use tracing::{error, info, warn};

pub(crate) fn handle_step(
    env: &mut SolanaEnv,
    action: AgentAction,
) -> Result<Step<AgentObservation>> {
    let instruction = action.0;
    let fee_payer_keypair = env.get_fee_payer_keypair()?;
    let mut signers = vec![fee_payer_keypair];

    for acc in &instruction.accounts {
        if acc.is_signer {
            if let Some(keypair) = env
                .keypair_map
                .values()
                .find(|kp| kp.pubkey() == acc.pubkey)
            {
                signers.push(keypair);
            } else {
                warn!(
                    "Signer keypair for pubkey {} not found in keypair_map. Transaction may fail.",
                    acc.pubkey
                );
            }
        }
    }
    signers.sort_by_key(|k| k.pubkey());
    signers.dedup_by_key(|k| k.pubkey());

    let transaction =
        Transaction::new_with_payer(&[instruction.clone()], Some(&fee_payer_keypair.pubkey()));

    info!(
        "Executing instruction for program: {}",
        instruction.program_id
    );

    // --- Simulation Logic ---
    info!("Simulating transaction before sending...");
    let sim_config = RpcSimulateTransactionConfig {
        sig_verify: false,
        replace_recent_blockhash: true,
        commitment: Some(env.rpc_client.commitment()),
        ..Default::default()
    };
    let sim_result = env
        .rpc_client
        .simulate_transaction_with_config(&transaction, sim_config)?;

    let sim_logs = sim_result.value.logs.clone().unwrap_or_default();
    info!(simulation_logs = ?sim_logs, "Transaction simulation logs");

    if let Some(err) = sim_result.value.err {
        let error_string = format!("Transaction simulation failed: {err}");
        error!("{}", error_string);
        let obs =
            observation::get_observation(env, "Failure", Some(error_string.clone()), sim_logs)?;
        return Ok(Step {
            observation: obs,
            reward: 0.0,
            terminated: true,
            truncated: false,
            info: json!({ "error": error_string }),
        });
    }
    info!("Transaction simulation successful.");
    // --- End Simulation Logic ---

    match env.sign_and_send_transaction(transaction, &signers) {
        Ok(sig) => {
            let tx_info = env
                .rpc_client
                .get_transaction(&sig, UiTransactionEncoding::Json)?;
            let logs = tx_info
                .transaction
                .meta
                .and_then(|meta| meta.log_messages.into())
                .unwrap_or_default();
            let info = json!({ "signature": sig.to_string() });
            let obs = observation::get_observation(env, "Success", None, logs)?;
            Ok(Step {
                observation: obs,
                reward: 1.0,
                terminated: true,
                truncated: false,
                info,
            })
        }
        Err(e) => {
            let error_string = format!("Transaction failed: {e}");
            warn!("{}", error_string);
            let obs = observation::get_observation(env, "Failure", Some(e.to_string()), sim_logs)?;
            Ok(Step {
                observation: obs,
                reward: 0.0,
                terminated: true,
                truncated: false,
                info: json!({ "error": error_string }),
            })
        }
    }
}
