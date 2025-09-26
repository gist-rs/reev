use anyhow::{Context, Result};

use reev_lib::{
    actions::sol_transfer,
    agent::{RawAccountMeta, RawInstruction},
};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// The arguments for the SOL transfer tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct SolTransferArgs {
    pub from_pubkey: String,
    pub to_pubkey: String,
    pub lamports: u64,
}

/// A custom error type for the SOL transfer tool.
#[derive(Debug, thiserror::Error)]
pub enum SolTransferError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("An unexpected error occurred: {0}")]
    Unexpected(#[from] anyhow::Error),
}

/// A `rig` tool for performing native SOL transfers.
#[derive(Deserialize, Serialize, Default)]
pub struct SolTransferTool;

impl Tool for SolTransferTool {
    const NAME: &'static str = "sol_transfer";
    type Error = SolTransferError;
    type Args = SolTransferArgs;
    type Output = String; // The tool will return the raw instruction as a JSON string.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer native SOL from one account to another. This is for the native cryptocurrency of the Solana blockchain, not for tokens.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "from_pubkey": {
                        "type": "string",
                        "description": "The public key of the account that will send the SOL. This account will also pay the transaction fee."
                    },
                    "to_pubkey": {
                        "type": "string",
                        "description": "The public key of the account that will receive the SOL."
                    },
                    "lamports": {
                        "type": "number",
                        "description": "The amount of lamports to transfer. 1 SOL = 1,000,000,000 lamports."
                    }
                },
                "required": ["from_pubkey", "to_pubkey", "lamports"],
            }),
        }
    }

    /// Executes the tool's logic: creates a Solana instruction and serializes it.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 1. Parse the pubkeys provided by the AI model.
        let from_pubkey = Pubkey::from_str(&args.from_pubkey)
            .map_err(|e| SolTransferError::PubkeyParse(e.to_string()))?;
        let to_pubkey = Pubkey::from_str(&args.to_pubkey)
            .map_err(|e| SolTransferError::PubkeyParse(e.to_string()))?;

        // 2. Call the centralized, secure function from `reev-lib` to create the instruction.
        let instruction = sol_transfer::create_instruction(&from_pubkey, &to_pubkey, args.lamports);

        // 3. Convert the native `solana_sdk::Instruction` into our serializable `RawInstruction` format.
        let raw_instruction = RawInstruction {
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
        };

        // 4. Serialize the `RawInstruction` to a JSON string. This is the final output of the tool.
        let output = serde_json::to_string(&raw_instruction)
            .context("Failed to serialize raw instruction")?;

        Ok(output)
    }
}
