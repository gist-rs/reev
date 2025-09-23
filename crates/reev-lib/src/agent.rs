use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use anyhow::Result;

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

/// A simple agent that returns a hardcoded action, useful for testing the evaluation loop.
pub struct DummyAgent;

impl Agent for DummyAgent {
    fn get_action(&mut self, _observation: &AgentObservation) -> Result<AgentAction> {
        // This agent always returns a "no-op" action, regardless of the observation.
        Ok(AgentAction {
            tool_name: "no_op".to_string(),
            parameters: HashMap::new(),
        })
    }
}
