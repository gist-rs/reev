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
use std::{
    collections::HashMap,
    process::{Child, Command},
    thread,
    time::Duration,
};

const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

/// The main environment for interacting with a hermetic Solana test validator.
pub struct SolanaEnv {
    /// The handle to the running `surfpool` process.
    surfpool_process: Option<Child>,
    /// The RPC client for communicating with the `surfpool` instance.
    rpc_client: RpcClient,
    /// A map of placeholder names (e.g., "USER_WALLET_PUBKEY") to their actual `Keypair`.
    keypair_map: HashMap<String, Keypair>,
}

impl SolanaEnv {
    /// Creates a new `SolanaEnv`.
    /// The validator process is not started until `reset` is called.
    pub fn new() -> Result<Self> {
        Ok(Self {
            surfpool_process: None,
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
        for (name, keypair) in &self.keypair_map {
            let pubkey = keypair.pubkey();
            let account = self.rpc_client.get_account(&pubkey)?;
            let account_json = serde_json::json!({
                "lamports": account.lamports,
                "owner": account.owner.to_string(),
                "executable": account.executable,
                "data_len": account.data.len(),
            });
            account_states.insert(name.clone(), account_json);
        }

        Ok(AgentObservation {
            last_transaction_status: status.to_string(),
            last_transaction_error: error,
            last_transaction_logs: logs,
            account_states,
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    #[tracing::instrument(skip_all, name = "env.reset")]
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        println!("[SolanaEnv] Resetting environment...");
        self.close()?; // Ensure any previous surfpool process is terminated.

        println!("[SolanaEnv] Spawning new `surfpool`...");
        let process = Command::new("surfpool")
            .current_dir("surfpool")
            .args(["start"])
            .spawn()
            .context("Failed to spawn `surfpool`. Is it installed in the workspace?")?;
        self.surfpool_process = Some(process);

        // Wait for the RPC server to be ready.
        for _ in 0..30 {
            if self.rpc_client.get_health().is_ok() {
                println!("[SolanaEnv] Validator is healthy.");
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
        if self.rpc_client.get_health().is_err() {
            anyhow::bail!("Surfpool did not become healthy in time.");
        }

        // Poll the RPC endpoint to ensure it's fully ready for requests.
        let mut ready = false;
        for _ in 0..15 {
            // Try for up to 15 seconds
            let test_keypair = Keypair::new();
            if self.rpc_client.get_balance(&test_keypair.pubkey()).is_ok() {
                println!("[SolanaEnv] RPC endpoint is responsive.");
                ready = true;
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
        if !ready {
            anyhow::bail!("Validator RPC did not become responsive in time.");
        }

        self.keypair_map.clear();
        let initial_state_val = options
            .and_then(|v| v.get("initial_state").cloned())
            .context("Benchmark options must include 'initial_state'")?;
        let accounts: Vec<Value> = serde_json::from_value(initial_state_val)?;

        for account_config in accounts {
            let pubkey_placeholder = account_config["pubkey"]
                .as_str()
                .context("Missing 'pubkey' placeholder in account config")?;
            let keypair = Keypair::new();
            let pubkey = keypair.pubkey();
            self.keypair_map
                .insert(pubkey_placeholder.to_string(), keypair);

            let lamports = account_config["lamports"]
                .as_u64()
                .context("Missing 'lamports' in account config")?;
            let sig = self.rpc_client.request_airdrop(&pubkey, lamports)?;
            self.rpc_client.confirm_transaction(&sig)?;
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
                Err(e) => {
                    println!("  Could not fetch state for {name}: {e}");
                }
            }
        }
        println!("--------------------------------\n");
    }

    fn close(&mut self) -> Result<()> {
        if let Some(mut child) = self.surfpool_process.take() {
            println!("[SolanaEnv] Terminating validator process...");
            child.kill().context("Failed to kill validator process")?;
            child
                .wait()
                .context("Failed to wait for validator process to exit")?;
        }
        Ok(())
    }
}
