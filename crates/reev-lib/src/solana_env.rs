use crate::{
    agent::{AgentAction, AgentObservation},
    env::{GymEnv, Step},
};
use anyhow::Result;
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::Keypair;
use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

/// The Solana environment for the LLM agent.
/// Manages the `solana-test-validator` process and communication with it.
pub struct SolanaEnv {
    /// The handle to the running `solana-test-validator` child process.
    validator_process: Option<Child>,
    /// The RPC client for communicating with the validator.
    rpc_client: RpcClient,
    /// The keypair representing the agent's identity on-chain.
    agent_keypair: Keypair,
}

impl SolanaEnv {
    /// Creates a new instance of the SolanaEnv.
    pub fn new() -> Result<Self> {
        Ok(Self {
            validator_process: None,
            // The RPC URL for the local test validator.
            rpc_client: RpcClient::new("http://127.0.0.1:8899".to_string()),
            // A new, ephemeral keypair is generated for the agent for each environment instance.
            agent_keypair: Keypair::new(),
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    fn reset(&mut self, _seed: Option<u64>, _options: Option<Value>) -> Result<Self::Observation> {
        // 1. Kill any existing validator process.
        self.close();

        // 2. Start a new `solana-test-validator` instance.
        let child = Command::new("solana-test-validator")
            .arg("--reset")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        self.validator_process = Some(child);

        // 3. Wait for the validator to be responsive.
        // A simple sleep for now. A more robust solution would be to poll the RPC endpoint.
        thread::sleep(Duration::from_secs(5));

        // 5. Fetch the initial state and return the first `AgentObservation`.
        Ok(AgentObservation {
            last_transaction_status: "Success".to_string(),
            last_transaction_error: None,
            last_transaction_logs: vec![],
            account_states: HashMap::new(),
        })
    }

    fn step(&mut self, _action: Self::Action) -> Result<Step<Self::Observation>> {
        // 1. Take an `AgentAction` and parse it.
        // 2. Construct the appropriate Solana transaction using `solana-sdk`.
        // 3. Sign the transaction with `self.agent_keypair`.
        // 4. Send the transaction using `self.rpc_client`.
        // 5. Wait for confirmation and query the status and logs.
        // 6. Build and return the `Step<AgentObservation>` result.
        todo!();
    }

    fn render(&self) {
        // Print a formatted summary of the current state to the console.
        // (e.g., last transaction signature, key account balances)
        println!("Rendering current state... (Not yet implemented)");
    }

    fn close(&mut self) {
        // Ensure the `validator_process` is properly terminated using its `kill()` method.
        if let Some(mut child) = self.validator_process.take() {
            if let Err(e) = child.kill() {
                eprintln!("Failed to kill validator process: {}", e);
            }
        }
    }
}
