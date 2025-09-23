use crate::{
    actions::{
        Action, sol_transfer::SolTransferAction, spl_token_transfer::SplTokenTransferAction,
        token_2022_transfer::Token2022TransferAction,
    },
    agent::{AgentAction, AgentObservation},
    benchmark::{GroundTruth, InitialAccountState},
    env::{GymEnv, Step},
    metrics::calculate_task_success_rate,
};
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpLQtRect";

/// Represents the specific data for a mocked SPL-Token account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SplTokenState {
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

/// An enum representing the different types of account data in the mock state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum MockAccountData {
    System,
    SplToken(SplTokenState),
}

/// Represents the full state of a single account in the mocked, in-memory environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockAccountState {
    pub lamports: u64,
    pub owner: String, // The program that owns this account
    pub data: MockAccountData,
}

/// A mocked Solana environment that simulates on-chain state in memory.
pub struct SolanaEnv {
    /// The in-memory key/value store representing the Solana ledger.
    pub state: HashMap<String, MockAccountState>,
}

impl SolanaEnv {
    pub fn new() -> Result<Self> {
        Ok(Self {
            state: HashMap::new(),
        })
    }
}

impl GymEnv for SolanaEnv {
    type Action = AgentAction;
    type Observation = AgentObservation;

    fn reset(&mut self, _seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation> {
        self.state.clear();
        let initial_states: Vec<InitialAccountState> = if let Some(options_value) = options {
            serde_json::from_value(options_value)?
        } else {
            vec![]
        };

        println!(
            "[MockSolanaEnv] Resetting environment with {} initial accounts...",
            initial_states.len()
        );

        for spec in initial_states {
            let data = if (spec.owner == SPL_TOKEN_PROGRAM_ID
                || spec.owner == TOKEN_2022_PROGRAM_ID)
                && spec.data.is_some()
            {
                let token_data_str = spec.data.as_deref().unwrap(); // Safe due to is_some() check
                // We expect the `data` field in the benchmark YAML to be a JSON string
                // representing the SplTokenState.
                let token_state: SplTokenState = serde_json::from_str(token_data_str).context(
                    format!("Failed to parse SPL Token data for {}", spec.pubkey),
                )?;
                MockAccountData::SplToken(token_state)
            } else {
                // Treat as a system account if not owned by token program or if it has no data (like a mint)
                MockAccountData::System
            };

            let account_state = MockAccountState {
                lamports: spec.lamports,
                owner: spec.owner,
                data,
            };
            self.state.insert(spec.pubkey.clone(), account_state);
        }

        let mut account_states = HashMap::new();
        for (pubkey, account) in &self.state {
            account_states.insert(pubkey.clone(), serde_json::to_value(account)?);
        }

        Ok(AgentObservation {
            last_transaction_status: "Success".to_string(),
            last_transaction_error: None,
            last_transaction_logs: vec!["Environment reset.".to_string()],
            account_states,
        })
    }

    fn step(
        &mut self,
        action: Self::Action,
        ground_truth: &GroundTruth,
    ) -> Result<Step<Self::Observation>> {
        println!("[MockSolanaEnv] Dispatching action: {}", action.tool_name);

        if action.tool_name == "no_op" {
            let mut account_states = HashMap::new();
            for (pubkey, account) in &self.state {
                account_states.insert(pubkey.clone(), serde_json::to_value(account)?);
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
                terminated: true,
                truncated: false,
                info: serde_json::json!({}),
            });
        }

        let handler: Box<dyn Action> = match action.tool_name.as_str() {
            "sol_transfer" => Box::new(SolTransferAction),
            "spl_token_transfer" | "nft_transfer" => Box::new(SplTokenTransferAction),
            "token_2022_transfer" => Box::new(Token2022TransferAction),
            _ => return Err(anyhow!("Unknown tool name: '{}'", action.tool_name)),
        };

        let params_as_value = serde_json::to_value(&action.parameters)?;
        let result = handler.execute(&mut self.state, &params_as_value);

        let mut account_states = HashMap::new();
        for (pubkey, account) in &self.state {
            account_states.insert(pubkey.clone(), serde_json::to_value(account)?);
        }

        let (status, error, logs, mut reward, mut terminated) = match result {
            Ok(_) => (
                "Success".to_string(),
                None,
                vec![format!(
                    "Action '{}' executed successfully.",
                    action.tool_name
                )],
                0.0,
                false,
            ),
            Err(ref e) => (
                "Failure".to_string(),
                Some(e.to_string()),
                vec![format!("Action '{}' failed: {}", action.tool_name, e)],
                -1.0,
                true,
            ),
        };

        let observation = AgentObservation {
            last_transaction_status: status,
            last_transaction_error: error,
            last_transaction_logs: logs,
            account_states,
        };

        if result.is_ok() && calculate_task_success_rate(&observation, ground_truth)? == 1.0 {
            println!("[MockSolanaEnv] Task success conditions met. Terminating episode.");
            terminated = true;
            reward = 1.0;
        }

        Ok(Step {
            observation,
            reward,
            terminated,
            truncated: false,
            info: serde_json::json!({}),
        })
    }

    fn render(&self) {
        println!("\n--- MockSolanaEnv State ---");
        if self.state.is_empty() {
            println!("  (empty)");
        } else {
            for (pubkey, account) in &self.state {
                println!("  - Pubkey: {pubkey}");
                println!("    Lamports: {}", account.lamports);
                println!("    Owner: {}", account.owner);
                println!(
                    "    Data: {}",
                    serde_json::to_string(&account.data).unwrap_or_default()
                );
            }
        }
        println!("---------------------------\n");
    }

    fn close(&mut self) {
        // No-op for the in-memory environment.
    }
}
