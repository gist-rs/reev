//! Jupiter swap tool wrapper
//!
//! This tool provides AI agent access to Jupiter's swap functionality,
//! acting as a thin wrapper around the protocol handler.

use reev_protocols::jupiter::{get_jupiter_config, swap::handle_jupiter_swap};

use reev_lib::balance_validation::{BalanceValidationError, BalanceValidator};
use reev_lib::constants::{sol_mint, usdc_mint};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, instrument, warn};

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
    #[error("Balance validation failed: {0}")]
    BalanceValidation(#[from] Box<BalanceValidationError>),
}

impl From<BalanceValidationError> for JupiterSwapError {
    fn from(err: BalanceValidationError) -> Self {
        Self::BalanceValidation(Box::new(err))
    }
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
        let input_mint_description = format!(
            "The mint address of the input token to swap (e.g., '{}' for SOL, '{}' for USDC)",
            sol_mint(),
            usdc_mint()
        );
        let output_mint_description = format!(
            "The mint address of the output token to receive (e.g., '{}' for SOL, '{}' for USDC)",
            sol_mint(),
            usdc_mint()
        );

        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "PRIMARY tool for swapping tokens using Jupiter. Supports SOL, USDC, and other tokens. Use when user says 'swap', 'exchange', or mentions token conversion. IMPORTANT: This tool will automatically validate the input token balance. If user mentions 'lend', 'deposit', 'mint', or 'redeem', use Jupiter lending tools instead.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the swap."
                    },
                    "input_mint": {
                        "type": "string",
                        "description": input_mint_description
                    },
                    "output_mint": {
                        "type": "string",
                        "description": output_mint_description
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the input token to swap, in its smallest denomination (e.g., lamports for SOL). This will be validated against available balance."
                    },
                    "slippage_bps": {
                        "type": "integer",
                        "description": "Optional slippage tolerance in basis points (1-10000). Default is 100 (1%)."
                    }
                },
                "required": ["user_pubkey", "input_mint", "output_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the protocol handler.
    #[instrument(
        name = "jupiter_swap_tool_call",
        skip(self),
        fields(
            tool_name = "jupiter_swap",
            user_pubkey = %args.user_pubkey,
            input_mint = %args.input_mint,
            output_mint = %args.output_mint,
            amount = args.amount,
            slippage_bps = ?args.slippage_bps
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[JupiterSwapTool] Starting tool execution with OpenTelemetry tracing");
        let start_time = Instant::now();

        // Prepare tool args for logging
        let _tool_args = json!({
            "input_mint": args.input_mint,
            "output_mint": args.output_mint,
            "amount": args.amount,
            "slippage_bps": args.slippage_bps
        })
        .to_string();

        // Execute the swap operation and log both success and failure
        let result = self.execute_swap_internal(&args, start_time).await;

        match &result {
            Ok(output) => {
                // Try to extract instruction count for logging
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(output) {
                    if let Some(instruction_count) =
                        response.get("transaction_count").and_then(|v| v.as_u64())
                    {
                        // Jupiter swap successful with instruction_count instructions
                        info!(
                            "[JupiterSwapTool] Successfully created swap with {} instructions",
                            instruction_count
                        );
                    }
                }
            }
            Err(e) => {
                warn!("[JupiterSwapTool] Swap failed: {}", e);
            }
        }

        result
    }
}

