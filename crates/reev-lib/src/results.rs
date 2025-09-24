use crate::{benchmark::TestCase, metrics::QuantitativeScores, trace::ExecutionTrace};
use serde::{Deserialize, Serialize};

/// Represents the final, comprehensive result of a single test case execution.
/// This struct is designed to be serialized to YAML.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub id: String,
    pub prompt: String,
    pub final_status: FinalStatus,
    pub metrics: QuantitativeScores,
    pub trace: ExecutionTrace,
}

impl TestResult {
    /// Creates a new `TestResult` from a `TestCase` and other computed outcomes.
    pub fn new(
        test_case: &TestCase,
        final_status: FinalStatus,
        metrics: QuantitativeScores,
        trace: ExecutionTrace,
    ) -> Self {
        Self {
            id: test_case.id.clone(),
            prompt: test_case.prompt.clone(),
            final_status,
            metrics,
            trace,
        }
    }
}

/// The final pass/fail status of a test case.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FinalStatus {
    Succeeded,
    Failed,
}
