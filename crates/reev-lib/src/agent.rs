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