impl JupiterSwapTool {
    /// Internal method to execute the swap logic
    async fn execute_swap_internal(
        &self,
        args: &JupiterSwapArgs,
        start_time: std::time::Instant,
    ) -> Result<String, JupiterSwapError> {
        // Check for placeholder addresses and resolve them from key_map if possible
        let user_pubkey = if args.user_pubkey.starts_with("USER_")
            || args.user_pubkey.starts_with("RECIPIENT_")
        {
            if let Some(resolved_pubkey) = self.key_map.get(&args.user_pubkey) {
                info!(
                    "Resolved {} from key_map: {}",
                    args.user_pubkey, resolved_pubkey
                );
                Pubkey::from_str(resolved_pubkey)
                    .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
            } else {
                info!(
                    "Could not resolve {} from key_map, using simulated pubkey for swap",
                    args.user_pubkey
                );
                Pubkey::from_str("11111111111111111111111111111111")
                    .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
            }
        } else {
            Pubkey::from_str(&args.user_pubkey)
                .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
        };

        let input_mint =
            if args.input_mint.starts_with("USER_") || args.input_mint.starts_with("RECIPIENT_") {
                if let Some(resolved_mint) = self.key_map.get(&args.input_mint) {
                    info!(
                        "Resolved {} from key_map: {}",
                        args.input_mint, resolved_mint
                    );
                    Pubkey::from_str(resolved_mint)
                        .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
                } else {
                    info!(
                        "Could not resolve {} from key_map, using simulated mint for swap",
                        args.input_mint
                    );
                    Pubkey::from_str("So11111111111111111111111111111111111112")
                        .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
                }
            } else {
                Pubkey::from_str(&args.input_mint)
                    .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
            };

        let output_mint = if args.output_mint.starts_with("USER_")
            || args.output_mint.starts_with("RECIPIENT_")
        {
            if let Some(resolved_mint) = self.key_map.get(&args.output_mint) {
                info!(
                    "Resolved {} from key_map: {}",
                    args.output_mint, resolved_mint
                );
                Pubkey::from_str(resolved_mint)
                    .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
            } else {
                info!(
                    "Could not resolve {} from key_map, using simulated mint for swap",
                    args.output_mint
                );
                usdc_mint()
            }
        } else {
            Pubkey::from_str(&args.output_mint)
                .map_err(|e| JupiterSwapError::PubkeyParse(e.to_string()))?
        };

        // Validate business logic
        if args.amount == 0 {
            return Err(JupiterSwapError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Use shared balance validation utility for input token
        let balance_validator = BalanceValidator::new(self.key_map.clone());

        match balance_validator
            .validate_token_balance(&input_mint.to_string(), &args.user_pubkey, args.amount)
            .map_err(JupiterSwapError::from)
        {
            Ok(()) => {
                info!(
                    "✅ Balance validation passed: requested {} for input mint {}",
                    args.amount, input_mint
                );

                // Log the available balance for debugging
                if let Ok(available) =
                    balance_validator.get_token_balance(&input_mint.to_string(), &args.user_pubkey)
                {
                    info!("Available balance: {}", available);
                }
            }
            Err(e) => {
                warn!(
                    "❌ Balance validation failed for input mint {}: {}",
                    input_mint, e
                );

                // Provide helpful guidance for insufficient funds errors
                if let JupiterSwapError::BalanceValidation(boxed_err) = &e {
                    if let BalanceValidationError::InsufficientFunds {
                        requested,
                        available,
                    } = boxed_err.as_ref()
                    {
                        warn!(
                            "💡 Suggestion: Check available balance before swapping. \
                            Available: {}, Requested: {}",
                            available, requested
                        );
                    }
                }

                return Err(e);
            }
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

        // Call the protocol handler with flow tracking
        let protocol_start_time = Instant::now();
        info!("[JupiterSwapTool] Calling Jupiter protocol handler");
        let raw_instructions = handle_jupiter_swap(
            user_pubkey,
            input_mint,
            output_mint,
            args.amount,
            slippage_bps,
        )
        .await
        .map_err(JupiterSwapError::ProtocolCall)?;
        let protocol_execution_time = protocol_start_time.elapsed().as_millis() as u32;
        let total_execution_time = start_time.elapsed().as_millis() as u32;

        info!(
            "[JupiterSwapTool] Protocol execution completed - protocol_time: {}ms, total_time: {}ms, instructions: {}",
            protocol_execution_time, total_execution_time, raw_instructions.len()
        );

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
