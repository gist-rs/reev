use crate::{
    actions,
    agent::{AgentAction, AgentObservation},
    benchmark::{GroundTruth, InitialAccountState},
    env::{GymEnv, Step},
};
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use serde_json::Value;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_sdk::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token::state::{Account as SplTokenAccount, AccountState, Mint};
use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};
use tracing::instrument;

/// The default RPC URL for a locally running `surfpool` instance.
const LOCAL_SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";

/// A struct used for serializing the parameters for the `surfnet_setAccount` RPC call.
#[derive(Serialize)]
struct SetAccountParams {
    lamports: u64,
    owner: String,
    executable: bool,
    data: String, // Hex encoded data
}

/// A Solana environment that manages an external `surfpool` process as a self-contained, black-box service.
pub struct SolanaEnv {
    /// The handle to the running `surfpool start` child process.
    surfpool_process: Option<Child>,
    /// The RPC client for communicating with the `surfpool` testnet.
    rpc_client: RpcClient,
    /// Maps placeholder strings from benchmarks (e.g., "USER_WALLET_PUBKEY") to their
    /// real, randomly generated Keypairs for the current test case.
    keypair_map: HashMap<String, Keypair>,
}

impl Default for SolanaEnv {
    fn default() -> Self {
        Self::new().expect("Failed to create SolanaEnv")
    }
}

impl SolanaEnv {
    /// Creates a new, uninitialized `SolanaEnv`.
    pub fn new() -> Result<Self> {
        Ok(Self {
            surfpool_process: None,
            rpc_client: RpcClient::new(LOCAL_SURFPOOL_RPC_URL.to_string()),
            keypair_map: HashMap::new(),
        })
    }

