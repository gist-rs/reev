use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::GroundTruth,
    env::{GymEnv, Step},
};
use anyhow::{Context, Result};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::{collections::HashMap, thread, time::Duration};

const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

/// The main environment for interacting with a hermetic Solana test validator.
/// It connects to an externally managed `surfpool` instance.
pub struct SolanaEnv {
    /// The RPC client for communicating with the `surfpool` instance.
    rpc_client: RpcClient,
    /// A map of placeholder names (e.g., "USER_WALLET_PUBKEY") to their actual `Keypair`.
    keypair_map: HashMap<String, Keypair>,
}

impl SolanaEnv {
    /// Creates a new `SolanaEnv`.
    /// This assumes a `surfpool` validator is already running and accessible.
    pub fn new() -> Result<Self> {
        Ok(Self {
            rpc_client: RpcClient::new_with_commitment(
                LOCAL_SURFPOOL_RPC_URL.to_string(),
                CommitmentConfig::confirmed(),
            ),
            keypair_map: HashMap::new(),
        })
    }

    /// Gathers the current state of relevant accounts to form an observation.
    fn get_observation(
        &self,
        status: &str,
        error: Option<String>,
        logs: Vec<String>,
    ) -> Result<AgentObservation> {
        let mut account_states = HashMap::new();
        let mut key_map = HashMap::new();

        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            // Always include the key in the key_map for full context.
            key_map.insert(name.clone(), pubkey.to_string());

            // Only include the state if the account actually exists on-chain.
            if let Ok(account) = self.rpc_client.get_account(&pubkey) {
                let account_json = serde_json::json!({
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "executable": account.executable,
                    "data_len": account.data.len(),
                });
                account_states.insert(name.clone(), account_json);
            }
        }

        Ok(AgentObservation {
            last_transaction_status: status.to_string(),
            last_transaction_error: error,
            last_transaction_logs: logs,
            account_states,
            key_map,
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    #[tracing::instrument(skip_all, name = "env.reset")]
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        println!("[SolanaEnv] Resetting environment...");

        // Check for the RPC server to be ready.
        println!("[SolanaEnv] Checking for running `surfpool` validator...");
        for i in 0..10 {
            if self.rpc_client.get_health().is_ok() {
                break;
            }
            if i == 9 {
                anyhow::bail!("Could not connect to `surfpool` validator. Is it running at {LOCAL_SURFPOOL_RPC_URL}?");
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("[SolanaEnv] Validator is healthy.");

        self.keypair_map.clear();
        let initial_state_val = options
            .and_then(|v| v.get("initial_state").cloned())
            .context("Benchmark options must include 'initial_state'")?;
        let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

        // First pass: create all keypairs so the key_map is complete from the start.
        for account_config in &accounts {
            let pubkey_placeholder = account_config["pubkey"]
                .as_str()
                .context("Missing 'pubkey' placeholder in account config")?;
            let keypair = Keypair::new();
            self.keypair_map
                .insert(pubkey_placeholder.to_string(), keypair);
        }

        // Second pass: airdrop funds where necessary.
        for account_config in accounts {
            let pubkey_placeholder = account_config["pubkey"].as_str().unwrap();
            let keypair = self.keypair_map.get(pubkey_placeholder).unwrap();
            let pubkey = keypair.pubkey();

            let lamports = account_config["lamports"]
                .as_u64()
                .context("Missing 'lamports' in account config")?;

            if lamports > 0 {
                println!(
                    "[SolanaEnv] Attempting to airdrop {lamports} lamports to {pubkey_placeholder} ({pubkey})..."
                );
                match self.rpc_client.request_airdrop(&pubkey, lamports) {
                    Ok(sig) => {
                        if let Err(e) = self.rpc_client.confirm_transaction(&sig) {
                            println!("[SolanaEnv] WARNING: Failed to confirm airdrop transaction: {e}. Continuing...");
                        } else {
                            println!("[SolanaEnv] Airdrop successful.");
                        }
                    }
                    Err(e) => {
                        println!(
                            "[SolanaEnv] WARNING: Airdrop request failed: {e}. Continuing..."
                        );
                    }
                }
            }
        }

        println!("[SolanaEnv] Environment reset complete.");
        self.get_observation("Success", None, vec!["Environment reset.".to_string()])
    }

    #[tracing::instrument(skip(self, action), name = "env.step")]
    fn step(
        &mut self,
        action: Self::Action,
        _ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        let instruction = action.0;
        println!(
            "[SolanaEnv] Executing instruction for program: {}",
            instruction.program_id
        );

        let signer_keypair = instruction
            .accounts
            .iter()
            .find(|acc| acc.is_signer)
            .and_then(|signer_acc| {
                self.keypair_map
                    .values()
                    .find(|kp| kp.pubkey() == signer_acc.pubkey)
            })
            .context("Agent-generated instruction requires a signer that the environment does not control.")?;

        let mut transaction =
            Transaction::new_with_payer(&[instruction], Some(&signer_keypair.pubkey()));

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(&[signer_keypair], recent_blockhash);

        let mut logs = Vec::new();
        let (status, error, terminated) =
            match self.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(sig) => {
                    let sig_str = sig.to_string();
                    logs.push(format!("Transaction confirmed: {sig_str}"));
                    ("Success", None, true)
                }
                Err(e) => {
                    let err_str = e.to_string();
                    ("Failure", Some(err_str), true)
                }
            };

        let observation = self.get_observation(status, error.clone(), logs)?;

        Ok(Step {
            reward: if error.is_none() { 1.0 } else { 0.0 },
            terminated,
            truncated: false,
            info: serde_json::json!({ "error": error }),
            observation,
        })
    }

    fn render(&self) {
        println!("\n--- Current On-Chain State ---");
        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            match self.rpc_client.get_account(&pubkey) {
                Ok(account) => {
                    println!(
                        "  Pubkey: {} (Name: {})\n    Owner: {}\n    Lamports: {}",
                        pubkey, name, account.owner, account.lamports
                    );
                }
                Err(_) => {
                    println!(
                        "  Pubkey: {pubkey} (Name: {name})\n    Account not found on-chain."
                    );
                }
            }
        }
        println!("--------------------------------\n");
    }

    fn close(&mut self) -> Result<()> {
        println!("[SolanaEnv] Environment closed. Validator process is left running.");
        Ok(())
    }
}
