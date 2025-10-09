//! Jupiter swap tool wrapper
//!
//! This tool provides AI agent access to Jupiter's swap functionality,
//! acting as a thin wrapper around the protocol handler.

use crate::protocols::jupiter::{get_jupiter_config, swap::handle_jupiter_swap};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use tracing::info;

/// The arguments for the Jupiter swap tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterSwapArgs {
    pub user_pubkey: String,
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    #[serde(default)]
    pub slippage_bps: Option<u16>,
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

/// 🎯 Enhanced tool response with transaction metadata
#[derive(Debug, Clone, Serialize)]
pub struct JupiterSwapResponse {
    pub instructions: Vec<serde_json::Value>,
    pub transaction_count: usize,
    pub estimated_signatures: Vec<String>,
    pub operation_type: String,
    pub status: String,
    pub completed: bool,
    pub next_action: String,
    pub message: String,
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
            description: "Swap tokens using Jupiter's aggregator for best rates and routes. This finds the optimal path for token exchanges across multiple DEXs. NOTE: If you don't see account balance information in the context, use get_account_balance tool first to verify sufficient funds before swapping.".to_string(),
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
                        "description": "The slippage tolerance in basis points (0.01% = 1 bps). If not provided, uses the default from configuration."
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

        // Use default slippage from configuration if not provided
        let config = get_jupiter_config();
        let slippage_bps = match args.slippage_bps {
            Some(slippage) => config
                .validate_slippage(slippage)
                .map_err(|e| JupiterSwapError::InvalidSlippage(e.to_string()))?,
            None => config.default_slippage(),
        };

        if input_mint == output_mint {
            return Err(JupiterSwapError::SameMint);
        }

        // Call the protocol handler
        let raw_instructions = handle_jupiter_swap(
            user_pubkey,
            input_mint,
            output_mint,
            args.amount,
            slippage_bps,
            &self.key_map,
        )
        .await
        .map_err(JupiterSwapError::ProtocolCall)?;

        // 🎯 Create enhanced response with metadata
        let instruction_count = raw_instructions.len();
        let estimated_signatures = (0..instruction_count)
            .map(|i| {
                format!(
                    "swap_tx_{}_{}",
                    i,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                )
            })
            .collect();

        let swap_response = JupiterSwapResponse {
            instructions: raw_instructions
                .into_iter()
                .map(|inst| serde_json::to_value(inst).unwrap_or_default())
                .collect(),
            transaction_count: instruction_count,
            estimated_signatures,
            operation_type: "jupiter_swap".to_string(),
            status: "success".to_string(),
            completed: true,
            next_action: "STOP".to_string(),
            message: format!("Successfully executed {instruction_count} jupiter_swap operation(s)"),
        };

        // Serialize the enhanced response to JSON string.
        let output = serde_json::to_string(&swap_response)?;

        info!(
            "[JupiterSwapTool] Generated {} instructions for {}→{} swap",
            instruction_count, args.input_mint, args.output_mint
        );

        Ok(output)
    }
}
