use crate::agent::{AgentAction, AgentObservation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a single step in the execution trace.
/// It captures the agent's decision-making process and the environment's response.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraceStep {
    /// The agent's internal thought process or plan before taking the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    /// The action the agent decided to take.
    pub action: AgentAction,
    /// The observation returned by the environment after the action was executed.
    pub observation: AgentObservation,
    /// The diagnostic information returned by the environment's step function.
    pub info: Value,
}

/// Represents the complete execution trace for a single evaluation episode.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ExecutionTrace {
    /// The initial prompt given to the agent for the task.
    pub prompt: String,
    /// A chronological list of all the steps taken during the episode.
    pub steps: Vec<TraceStep>,
}

impl ExecutionTrace {
    /// Creates a new, empty execution trace for a given prompt.
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            steps: Vec::new(),
        }
    }

    /// Adds a new step to the trace.
    pub fn add_step(&mut self, step: TraceStep) {
        self.steps.push(step);
    }
}
