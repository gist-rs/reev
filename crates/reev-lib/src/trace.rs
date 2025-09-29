use crate::agent::{AgentAction, AgentObservation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the full execution trace of an agent for a single benchmark run.
/// It contains the initial prompt and a sequence of steps the agent took.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionTrace {
    /// The natural language prompt that initiated the evaluation.
    pub prompt: String,
    /// The sequence of steps taken by the agent.
    pub steps: Vec<TraceStep>,
}

impl ExecutionTrace {
    /// Creates a new, empty execution trace with the given prompt.
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

/// Represents a single step in the agent's execution, capturing the thought process,
/// the action taken, and the resulting observation from the environment.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraceStep {
    /// The agent's reasoning or thought process before taking the action. (Optional)
    pub thought: Option<String>,
    /// The action the agent decided to take.
    pub action: Vec<AgentAction>,
    /// The observation the environment returned after the action was executed.
    pub observation: AgentObservation,
    /// Any additional information returned by the environment's `step` function.
    pub info: Value,
}
