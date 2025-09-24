use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents an action taken by the agent, typically a tool call.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentAction {
    pub tool_name: String,
    pub parameters: HashMap<String, Value>,
}

/// Represents the observation of the environment state provided to the agent.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentObservation {
    pub last_transaction_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transaction_error: Option<String>,
    pub last_transaction_logs: Vec<String>,
    pub account_states: HashMap<String, Value>,
}

/// A trait that defines the interface for an LLM agent.
pub trait Agent {
    /// Takes an observation from the environment and returns the next action to take.
    fn get_action(&mut self, observation: &AgentObservation) -> Result<AgentAction>;
}

/// A simple, stateful agent that performs a single hardcoded action and then stops.
pub struct DummyAgent {
    has_acted: bool,
}

impl Default for DummyAgent {
    fn default() -> Self {
        Self::new()
    }
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
            // If the agent hasn't acted yet, perform the SPL transfer.
            self.has_acted = true;
            println!("[DummyAgent] Deciding to perform the SPL transfer.");
            let mut parameters = HashMap::new();
            // These parameters should match the spl-transfer-001.yml benchmark.
            parameters.insert(
                "from_pubkey".to_string(),
                serde_json::json!("USER_USDC_ATA"),
            );
            parameters.insert(
                "to_pubkey".to_string(),
                serde_json::json!("RECIPIENT_USDC_ATA"),
            );
            parameters.insert(
                "authority_pubkey".to_string(),
                serde_json::json!("USER_WALLET_PUBKEY"),
            );
            // 15 USDC with 6 decimals.
            parameters.insert("amount".to_string(), serde_json::json!(15_000_000));

            Ok(AgentAction {
                tool_name: "spl_transfer".to_string(),
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
