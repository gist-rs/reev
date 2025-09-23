use crate::{
    agent::{AgentAction, AgentObservation},
    env::{GymEnv, Step},
};
use anyhow::Result;
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::Keypair;
use std::process::Child;

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

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    fn reset(&mut self, _seed: Option<u64>, _options: Option<Value>) -> Result<Self::Observation> {
        // 1. Kill any existing validator process.
        // 2. Start a new `solana-test-validator` instance using `std::process::Command`.
        // 3. Wait for the validator to be responsive.
        // 4. Load the initial on-chain state based on the `options`.
        // 5. Fetch the initial state and return the first `AgentObservation`.
        todo!();
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
