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
    // If there are no actions, the agent has failed.
    if actions.is_empty() {
        let error_string = "Agent returned no actions to execute.".to_string();
        error!("{}", error_string);
        let obs = observation::get_observation(
            env,
            ground_truth,
            "Failure",
            Some(error_string.clone()),
            vec![],
        )?;
        return Ok(Step {
            observation: obs,
            reward: 0.0,
            terminated: true,
            truncated: false,
            info: json!({ "error": error_string }),
        });
    }

    // 1. Collect all instructions from the Vec<AgentAction>.
    let instructions: Vec<Instruction> = actions.into_iter().map(|a| a.0).collect();

    // 2. Aggregate all required signers from all instructions.
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
                    // Avoid adding the fee payer twice.
                    if !signers.iter().any(|s| s.pubkey() == keypair.pubkey()) {
                        signers.push(keypair);
                    }
                } else if acc.pubkey != fee_payer_keypair.pubkey() {
                    // This warning helps debug cases where a signer is genuinely missing.
                    warn!(
                        "Signer keypair for pubkey {} not found in keypair_map. Transaction may fail.",
                        acc.pubkey
                    );
                }
            }
        }
    }

    // 3. Create a single transaction with all instructions, paid for by the fee_payer.
    let mut transaction =
        Transaction::new_with_payer(&instructions, Some(&fee_payer_keypair.pubkey()));

    info!(
        "Executing transaction with {} instruction(s).",
        instructions.len()
    );

    // --- Simulation Logic ---
    info!("Simulating transaction before sending...");
    let sim_config = RpcSimulateTransactionConfig {
        sig_verify: false, // Signatures are not required for simulation.
        replace_recent_blockhash: true,
        commitment: Some(env.rpc_client.commitment()),
        ..Default::default()
    };
    let sim_result = env
        .rpc_client
        .simulate_transaction_with_config(&transaction, sim_config)?;

    let sim_logs = sim_result.value.logs.clone().unwrap_or_default();
    info!(simulation_logs = ?sim_logs, "Transaction simulation logs");

    // --- End Simulation Logic ---

    // 4. The result of the simulation determines the outcome.
    if let Some(err) = sim_result.value.err {
        // If simulation fails, the state has not changed.
        let error_string = format!("Transaction simulation failed: {err}");
        error!("{}", error_string);
        let obs = observation::get_observation(
            env,
            ground_truth,
            "Failure",
            Some(error_string.clone()),
            sim_logs,
        )?;
        Ok(Step {
            observation: obs,
            reward: 0.0,
            terminated: true,
            truncated: false,
            info: json!({ "error": error_string }),
        })
    } else {
        // If simulation succeeds, execute the transaction to persist the state change.
        info!("Transaction simulation successful. Executing transaction...");
        let latest_blockhash = env.rpc_client.get_latest_blockhash()?;
        transaction.sign(&signers, latest_blockhash);

        match env.rpc_client.send_and_confirm_transaction(&transaction) {
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
                let obs = observation::get_observation(env, ground_truth, "Success", None, logs)?;
                Ok(Step {
                    observation: obs,
                    reward: 1.0,
                    terminated: true,
                    truncated: false,
                    info,
                })
            }
            Err(e) => {
                let error_string =
                    format!("Transaction execution failed after successful simulation: {e}");
                error!("{}", error_string);
                let obs = observation::get_observation(
                    env,
                    ground_truth,
                    "Failure",
                    Some(error_string.clone()),
                    sim_logs,
                )?;
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
}
