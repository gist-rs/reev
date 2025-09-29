use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::collections::HashMap;
use std::str::FromStr;

/// Represents a raw, JSON-formatted instruction, suitable for serialization
/// and for being the target format for an LLM agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawInstruction {
    pub program_id: String,
    pub accounts: Vec<RawAccountMeta>,
    pub data: String, // Base58 encoded
}

/// A simplified, string-based version of `AccountMeta` for easy JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<Instruction> for RawInstruction {
    fn from(instruction: Instruction) -> Self {
        Self {
            program_id: instruction.program_id.to_string(),
            accounts: instruction
                .accounts
                .iter()
                .map(|acc| RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: bs58::encode(instruction.data).into_string(),
        }
    }
}

/// A wrapper around a native Solana `Instruction` to represent a single, executable action by an agent.
#[derive(Debug, Clone)]
pub struct AgentAction(pub Instruction);

/// Manual Serialize implementation for AgentAction to produce a human-readable JSON format.
impl Serialize for AgentAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw_instruction = RawInstruction {
            program_id: self.0.program_id.to_string(),
            accounts: self
                .0
                .accounts
                .iter()
                .map(|acc| RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: bs58::encode(&self.0.data).into_string(),
        };
        raw_instruction.serialize(serializer)
    }
}

/// Manual Deserialize implementation for AgentAction from a human-readable JSON format.
impl<'de> Deserialize<'de> for AgentAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawInstruction::deserialize(deserializer)?;
        raw.try_into().map_err(serde::de::Error::custom)
    }
}

/// Represents the state of the environment that the agent can perceive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservation {
    /// The status of the last transaction (e.g., "Success", "Failure").
    pub last_transaction_status: String,
    /// An optional error message from the last transaction.
    pub last_transaction_error: Option<String>,
    /// Logs from the last transaction.
    pub last_transaction_logs: Vec<String>,
    /// A map of account placeholder names to their on-chain state.
    pub account_states: HashMap<String, Value>,
    /// A map of account placeholder names to their actual public keys.
    pub key_map: HashMap<String, String>,
}

#[async_trait]
pub trait Agent {
    /// Given a prompt and an observation of the environment, returns the next action to take.
    async fn get_action(
        &mut self,
        id: &str,
        prompt: &str,
        observation: &AgentObservation,
        fee_payer: Option<&String>,
    ) -> Result<Vec<AgentAction>>;
}

/// Structs for deserializing the third-party LLM's JSON response.
#[derive(Debug, Deserialize)]
pub struct LlmResponse {
    pub result: LlmResult,
}

#[derive(Debug, Deserialize)]
pub struct LlmResult {
    #[serde(deserialize_with = "deserialize_string_to_instructions")]
    pub text: Vec<RawInstruction>,
}

fn deserialize_string_to_instructions<'de, D>(
    deserializer: D,
) -> Result<Vec<RawInstruction>, D::Error>
where
    D: Deserializer<'de>,
{
    // The `text` field is a string containing JSON. First, deserialize it into a string.
    let s: String = Deserialize::deserialize(deserializer)?;

    // Now, parse that string into a `serde_json::Value`.
    let v: Value = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

    // Check if it's an array of instructions or a single instruction object.
    if v.is_array() {
        serde_json::from_value(v).map_err(serde::de::Error::custom)
    } else {
        let ix: RawInstruction = serde_json::from_value(v).map_err(serde::de::Error::custom)?;
        Ok(vec![ix])
    }
}

/// Conversion from the raw format to our action wrapper.
impl TryFrom<RawInstruction> for AgentAction {
    type Error = anyhow::Error;

    fn try_from(raw: RawInstruction) -> Result<Self, Self::Error> {
        let program_id = Pubkey::from_str(&raw.program_id)
            .with_context(|| format!("Invalid program_id: {}", raw.program_id))?;

        let accounts = raw
            .accounts
            .into_iter()
            .map(|acc| {
                let pubkey = Pubkey::from_str(&acc.pubkey)
                    .with_context(|| format!("Invalid pubkey in accounts: {}", acc.pubkey))?;
                Ok(AccountMeta {
                    pubkey,
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let data = bs58::decode(&raw.data)
            .into_vec()
            .with_context(|| format!("Invalid base58 data: {}", raw.data))?;

        Ok(AgentAction(Instruction {
            program_id,
            accounts,
            data,
        }))
    }
}
