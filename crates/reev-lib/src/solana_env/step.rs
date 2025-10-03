use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::GroundTruth,
    env::Step,
    solana_env::{environment::SolanaEnv, observation},
};
use anyhow::Result;
use serde_json::json;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::{instruction::Instruction, signature::Signer, transaction::Transaction};
use solana_transaction_status::UiTransactionEncoding;
use tracing::{error, info, warn};

pub(crate) fn handle_step(
    env: &mut SolanaEnv,
    actions: Vec<AgentAction>,
    ground_truth: &GroundTruth,
) -> Result<Step<AgentObservation>> {
    // --- 1. Define variables to hold the outcome of the transaction ---
    let mut tx_status = "Failure";
    let mut tx_error: Option<String> = None;
    let mut tx_logs: Vec<String> = Vec::new();
    let info;
    let mut reward = 0.0;

    // --- 2. Handle the case of no actions ---
    if actions.is_empty() {
        let error_string = "Agent returned no actions to execute.".to_string();
        error!("{}", error_string);
        tx_error = Some(error_string.clone());
        info = json!({ "error": error_string });
    } else {
        // --- 3. Build and simulate the transaction ---
        let instructions: Vec<Instruction> = actions.into_iter().map(|a| a.0).collect();
        let fee_payer_keypair = env.get_fee_payer_keypair()?;
        let mut signers = vec![fee_payer_keypair];

        for instruction in &instructions {
            for acc in &instruction.accounts {
                if acc.is_signer {
                    if let Some(keypair) = env
                        .keypair_map
                        .values()
                        .find(|kp| kp.pubkey() == acc.pubkey)
                    {
                        if !signers.iter().any(|s| s.pubkey() == keypair.pubkey()) {
                            signers.push(keypair);
                        }
                    } else if acc.pubkey != fee_payer_keypair.pubkey() {
                        warn!(
                            "Signer keypair for pubkey {} not found in keypair_map. Transaction may fail.",
                            acc.pubkey
                        );
                    }
                }
            }
        }

        let mut transaction =
            Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));

        info!(
            "Executing transaction with {} instruction(s).",
            instructions.len()
        );

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
            tx_error = Some(error_string.clone());
            tx_logs = sim_logs;
            info = json!({ "error": error_string });
        } else {
            // --- 4. If simulation succeeds, execute the transaction ---
            info!("Transaction simulation successful. Executing transaction...");
            let latest_blockhash = env.rpc_client.get_latest_blockhash()?;
            transaction.sign(&signers, latest_blockhash);

            match env.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(sig) => {
                    let tx_info = env
                        .rpc_client
                        .get_transaction(&sig, UiTransactionEncoding::Json)?;
                    tx_logs = tx_info
                        .transaction
                        .meta
                        .and_then(|meta| meta.log_messages.into())
                        .unwrap_or_default();
                    info = json!({ "signature": sig.to_string() });
                    tx_status = "Success";
                    reward = 1.0;
                }
                Err(e) => {
                    let error_string =
                        format!("Transaction execution failed after successful simulation: {e}");
                    error!("{}", error_string);
                    tx_error = Some(error_string.clone());
                    tx_logs = sim_logs; // On execution failure, we only have simulation logs
                    info = json!({ "error": error_string });
                }
            }
        }
    }

    // --- 5. Get the final observation AFTER the transaction has settled ---
    let obs = observation::get_observation(env, ground_truth, tx_status, tx_error, tx_logs)?;

    // --- 6. Return the final step result ---
    Ok(Step {
        observation: obs,
        reward,
        terminated: true,
        truncated: false,
        info,
    })
}
