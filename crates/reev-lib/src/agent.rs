use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::collections::HashMap;
use std::str::FromStr;

// --- Structs for Deserializing the Third-Party API Response ---
// These structs are designed to perfectly match the nested structure of the API's JSON response.

/// Matches the innermost JSON object: the instruction itself.
/// The `text` field of the API response is expected to contain this structure.
#[derive(Debug, Deserialize)]
pub struct LlmInstruction {
    pub program_id: String,
    pub accounts: Vec<LlmAccountMeta>,
    pub data: String, // Expected to be a Base58 encoded string
}

/// Matches the structure of an account within the `accounts` array of the API response.
#[derive(Debug, Deserialize)]
pub struct LlmAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// Matches the `{"text": ...}` object in the API response.
/// This is the key change to align with the provided server code.
#[derive(Debug, Deserialize)]
pub struct LlmResult {
    pub text: LlmInstruction,
}

/// Matches the top-level `{"result": ...}` object in the API response.
#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub result: LlmResult,
}

// --- Core Agent Definitions ---

/// The action an agent decides to take.
///
/// This is a wrapper around the native `solana_sdk::instruction::Instruction`.
/// This design makes it easy for the `SolanaEnv` to directly use the action
/// without further parsing, and it allows the execution trace to be serializable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentAction(pub Instruction);

/// Provides a direct conversion from the API's instruction format to the native `AgentAction`.
/// This is the bridge between the external LLM world and the internal `reev` framework.
impl TryFrom<LlmInstruction> for AgentAction {
    type Error = anyhow::Error;

    fn try_from(llm_instruction: LlmInstruction) -> Result<Self, Self::Error> {
        // Parse the program_id string into a native Pubkey.
        let program_id = Pubkey::from_str(&llm_instruction.program_id)
            .context("Failed to parse 'program_id' string into a Pubkey")?;

        // Map the API's account format to the native `solana_sdk::instruction::AccountMeta`.
        let accounts = llm_instruction
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

        // Decode the base58 `data` string into a raw byte vector.
        let data = bs58::decode(&llm_instruction.data)
            .into_vec()
            .context("Failed to decode base58 'data' string")?;

        // Construct the native `Instruction` and wrap it in our `AgentAction`.
        Ok(AgentAction(Instruction {
            program_id,
            accounts,
            data,
        }))
    }
}

/// Represents the observation of the environment state provided back to the agent.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentObservation {
    pub last_transaction_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transaction_error: Option<String>,
    pub last_transaction_logs: Vec<String>,
    pub account_states: HashMap<String, serde_json::Value>,
}

/// The trait that all agents must implement.
#[async_trait]
pub trait Agent {
    /// Takes an observation from the environment and returns the next action (a raw instruction) to take.
    async fn get_action(&mut self, observation: &AgentObservation) -> Result<AgentAction>;
}
