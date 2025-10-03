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

/// Provides a default weight of 1.0 for an assertion.
fn default_weight() -> f64 {
    1.0
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
        /// The weight of this assertion for scoring. Defaults to 1.0.
        #[serde(default = "default_weight")]
        weight: f64,
    },
    /// Asserts a change in the SOL balance of an account.
    SolBalanceChange {
        /// The pubkey of the account to check.
        pubkey: String,
        /// The expected minimum change in lamports (can be negative).
        expected_change_gte: i64,
        /// The weight of this assertion for scoring. Defaults to 1.0.
        #[serde(default = "default_weight")]
        weight: f64,
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
        /// Optional field to derive the token account address dynamically.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        address_derivation: Option<AddressDerivation>,
        /// The weight of this assertion for scoring. Defaults to 1.0.
        #[serde(default = "default_weight")]
        weight: f64,
    },
}

impl StateAssertion {
    pub fn pubkey(&self) -> &str {
        match self {
            StateAssertion::SolBalance { pubkey, .. } => pubkey,
            StateAssertion::SolBalanceChange { pubkey, .. } => pubkey,
            StateAssertion::TokenAccountBalance { pubkey, .. } => pubkey,
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            StateAssertion::SolBalance { weight, .. } => *weight,
            StateAssertion::SolBalanceChange { weight, .. } => *weight,
            StateAssertion::TokenAccountBalance { weight, .. } => *weight,
        }
    }

    pub fn address_derivation(&self) -> Option<&AddressDerivation> {
        match self {
            StateAssertion::TokenAccountBalance {
                address_derivation, ..
            } => address_derivation.as_ref(),
            _ => None,
        }
    }
}

/// Defines how to derive an on-chain address from other known accounts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "PascalCase")]
pub enum AddressDerivation {
    /// Derives an Associated Token Account (ATA) address.
    AssociatedTokenAccount {
        /// The placeholder for the owner's wallet address.
        owner: String,
        /// The mint address of the token.
        mint: String,
    },
}

/// A serializable representation of a Solana instruction for use in benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BenchmarkInstruction {
    /// The pubkey of the program to be executed.
    pub program_id: String,
    /// The weight for a correct program_id.
    #[serde(default = "default_weight")]
    pub program_id_weight: f64,
    /// The accounts required by the instruction.
    pub accounts: Vec<BenchmarkAccountMeta>,
    /// The instruction data, typically as a Base58 string.
    pub data: String,
    /// The weight for correct instruction data.
    #[serde(default = "default_weight")]
    pub data_weight: f64,
}

/// A serializable representation of an `AccountMeta` for use in benchmarks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BenchmarkAccountMeta {
    /// The pubkey of the account.
    pub pubkey: String,
    /// `true` if the transaction must be signed by this account's private key.
    pub is_signer: bool,
    /// `true` if the account's data may be mutated by the program.
    pub is_writable: bool,
    /// The weight for a correct account in this position.
    #[serde(default = "default_weight")]
    pub weight: f64,
}
