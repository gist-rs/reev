//! Defines the data structures for loading and representing a benchmark test case.
//!
//! This module contains the structs that correspond to the YAML format of a benchmark
//! file. `serde_yaml` is used to deserialize the file content directly into these
//! strongly-typed structures.

use serde::{Deserialize, Serialize};

/// Represents a complete benchmark test case, deserialized from a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TestCase {
    /// A unique identifier for the benchmark (e.g., "001-SOL-TRANSFER").
    pub id: String,
    /// A brief description of what the benchmark tests.
    pub description: String,
    /// A list of tags for categorizing the benchmark.
    pub tags: Vec<String>,
    /// The on-chain state to be set up before the agent runs.
    pub initial_state: Vec<InitialStateItem>,
    /// The natural language prompt given to the agent.
    pub prompt: String,
    /// The ground truth assertions and expected outcomes for this benchmark.
    pub ground_truth: GroundTruth,
}

/// Defines the initial state of a single on-chain account for a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InitialStateItem {
    /// The placeholder or actual pubkey for the account.
    pub pubkey: String,
    /// The pubkey of the program that owns this account.
    pub owner: String,
    /// The initial lamport balance of the account.
    pub lamports: u64,
    /// Optional data for the account, typically used for SPL token accounts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<SplAccountData>,
}

/// Represents the data field for an SPL token account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SplAccountData {
    /// The mint address of the token.
    pub mint: String,
    /// The pubkey of the wallet that owns this token account.
    pub owner: String,
    /// The initial token balance, as a string.
    pub amount: String,
}

/// The set of ground truth conditions and expected outcomes for a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct GroundTruth {
    /// The expected final status of the transaction (e.g., "Success" or "Failure").
    #[serde(default = "default_transaction_status")]
    pub transaction_status: String,

    /// A list of conditions that must be true on the blockchain after the agent has finished.
    pub final_state_assertions: Vec<StateAssertion>,

    /// The ideal instruction(s) the agent is expected to generate to solve the task.
    /// This is used for calculating instruction accuracy.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "expected_instructions"
    )]
    pub expected_instructions: Vec<BenchmarkInstruction>,
}

/// Provides a default value for `transaction_status` for backward compatibility.
fn default_transaction_status() -> String {
    "Success".to_string()
}

/// An enum representing a single assertion about the final on-chain state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "PascalCase")]
pub enum StateAssertion {
    /// Asserts the final SOL balance of a specific account.
    SolBalance {
        pubkey: String,
        /// The exact expected balance in lamports.
        expected: u64,
    },
    /// Asserts a change in the SOL balance of an account.
    SolBalanceChange {
        /// The pubkey of the account to check.
        pubkey: String,
        /// The expected minimum change in lamports (can be negative).
        expected_change_gte: i64,
    },
    /// Asserts the balance of an SPL token account.
    TokenAccountBalance {
        /// The pubkey of the token account to check.
        pubkey: String,
        /// The exact expected token balance (in the smallest unit).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected: Option<u64>,
        /// The expected minimum token balance.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_gte: Option<u64>,
    },
}

/// A serializable representation of a Solana instruction for use in benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct BenchmarkInstruction {
    /// The pubkey of the program to be executed.
    pub program_id: String,
    /// The accounts required by the instruction.
    pub accounts: Vec<BenchmarkAccountMeta>,
    /// The instruction data, typically as a Base58 string.
    pub data: String,
}

/// A serializable representation of an `AccountMeta` for use in benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct BenchmarkAccountMeta {
    /// The pubkey of the account.
    pub pubkey: String,
    /// `true` if the transaction must be signed by this account's private key.
    pub is_signer: bool,
    /// `true` if the account's data may be mutated by the program.
    pub is_writable: bool,
}
