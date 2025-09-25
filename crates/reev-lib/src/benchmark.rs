use crate::agent::AgentAction;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::str::FromStr;

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

/// Optional data for initializing a new SPL token mint account.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MintData {
    /// The authority that can mint new tokens. Defaults to `USER_WALLET_PUBKEY`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint_authority: Option<String>,
    /// The number of decimal places for the token.
    pub decimals: u8,
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
    /// If present, initializes this account as an SPL token mint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint_data: Option<MintData>,
}

/// A deserializable representation of a Solana instruction account,
/// tailored for use in benchmark files.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BenchmarkAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// A deserializable representation of a Solana instruction,
/// tailored for use in benchmark files.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BenchmarkInstruction {
    pub program_id: String,
    pub accounts: Vec<BenchmarkAccountMeta>,
    pub data: String, // Expected to be a Base58 encoded string
}

impl TryFrom<BenchmarkInstruction> for AgentAction {
    type Error = anyhow::Error;

    fn try_from(bench_instruction: BenchmarkInstruction) -> Result<Self, Self::Error> {
        let program_id = Pubkey::from_str(&bench_instruction.program_id)
            .context("Failed to parse 'program_id' string into a Pubkey")?;

        let accounts = bench_instruction
            .accounts
            .into_iter()
            .map(|acc| {
                let pubkey = Pubkey::from_str(&acc.pubkey)
                    .context(format!("Failed to parse account pubkey: '{}'", acc.pubkey))?;
                Ok(AccountMeta {
                    pubkey,
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
            })
            .collect::<Result<Vec<AccountMeta>>>()?;

        let data = bs58::decode(&bench_instruction.data)
            .into_vec()
            .context("Failed to decode base58 'data' string")?;

        Ok(AgentAction(Instruction {
            program_id,
            accounts,
            data,
        }))
    }
}

/// Contains the objective criteria for judging the agent's performance on a test case.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GroundTruth {
    /// A list of conditions that must be true on the blockchain after the agent has finished.
    pub final_state_assertions: Vec<StateAssertion>,
    /// The ideal instruction the agent is expected to generate to solve the task.
    /// This is used for calculating instruction accuracy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_instruction: Option<BenchmarkInstruction>,
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
