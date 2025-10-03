use crate::{benchmark::TestCase, trace::ExecutionTrace};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Represents the final, high-level status of a test case run.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum FinalStatus {
    Succeeded,
    Failed,
}

impl Display for FinalStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalStatus::Succeeded => write!(f, "Succeeded"),
            FinalStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// A comprehensive summary of the results of a single test case evaluation.
/// This struct is designed to be serialized to YAML to serve as the canonical,
/// machine-readable output for a `reev` run.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    /// The unique identifier of the test case, from the benchmark file.
    pub id: String,
    /// The natural language prompt that was given to the agent.
    pub prompt: String,
    /// The final, high-level status of the run.
    pub final_status: FinalStatus,
    /// The final score of the run, between 0.0 and 1.0.
    pub score: f64,
    /// The complete, step-by-step record of the agent's actions and the environment's responses.
    pub trace: ExecutionTrace,
}

impl TestResult {
    /// Constructs a new `TestResult` from the components of a completed run.
    pub fn new(
        test_case: &TestCase,
        final_status: FinalStatus,
        score: f64,
        trace: ExecutionTrace,
    ) -> Self {
        Self {
            id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            final_status,
            score,
            trace,
        }
    }
}
