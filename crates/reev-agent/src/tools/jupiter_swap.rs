use crate::jupiter::swap::handle_jupiter_swap;
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};
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
    #[error("Jupiter API call failed: {0}")]
    ApiCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing swaps using the Jupiter API.
/// This tool requires the on-chain context (`key_map`) to be provided during its construction.
#[derive(Deserialize, Serialize)]
pub struct JupiterSwapTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterSwapTool {
    const NAME: &'static str = "jupiter_swap";
    type Error = JupiterSwapError;
    type Args = JupiterSwapArgs;
    type Output = String; // The tool will return the raw instruction as a JSON string.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Swap one token for another using the Jupiter aggregator. This finds the best price across many decentralized exchanges.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the swap. This wallet must sign the transaction."
                    },
                    "input_mint": {
                        "type": "string",
                        "description": "The mint address of the token to be swapped FROM. For native SOL, use 'So11111111111111111111111111111111111111112'."
                    },
                    "output_mint": {
                        "type": "string",
                        "description": "The mint address of the token to be swapped TO. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the input token to swap, in its smallest denomination (e.g., lamports for SOL)."
                    },
                    "slippage_bps": {
                        "type": "number",
                        "description": "The slippage tolerance in basis points (e.g., 50 for 0.5%)."
                    }
                },
                "required": ["user_pubkey", "input_mint", "output_mint", "amount", "slippage_bps"],
            }),
        }
    }

    /// Executes the tool's logic: calls the centralized `handle_jupiter_swap` function,
    /// which transparently handles mock vs. mainnet mints.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let input_mint = Pubkey::from_str(&args.input_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;
        let output_mint = Pubkey::from_str(&args.output_mint)
            .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?;

        // Call the centralized handler, passing the key_map from the struct.
        // This ensures mock mints are correctly replaced before the API call.
        let raw_instruction = handle_jupiter_swap(
            user_pubkey,
            input_mint,
            output_mint,
            args.amount,
            args.slippage_bps,
            &self.key_map,
        )
        .await?;

        // Serialize the RawInstruction to a JSON string. This is the final output of the tool.
        let output = serde_json::to_string(&raw_instruction)?;

        Ok(output)
    }
}