    /// A helper function to construct the `AgentObservation` from the current on-chain state.
    fn get_observation(
        &self,
        status: &str,
        error: Option<String>,
        logs: Vec<String>,
    ) -> Result<AgentObservation> {
        let mut account_states = HashMap::new();
        for (placeholder, keypair) in &self.keypair_map {
            let account = self
                .rpc_client
                .get_account(&keypair.pubkey())
                .context(format!(
                    "Failed to get account for placeholder '{placeholder}'"
                ))?;
            let value = serde_json::to_value(account)?;
            account_states.insert(placeholder.clone(), value);
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

    /// Resets the environment by:
    /// 1. Terminating any existing `surfpool` process.
    /// 2. Spawning a new, clean `surfpool start` process.
    /// 3. Waiting for the process to become responsive.
    /// 4. Generating local keypairs for all accounts in the benchmark's `initial_state`.
    /// 5. Using the `surfnet_setAccount` RPC "cheatcode" to create and fund these accounts on the new validator.
    #[instrument(skip_all)]
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        self.close()?;
        self.keypair_map.clear();

        let initial_states: Vec<InitialAccountState> =
            serde_json::from_value(options.context("Missing options for reset")?)?;

        // 1. Start the surfpool process
        println!("[SolanaEnv] Spawning `surfpool start` process...");
        let child = Command::new("surfpool")
            .arg("start")
            .stdin(Stdio::null())
            .stdout(Stdio::null()) // Suppress output to keep the runner clean
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start `surfpool`. Is it installed and in your PATH?")?;
        self.surfpool_process = Some(child);

        // 2. Wait for it to be responsive
        println!("[SolanaEnv] Waiting for surfpool to be responsive...");
        for i in 0..30 {
            if self.rpc_client.get_health().is_ok() {
                println!("[SolanaEnv] Surfpool is online.");
                break;
            }
            if i == 29 {
                anyhow::bail!("Surfpool RPC endpoint did not become responsive in time.");
            }
            thread::sleep(Duration::from_secs(1));
        }

        // 3. Generate local keypairs for all accounts defined in the benchmark
        for spec in &initial_states {
            self.keypair_map
                .entry(spec.pubkey.clone())
                .or_insert_with(Keypair::new);
        }

        // 4. Use the `surfnet_setAccount` cheatcode to configure the initial on-chain state.
        println!("[SolanaEnv] Configuring initial on-chain state via RPC...");
        for spec in &initial_states {
            let keypair = self.keypair_map.get(&spec.pubkey).unwrap();
            // Handle account data. The benchmark can provide data in several ways.
            let hex_data = if let Some(mint_data) = &spec.mint_data {
                // This account should be initialized as an SPL Token Mint.
                let mint_authority_placeholder = mint_data
                    .mint_authority
                    .as_deref()
                    .unwrap_or("USER_WALLET_PUBKEY");

                let mint_authority_pubkey = self
                    .keypair_map
                    .get(mint_authority_placeholder)
                    .with_context(|| {
                        format!(
                            "Mint authority placeholder '{mint_authority_placeholder}' not found"
                        )
                    })?
                    .pubkey();

                let spl_mint = Mint {
                    mint_authority: Some(mint_authority_pubkey).into(),
                    supply: 0, // Mints start with 0 supply
                    decimals: mint_data.decimals,
                    is_initialized: true,
                    freeze_authority: None.into(),
                };

                let mut mint_data_bytes = vec![0; Mint::LEN];
                spl_mint.pack_into_slice(&mut mint_data_bytes);
                hex::encode(mint_data_bytes)
            } else if let Some(data_str) = &spec.data {
                // Handle token accounts or raw base64 data.
                if let Ok(json_val) = serde_json::from_str::<Value>(data_str) {
                    // It's a JSON string, assume it's an SPL Token Account.
                    #[derive(serde::Deserialize)]
                    struct SplTokenData {
                        mint: String,
                        owner: String,
                        amount: u64,
                    }
                    let token_data: SplTokenData = serde_json::from_value(json_val)
                        .context("Failed to deserialize SPL token data from benchmark spec")?;

                    let mint_pubkey = self
                        .keypair_map
                        .get(&token_data.mint)
                        .with_context(|| {
                            format!("Mint placeholder '{}' not found", token_data.mint)
                        })?
                        .pubkey();
                    let owner_pubkey = self
                        .keypair_map
                        .get(&token_data.owner)
                        .with_context(|| {
                            format!("Owner placeholder '{}' not found", token_data.owner)
                        })?
                        .pubkey();

                    let spl_account = SplTokenAccount {
                        mint: mint_pubkey,
                        owner: owner_pubkey,
                        amount: token_data.amount,
                        state: AccountState::Initialized,
                        ..Default::default()
                    };

                    let mut account_data_bytes = vec![0; SplTokenAccount::LEN];
                    spl_account.pack_into_slice(&mut account_data_bytes);
                    hex::encode(account_data_bytes)
                } else {
                    // Not valid JSON, so assume it's a base64 encoded string.
                    let data_bytes = STANDARD
                        .decode(data_str)
                        .context("Failed to decode base64 account data from benchmark spec")?;
                    hex::encode(data_bytes)
                }
            } else {
                // No data field provided.
                String::new()
            };

            let params = SetAccountParams {
                lamports: spec.lamports,
                owner: spec.owner.clone(),
                executable: spec.is_executable.unwrap_or(false),
                data: hex_data,
            };

            let rpc_params = vec![
                serde_json::to_value(keypair.pubkey().to_string())?,
                serde_json::to_value(params)?,
            ];

            // Use the generic `send` method for custom RPC calls.
            self.rpc_client
                .send::<serde_json::Value>(
                    RpcRequest::Custom {
                        method: "surfnet_setAccount",
                    },
                    serde_json::Value::Array(rpc_params),
                )
                .context(format!("Failed to set account state for '{}'", spec.pubkey))?;
        }
        println!("[SolanaEnv] Initial state configured.");

        // 5. Return the initial observation of the fully set up state
        self.get_observation("Success", None, vec!["Environment reset.".to_string()])
    }

    /// Executes a single step in the environment based on the agent's action.
    #[instrument(skip(self, _ground_truth), fields(tool_name = %action.tool_name))]
    fn step(
        &mut self,
        action: Self::Action,
        _ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        let mut terminated = false;
        let mut error: Option<String> = None;
        let mut logs: Vec<String> = vec![];

        match action.tool_name.as_str() {
            "sol_transfer" => {
                println!("[SolanaEnv] Executing 'sol_transfer' action...");
                let pubkey_map: HashMap<String, Pubkey> = self
                    .keypair_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.pubkey()))
                    .collect();
                let mut transaction =
                    actions::sol_transfer::build_transaction(&action.parameters, &pubkey_map)?;

                let from_pubkey_placeholder = action
                    .parameters
                    .get("from_pubkey")
                    .and_then(|v| v.as_str())
                    .context("Missing 'from_pubkey' in parameters")?;
                let from_keypair = self
                    .keypair_map
                    .get(from_pubkey_placeholder)
                    .context("Signer keypair not found")?;

                let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
                transaction.sign(&[from_keypair], recent_blockhash);

                match self.rpc_client.send_and_confirm_transaction(&transaction) {
                    Ok(sig) => {
                        let sig_str = sig.to_string();
                        println!("[SolanaEnv] Transaction successful: {sig_str}");
                        logs.push(format!("Transaction confirmed: {sig_str}"));
                    }
                    Err(e) => {
                        println!("[SolanaEnv] Transaction failed: {e}");
                        error = Some(e.to_string());
                        terminated = true; // Fail fast on transaction error
                    }
                }
            }
            "spl_transfer" => {
                println!("[SolanaEnv] Executing 'spl_transfer' action...");
                let pubkey_map: HashMap<String, Pubkey> = self
                    .keypair_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.pubkey()))
                    .collect();
                let mut transaction =
                    actions::spl_transfer::build_transaction(&action.parameters, &pubkey_map)?;

