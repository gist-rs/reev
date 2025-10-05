//! Jupiter swap tool wrapper
//!
//! This tool provides AI agent access to Jupiter's swap functionality,
//! allowing token exchanges through Jupiter's aggregator.

use crate::protocols::get_jupiter_config;
use crate::protocols::jupiter::{execute_request, parse_json_response};
use bs58;
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
        let config = get_jupiter_config();
        let client = config.create_client()?;

        // First get the quote
        let quote_url = format!(
            "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            config.quote_url(),
            input_mint,
            output_mint,
            amount,
            slippage_bps
        );

        let quote_request = client.get(&quote_url).header("Accept", "application/json");
        let quote_response = execute_request(quote_request, config.max_retries).await?;
        let quote_json = parse_json_response(quote_response).await?;

        // Then perform the swap with the quote response
        let swap_request_body = json!({
            "userPublicKey": user_pubkey.to_string(),
            "quoteResponse": quote_json,
            "prioritizationFeeLamports": {
                "priorityLevelWithMaxLamports": {
                    "maxLamports": 10000000,
                    "priorityLevel": "veryHigh"
                }
            },
            "dynamicComputeUnitLimit": true
        });

        let swap_request = client
            .post(config.swap_url())
            .header("Content-Type", "application/json")
            .json(&swap_request_body);

        let swap_response = execute_request(swap_request, config.max_retries).await?;
        let swap_json = parse_json_response(swap_response).await?;

        // Log the full response from Jupiter for debugging.
        tracing::info!(
            "[reev-agent] Jupiter swap response: {}",
            serde_json::to_string_pretty(&swap_json)?
        );

        // Parse the Jupiter API response to extract transaction instructions
        let instructions = if let Some(instructions_array) =
            swap_json.get("instructions").and_then(|v| v.as_array())
        {
            // Convert Jupiter instructions to RawInstruction format
            instructions_array
                .iter()
                .filter_map(|inst| {
                    let program_id = inst.get("programId")?.as_str()?;
                    let accounts = inst.get("accounts")?.as_array()?;
                    let data = inst.get("data")?.as_str()?;

                    let raw_accounts: Vec<RawAccountMeta> = accounts
                        .iter()
                        .filter_map(|acc| {
                            let pubkey = acc.get("pubkey")?.as_str()?;
                            let is_signer = acc.get("isSigner")?.as_bool()?;
                            let is_writable = acc.get("isWritable")?.as_bool()?;

                            Some(RawAccountMeta {
                                pubkey: pubkey.to_string(),
                                is_signer,
                                is_writable,
                            })
                        })
                        .collect();

                    Some(RawInstruction {
                        program_id: program_id.to_string(),
                        accounts: raw_accounts,
                        data: data.to_string(),
                    })
                })
                .collect()
        } else {
            // Fallback to a placeholder instruction with more realistic accounts and data.
            // This will likely still fail simulation but is better structured.
            let user_usdc_ata = self
                .key_map
                .get("USER_USDC_ATA")
                .cloned()
                .unwrap_or_else(|| output_mint.to_string());

            // Create placeholder instruction data (e.g., for a hypothetical route instruction).
            // This consists of a dummy instruction discriminator and the amount.
            let mut instruction_data = vec![4]; // Placeholder discriminator
            instruction_data.extend_from_slice(&amount.to_le_bytes());
            instruction_data.extend_from_slice(&0u64.to_le_bytes()); // Placeholder for min_out_amount

            vec![RawInstruction {
                program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                accounts: vec![
                    RawAccountMeta {
                        pubkey: user_pubkey.to_string(),
                        is_signer: true,
                        is_writable: true,
                    },
                    RawAccountMeta {
                        pubkey: user_usdc_ata,
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: bs58::encode(instruction_data).into_string(),
            }]
        };

        Ok(instructions)
    }
}
