use crate::agent::AgentAction;
use serde::{Deserialize, Serialize};

/// Defines a specific condition that must be true on the blockchain
/// after the agent has completed its task.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum StateAssertion {
    /// Asserts the final SOL balance of a specific account.
    SolBalance {
        pubkey: String,
        /// The exact expected balance in lamports.
        expected: u64,
    },
    /// Asserts the final token balance of a SPL Token account.
    TokenAccountBalance {
        pubkey: String,
        /// The exact expected token balance (in the smallest unit, e.g., token atoms).
        expected: u64,
    },
    /// Asserts that the SOL balance of an account has changed by at least a certain amount.
    /// This is useful for tasks where the exact final balance is unknown due to fees.
    SolBalanceChange {
        pubkey: String,
        /// The expected minimum change in lamports (can be negative).
        expected_change_gte: i64,
    },
}

/// Defines the initial state of a single on-chain account for a test case.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InitialAccountState {
    /// A unique identifier or the public key for the account.
    pub pubkey: String,
    /// The initial SOL balance in lamports. Defaults to 0.
    #[serde(default)]
    pub lamports: u64,
    /// The public key of the account's owner program.
    pub owner: String,
    /// Optional base64 encoded data for the account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Specifies if the account is an executable program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_executable: Option<bool>,
    /// Path to a file containing the account data (e.g., a compiled program `.so` file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_from_file: Option<String>,
}

/// Contains the objective criteria for judging the agent's performance on a test case.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GroundTruth {
    /// A list of conditions that must be true on the blockchain after the agent has finished.
    pub final_state_assertions: Vec<StateAssertion>,
    /// An optional, ordered list of the ideal tool calls the agent should make.
    /// This is used for calculating metrics like Tool Selection Accuracy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_tool_calls: Vec<AgentAction>,
}

/// Represents a single, self-contained test case for evaluating an agent.
/// This struct is designed to be deserialized from a YAML file.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestCase {
    /// A unique identifier for the test case (e.g., "TRANSFER-SIMPLE-001").
    pub id: String,
    /// A human-readable description of the task's objective.
    pub description: String,
    /// A list of tags for categorization (e.g., ["token-program", "t1", "t2"]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// A declarative definition of the on-chain state required at the beginning of the test.
    pub initial_state: Vec<InitialAccountState>,
    /// The natural language prompt that is given to the agent as its instruction.
    pub prompt: String,
    /// The ground truth criteria used to evaluate the agent's success on this test case.
    pub ground_truth: GroundTruth,
}