                // For SPL transfers, the signing authority is the "owner" of the token account.
                let authority_pubkey_placeholder = action
                    .parameters
                    .get("authority_pubkey")
                    .and_then(|v| v.as_str())
                    .context("Missing 'authority_pubkey' in parameters")?;
                let authority_keypair = self
                    .keypair_map
                    .get(authority_pubkey_placeholder)
                    .context("Signer keypair not found")?;

                let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
                transaction.sign(&[authority_keypair], recent_blockhash);

                match self.rpc_client.send_and_confirm_transaction(&transaction) {
                    Ok(sig) => {
                        let sig_str = sig.to_string();
                        println!("[SolanaEnv] Transaction successful: {sig_str}");
                        logs.push(format!("Transaction confirmed: {sig_str}"));
                    }
                    Err(e) => {
                        println!("[SolanaEnv] Transaction failed: {e}");
                        error = Some(e.to_string());
                        terminated = true; // Fail fast on transaction error
                    }
                }
            }
            "no_op" => {
                println!("[SolanaEnv] Executing 'no_op' action. Agent considers task finished.");
                logs.push("No operation performed.".to_string());
                terminated = true; // The agent has decided it's done.
            }
            _ => {
                let error_msg = format!("Unknown tool name: '{}'", action.tool_name);
                println!("[SolanaEnv] {error_msg}");
                error = Some(error_msg);
                terminated = true;
            }
        }

        let status = if error.is_some() {
            "Failure"
        } else {
            "Success"
        };
        let observation = self.get_observation(status, error.clone(), logs)?;

        Ok(Step {
            reward: if terminated && error.is_none() {
                1.0
            } else {
                0.0
            },
            terminated,
            truncated: false,
            info: serde_json::json!({ "error": error }),
            observation,
        })
    }

    /// Renders the current on-chain state of all managed accounts to the console.
    fn render(&self) {
        println!("\n--- SolanaEnv State (On-Chain via RPC) ---");
        if self.keypair_map.is_empty() {
            println!("  No accounts loaded.");
        }
        for (placeholder, keypair) in &self.keypair_map {
            match self.rpc_client.get_balance(&keypair.pubkey()) {
                Ok(balance) => println!(
                    "  - {placeholder} ({}): {} SOL",
                    keypair.pubkey(),
                    (balance as f64) / 1_000_000_000.0
                ),
                Err(e) => println!(
                    "  - {placeholder} ({}): Error fetching balance: {e}",
                    keypair.pubkey()
                ),
            }
        }
        println!("------------------------------------------\n");
    }

    /// Terminates the managed `surfpool` process.
    fn close(&mut self) -> Result<()> {
        if let Some(mut child) = self.surfpool_process.take() {
            println!(
                "[SolanaEnv] Stopping surfpool process (PID: {})...",
                child.id()
            );
            if let Err(e) = child.kill() {
                eprintln!("[SolanaEnv] Warning: Failed to kill surfpool process: {e}");
            }
            if let Err(e) = child.wait() {
                eprintln!("[SolanaEnv] Warning: Failed to wait on surfpool process: {e}");
            }
            println!("[SolanaEnv] Surfpool process stopped.");
        }
        Ok(())
    }
}
