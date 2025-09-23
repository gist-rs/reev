use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents an action taken by the agent, typically a tool call.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentAction {
    /// The name of the tool to be invoked.
    pub tool_name: String,
    /// The parameters to be passed to the tool, represented as a flexible JSON map.
    pub parameters: HashMap<String, Value>,
}

/// Represents the observation of the environment state provided to the agent.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentObservation {
    /// The status of the last transaction executed ("Success" or "Failure").
    pub last_transaction_status: String,
    /// An optional error message if the last transaction failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transaction_error: Option<String>,
    /// Logs emitted by the last transaction.
    pub last_transaction_logs: Vec<String>,
    /// The current state of relevant on-chain accounts.
    /// The key is the account's public key (as a string), and the value is its state (e.g., balance, data).
    pub account_states: HashMap<String, Value>,
}

/// A trait that defines the interface for an LLM agent.
pub trait Agent {
    /// Takes an observation from the environment and returns the next action to take.
    ///
    /// # Arguments
    /// * `observation`: The current state of the environment.
    ///
    /// # Returns
    /// The `AgentAction` to be executed, or an error if the agent fails to decide on an action.
    fn get_action(&mut self, observation: &AgentObservation) -> Result<AgentAction>;
}

/// A simple, stateful agent that performs a single hardcoded action and then stops.
pub struct DummyAgent {
    has_acted: bool,
}

impl DummyAgent {
    /// Creates a new `DummyAgent`.
    pub fn new() -> Self {
        Self { has_acted: false }
    }
}

impl Agent for DummyAgent {
    fn get_action(&mut self, _observation: &AgentObservation) -> Result<AgentAction> {
        if !self.has_acted {
            // If the agent hasn't acted yet, perform the SOL transfer.
            self.has_acted = true;
            println!("[DummyAgent] Deciding to perform the SOL transfer.");
            let mut parameters = HashMap::new();
            parameters.insert(
                "from_pubkey".to_string(),
                serde_json::json!("USER_WALLET_PUBKEY"),
            );
            parameters.insert(
                "to_pubkey".to_string(),
                serde_json::json!("RECIPIENT_WALLET_PUBKEY"),
            );
            parameters.insert("lamports".to_string(), serde_json::json!(100_000_000));

            Ok(AgentAction {
                tool_name: "sol_transfer".to_string(),
                parameters,
            })
        } else {
            // If the agent has already acted, it considers its job done.
            println!("[DummyAgent] Already acted, deciding to do nothing (no_op).");
            Ok(AgentAction {
                tool_name: "no_op".to_string(),
                parameters: HashMap::new(),
            })
        }
    }
}
