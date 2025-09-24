use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use tracing::instrument;

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

/// A simple, stateful agent that executes a pre-defined sequence of actions.
/// This agent is "dumb" and does not react to observations; it just follows its script.
pub struct DummyAgent {
    action_queue: VecDeque<AgentAction>,
}

impl DummyAgent {
    /// Creates a new `DummyAgent` with a sequence of actions to execute.
    ///
    /// The actions are typically sourced from the `expected_tool_calls` field
    /// of a benchmark's `ground_truth`.
    pub fn new(actions: Vec<AgentAction>) -> Self {
        Self {
            action_queue: VecDeque::from(actions),
        }
    }
}

impl Agent for DummyAgent {
    #[instrument(skip_all, fields(action_in_queue = !self.action_queue.is_empty()))]
    fn get_action(&mut self, _observation: &AgentObservation) -> Result<AgentAction> {
        if let Some(action) = self.action_queue.pop_front() {
            // If there's an action in the queue, perform it.
            println!(
                "[DummyAgent] Executing next action from queue: {}",
                action.tool_name
            );
            Ok(action)
        } else {
            // If the queue is empty, the agent considers its job done.
            println!("[DummyAgent] Action queue is empty. Deciding to do nothing (no_op).");
            Ok(AgentAction {
                tool_name: "no_op".to_string(),
                parameters: HashMap::new(),
            })
        }
    }
}
