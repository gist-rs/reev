use anyhow::Result;

use reev_lib::{
    actions::spl_transfer,
    agent::{RawAccountMeta, RawInstruction},
};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use thiserror::Error;

/// The arguments for the SPL Token transfer tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct SplTransferArgs {
    pub source_pubkey: String,
    pub destination_pubkey: String,
    pub authority_pubkey: String,
    pub amount: u64,
}

/// A custom error type for the SPL Token transfer tool.
#[derive(Debug, Error)]
pub enum SplTransferError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Failed to create instruction: {0}")]
    InstructionCreation(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing SPL Token transfers.
#[derive(Deserialize, Serialize, Default)]
pub struct SplTransferTool;

impl Tool for SplTransferTool {
    const NAME: &'static str = "spl_transfer";
    type Error = SplTransferError;
    type Args = SplTransferArgs;
    type Output = String; // The tool will return the raw instruction as a JSON string.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer a specified amount of a token from one token account to another. Use this for all non-native tokens like USDC, USDT, etc.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "source_pubkey": {
                        "type": "string",
                        "description": "The public key of the token account to send tokens FROM."
                    },
                    "destination_pubkey": {
                        "type": "string",
                        "description": "The public key of the token account to send tokens TO."
                    },
                    "authority_pubkey": {
                        "type": "string",
                        "description": "The public key of the owner of the source token account, who is authorized to sign the transaction."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the token to transfer, in its smallest denomination (e.g., if a token has 6 decimals, to send 15 tokens, this value should be 15000000)."
                    }
                },
                "required": ["source_pubkey", "destination_pubkey", "authority_pubkey", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: creates a Solana instruction and serializes it.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 1. Parse the pubkeys provided by the AI model.
        let source_pubkey = Pubkey::from_str(&args.source_pubkey)
            .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;
        let destination_pubkey = Pubkey::from_str(&args.destination_pubkey)
            .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;
        let authority_pubkey = Pubkey::from_str(&args.authority_pubkey)
            .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;

        // 2. Call the centralized, secure function from `reev-lib` to create the instruction.
        let instruction = spl_transfer::create_instruction(
            &source_pubkey,
            &destination_pubkey,
            &authority_pubkey,
            args.amount,
        )?;

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
        let output = serde_json::to_string(&raw_instruction)?;

        Ok(output)
    }
}
