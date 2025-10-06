//! Jupiter swap tool wrapper
//!
//! This tool provides AI agent access to Jupiter's swap functionality,
//! acting as a thin wrapper around the protocol handler.

use crate::protocols::jupiter::swap::handle_jupiter_swap;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// The arguments for the Jupiter swap tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterSwapArgs {
    pub user_pubkey: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
}

/// A custom error type for the Jupiter swap tool.
#[derive(Debug, Error)]
pub enum JupiterSwapError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter protocol call failed: {0}")]
    ProtocolCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid slippage: {0}")]
    InvalidSlippage(String),
    #[error("Same input and output mint")]
    SameMint,
}

/// A `rig` tool for performing swap operations using the Jupiter API.
/// This tool acts as a thin wrapper around the protocol handler.
#[derive(Deserialize, Serialize)]
pub struct JupiterSwapTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterSwapTool {
    const NAME: &'static str = "jupiter_swap";
    type Error = JupiterSwapError;
    type Args = JupiterSwapArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Swap tokens using Jupiter's aggregator for best rates and routes. This finds the optimal path for token exchanges across multiple DEXs.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the swap."
                    },
                    "input_mint": {
                        "type": "string",
                        "description": "The mint address of the input token to swap."
                    },
                    "output_mint": {
                        "type": "string",
                        "description": "The mint address of the output token to receive."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the input token to swap, in its smallest denomination."
                    },
                    "slippage_bps": {
                        "type": "number",
                        "description": "The slippage tolerance in basis points (0.01% = 1 bps)."
                    }
                },
                "required": ["user_pubkey", "input_mint", "output_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the protocol handler.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate and parse arguments
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let input_mint = Pubkey::from_str(&args.input_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let output_mint = Pubkey::from_str(&args.output_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;

        // Validate business logic
        if args.amount == 0 {
            return Err(JupiterSwapError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        if args.slippage_bps > 10000 {
            return Err(JupiterSwapError::InvalidSlippage(
                "Slippage cannot exceed 100% (10000 bps)".to_string(),
            ));
        }

        if input_mint == output_mint {
            return Err(JupiterSwapError::SameMint);
        }

        // Call the protocol handler
        let raw_instructions = handle_jupiter_swap(
            user_pubkey,
            input_mint,
            output_mint,
            args.amount,
            args.slippage_bps,
            &self.key_map,
        )
        .await
        .map_err(JupiterSwapError::ProtocolCall)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
