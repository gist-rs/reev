use crate::actions::{Action, sol_transfer::SolTransferAction};
use crate::{
    agent::{AgentAction, AgentObservation},
    benchmark::InitialAccountState,
    env::{GymEnv, Step},
    metrics::calculate_task_success_rate,
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the state of a single account in the mocked, in-memory environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAccountState {
    pub lamports: u64,
    pub owner: String,
    pub data: Vec<u8>,
}

/// A mocked Solana environment that simulates on-chain state in memory.
/// This removes the need for a real `solana-test-validator` during development.
pub struct SolanaEnv {
    /// The in-memory key/value store representing the Solana ledger.
    /// Maps a Pubkey (as a String) to its account state.
    state: HashMap<String, MockAccountState>,
}

impl SolanaEnv {
    /// Creates a new, empty mocked environment.
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: HashMap::new(),
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    /// Resets the environment and populates the in-memory state from the benchmark file.
    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        self.state.clear();
        println!("[MockSolanaEnv] In-memory state cleared.");

        let initial_state_specs: Vec<InitialAccountState> = if let Some(options_value) = options {
            serde_json::from_value(options_value)?
        } else {
            vec![]
        };

        if initial_state_specs.is_empty() {
            println!("[MockSolanaEnv] No initial state provided.");
        } else {
            println!(
                "[MockSolanaEnv] Populating in-memory state with {} accounts...",
                initial_state_specs.len()
            );
        }

        for spec in initial_state_specs {
            let account_state = MockAccountState {
                lamports: spec.lamports,
                owner: spec.owner,
                data: spec.data.map_or_else(Vec::new, |d| d.into_bytes()),
            };
            self.state.insert(spec.pubkey, account_state);
        }

        // Construct the initial observation from the newly created state.
        let mut account_states = HashMap::new();
        for (pubkey, account) in &self.state {
            // In the observation, we only expose the lamports for now.
            account_states.insert(pubkey.clone(), serde_json::json!(account.lamports));
        }

        Ok(AgentObservation {
            last_transaction_status: "Success".to_string(),
            last_transaction_error: None,
            last_transaction_logs: vec!["Environment reset successfully.".to_string()],
            account_states,
        })
    }

    /// Executes an action by dispatching to a mocked action handler.
    /// Executes an action by dispatching to the appropriate action handler.
    fn step(
        &mut self,
        action: Self::Action,
        ground_truth: &crate::benchmark::GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        println!("[MockSolanaEnv] Dispatching action: {}", action.tool_name);

        // If the agent takes no action, the episode is considered over from its perspective.
        if action.tool_name == "no_op" {
            let mut account_states = HashMap::new();
            for (pubkey, account) in &self.state {
                account_states.insert(pubkey.clone(), serde_json::json!(account.lamports));
            }
            let observation = AgentObservation {
                last_transaction_status: "Success".to_string(),
                last_transaction_error: None,
                last_transaction_logs: vec!["Agent chose to perform no action.".to_string()],
                account_states,
            };
            return Ok(Step {
                observation,
                reward: 0.0,
                terminated: true, // Terminate the episode.
                truncated: false,
                info: serde_json::json!({}),
            });
        }

        let handler: Box<dyn Action> = match action.tool_name.as_str() {
            "sol_transfer" => Box::new(SolTransferAction),
            _ => {
                return Err(anyhow!("Unknown tool name: '{}'", action.tool_name));
            }
        };

        let params_as_value = serde_json::to_value(&action.parameters)?;
        let result = handler.execute(&mut self.state, &params_as_value);

        // Construct the observation from the updated state regardless of the outcome.
        let mut account_states = HashMap::new();
        for (pubkey, account) in &self.state {
            account_states.insert(pubkey.clone(), serde_json::json!(account.lamports));
        }

        let (status, error, logs, mut reward, mut terminated) = match result {
            Ok(_) => (
                "Success".to_string(),
                None,
                vec![format!(
                    "Action '{}' executed successfully.",
                    action.tool_name
                )],
                0.0,   // Base reward for a successful step, not final success.
                false, // Default to not terminating unless success conditions are met.
            ),
            Err(ref e) => (
                "Failure".to_string(),
                Some(e.to_string()),
                vec![format!("Action '{}' failed: {}", action.tool_name, e)],
                -1.0, // Penalize action failure.
                true, // Terminate on any action failure.
            ),
        };

        let observation = AgentObservation {
            last_transaction_status: status,
            last_transaction_error: error,
            last_transaction_logs: logs,
            account_states,
        };

        // If the action itself didn't fail, check if the task's success criteria are now met.
        if result.is_ok() {
            if calculate_task_success_rate(&observation, ground_truth)? == 1.0 {
                println!("[MockSolanaEnv] Task success conditions met. Terminating episode.");
                terminated = true;
                reward = 1.0; // Final reward for completing the task.
            }
        }

        Ok(Step {
            observation,
            reward,
            terminated,
            truncated: false,
            info: serde_json::json!({}),
        })
    }

    /// Renders a human-readable view of the current in-memory state.
    fn render(&self) {
        println!("\n--- MockSolanaEnv State ---");
        if self.state.is_empty() {
            println!("  (empty)");
        } else {
            for (pubkey, account) in &self.state {
                println!(
                    "  - Pubkey: {}\n    Lamports: {}\n    Owner: {}",
                    pubkey, account.lamports, account.owner
                );
            }
        }
        println!("---------------------------\n");
    }

    /// Cleans up resources. For the mock environment, this does nothing.
    fn close(&mut self) {
        // No-op for the in-memory environment.
    }
}
