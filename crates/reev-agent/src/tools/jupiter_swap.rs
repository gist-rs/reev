//! Jupiter swap tool wrapper
//!
//! This tool provides AI agent access to Jupiter's swap functionality,
//! allowing token exchanges through Jupiter's aggregator.

use bs58;
use jup_sdk::{models::SwapParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
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
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid slippage: {0}")]
    InvalidSlippage(String),
    #[error("Same input and output mint")]
    SameMint,
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for performing swap operations using the Jupiter API.
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

    /// Executes the tool's logic: calls the Jupiter swap API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterSwapError::InvalidPubkey(e.to_string()))?;
        let input_mint = Pubkey::from_str(&args.input_mint)
            .map_err(|e| JupiterSwapError::InvalidPubkey(e.to_string()))?;
        let output_mint = Pubkey::from_str(&args.output_mint)
            .map_err(|e| JupiterSwapError::InvalidPubkey(e.to_string()))?;

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

        // Call the Jupiter swap API
        let raw_instructions = self
            .handle_jupiter_swap(
                user_pubkey,
                input_mint,
                output_mint,
                args.amount,
                args.slippage_bps,
            )
            .await
            .map_err(JupiterSwapError::ProtocolError)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}

impl JupiterSwapTool {
    /// Internal handler for Jupiter swap operations
    async fn handle_jupiter_swap(
        &self,
        user_pubkey: Pubkey,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> anyhow::Result<Vec<RawInstruction>> {
        // The jup-sdk's client is designed to work with a local validator.
        let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

        let swap_params = SwapParams {
            input_mint,
            output_mint,
            amount,
            slippage_bps,
        };

        // The sdk's swap builder will handle quoting and instruction generation
        // against the local surfpool instance.
        let (instructions, _alt_accounts) = jupiter_client
            .swap(swap_params)
            .prepare_transaction_components()
            .await?;

        // The sdk returns instructions in its own format, so we need to convert them.
        let raw_instructions = instructions
            .into_iter()
            .map(|inst| {
                let accounts = inst
                    .accounts
                    .into_iter()
                    .map(|acc| RawAccountMeta {
                        pubkey: acc.pubkey.to_string(),
                        is_signer: acc.is_signer,
                        is_writable: acc.is_writable,
                    })
                    .collect();

                RawInstruction {
                    program_id: inst.program_id.to_string(),
                    accounts,
                    data: bs58::encode(inst.data).into_string(),
                }
            })
            .collect();

        Ok(raw_instructions)
    }
}
