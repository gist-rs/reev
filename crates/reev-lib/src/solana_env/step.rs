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

use tracing::{error, info};

pub(crate) fn handle_step(
    env: &mut SolanaEnv,
    actions: Vec<AgentAction>,
    ground_truth: &GroundTruth,
) -> Result<Step<AgentObservation>> {
    // --- 1. Define variables to hold the outcome of the transaction ---
    let mut tx_status = "Failure";
    let mut tx_error: Option<String> = None;
    let mut tx_logs: Vec<String> = Vec::new();
    let mut info = json!({});
    let mut reward = 0.0;

    // --- 2. Handle API benchmarks (skip_instruction_validation) ---
    if ground_truth.skip_instruction_validation {
        // For API benchmarks, success is determined by making tool calls, not transactions
        if actions.is_empty() {
            let error_string = "Agent returned no actions for API benchmark.".to_string();
            error!("{}", error_string);
            tx_error = Some(error_string.clone());
            info = json!({ "error": error_string, "type": "api_benchmark" });
        } else {
            // Check if this is a flow benchmark by looking at the instruction data
            let is_flow_benchmark = actions
                .first()
                .and_then(|action| action.0.data.first())
                .map(|&byte| byte == 2) // Flow success indicator
                .unwrap_or(false);

            if is_flow_benchmark {
                tx_status = "Success";
                info = json!({
                    "message": "Flow benchmark completed successfully",
                    "type": "flow_benchmark",
                    "actions_count": actions.len()
                });
            } else {
                // For regular API benchmarks, any action indicates success
                tx_status = "Success";
                info = json!({
                    "message": "API benchmark completed - tool calls made",
                    "type": "api_benchmark",
                    "actions_count": actions.len()
                });
            }
            reward = 1.0;
        }
    } else if actions.is_empty() {
        // --- 3. Handle the case of no actions for transaction benchmarks ---
        let error_string = "Agent returned no actions to execute.".to_string();
        error!("{}", error_string);
        tx_error = Some(error_string.clone());
        info = json!({ "error": error_string });
        reward = 0.0; // Explicitly set reward for empty actions
    } else {
        // --- 4. Build and simulate the transaction for non-API benchmarks ---
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
                // --- 6. If simulation succeeds, execute the transaction ---
                info!("Transaction simulation successful. Executing transaction...");
                let latest_blockhash = env.rpc_client.get_latest_blockhash()?;
                transaction.sign(&signers, latest_blockhash);

                // Execute the transaction
                match env.rpc_client.send_and_confirm_transaction(&transaction) {
                    Ok(sig) => {
                        info!("Transaction executed successfully: {}", sig.to_string());
                        tx_status = "Success";
                        reward = 1.0;
                        // Use simulation logs since get_transaction might fail
                        tx_logs = sim_logs.clone();
                    }
                    Err(e) => {
                        let error_string = format!(
                            "Transaction execution failed after successful simulation: {e}"
                        );
                        error!("{}", error_string);
                        tx_error = Some(error_string.clone());
                        tx_logs = sim_logs; // On execution failure, we only have simulation logs
                        info = json!({ "error": error_string });
                    }
                }
            }
        }
    }

    // --- 7. Get the final observation AFTER the transaction has settled ---
    let obs = observation::get_observation(env, ground_truth, tx_status, tx_error, tx_logs)?;

    // --- 8. Return the final step result ---
    Ok(Step {
        observation: obs,
        reward,
        terminated: true,
        truncated: false,
        info,
    })
}
