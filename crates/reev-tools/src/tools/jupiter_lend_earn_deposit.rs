//! Jupiter lend earn deposit tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn/deposit functionality.
//! It acts as a thin wrapper around the protocol handler.

use reev_lib::balance_validation::{BalanceValidationError, BalanceValidator};
use reev_lib::constants::usdc_mint;
use reev_protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr, time::Instant};
use thiserror::Error;
use tracing::{info, instrument, warn};

/// The arguments for the Jupiter lend earn deposit tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendEarnDepositArgs {
    pub user_pubkey: String,
    pub asset_mint: String,
    pub amount: u64,
}

/// A custom error type for the Jupiter lend earn deposit tool.
#[derive(Debug, Error)]
pub enum JupiterLendEarnDepositError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter protocol call failed: {0}")]
    ProtocolCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Balance validation failed: {0}")]
    BalanceValidation(#[from] Box<BalanceValidationError>),
}

impl From<BalanceValidationError> for JupiterLendEarnDepositError {
    fn from(err: BalanceValidationError) -> Self {
        Self::BalanceValidation(Box::new(err))
    }
}

/// A `rig` tool for performing lend earn deposit operations using the Jupiter API.
/// This tool acts as a thin wrapper around the protocol handler.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendEarnDepositTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendEarnDepositTool {
    const NAME: &'static str = "jupiter_lend_earn_deposit";
    type Error = JupiterLendEarnDepositError;
    type Args = JupiterLendEarnDepositArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let asset_mint_description = format!(
            "The mint address of the token to be lent. For native SOL, use '{}'. For USDC, use '{}'.",
            native_mint::ID,
            usdc_mint()
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "PRIMARY tool for DEPOSITING tokens into Jupiter lending. Use ONLY when user says 'deposit', 'lend', or mentions depositing token amounts. IMPORTANT: This tool will automatically validate the balance against available funds. If you need to check the available balance first, use the get_account_balance tool. DO NOT use for 'mint' or 'redeem' operations. If user mentions 'mint', use jupiter_lend_earn_mint instead.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the deposit."
                    },
                    "asset_mint": {
                        "type": "string",
                        "description": asset_mint_description
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the token to deposit, in its smallest denomination (e.g., lamports for SOL). This will be validated against available balance."
                    }
                },
                "required": ["user_pubkey", "asset_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the protocol handler.
    #[instrument(
        name = "jupiter_lend_earn_deposit_tool_call",
        skip(self),
        fields(
            tool_name = "jupiter_lend_earn_deposit",
            user_pubkey = %args.user_pubkey,
            asset_mint = %args.asset_mint,
            amount = args.amount
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[JupiterLendEarnDepositTool] Starting tool execution with OpenTelemetry tracing");
        let start_time = Instant::now();
        info!("DEBUG: JupiterLendEarnDepositTool called with user_pubkey={}, asset_mint={}, amount={}",
              args.user_pubkey, args.asset_mint, args.amount);
        info!("DEBUG: Tool key_map contains: {:?}", self.key_map);

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
                    .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
            } else {
                info!(
                    "Could not resolve {} from key_map, using simulated pubkey for lend deposit",
                    args.user_pubkey
                );
                Pubkey::from_str("11111111111111111111111111111111")
                    .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
            }
        } else {
            Pubkey::from_str(&args.user_pubkey)
                .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
        };

        let asset_mint =
            if args.asset_mint.starts_with("USER_") || args.asset_mint.starts_with("RECIPIENT_") {
                if let Some(resolved_mint) = self.key_map.get(&args.asset_mint) {
                    info!(
                        "Resolved {} from key_map: {}",
                        args.asset_mint, resolved_mint
                    );
                    Pubkey::from_str(resolved_mint)
                        .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
                } else {
                    info!(
                        "Could not resolve {} from key_map, using simulated mint for lend deposit",
                        args.asset_mint
                    );
                    Pubkey::from_str("So11111111111111111111111111111111111111112")
                        .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
                }
            } else {
                Pubkey::from_str(&args.asset_mint)
                    .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
            };

        // Validate business logic
        if args.amount == 0 {
            return Err(JupiterLendEarnDepositError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Use shared balance validation utility
        let balance_validator = BalanceValidator::new(self.key_map.clone());

        match balance_validator
            .validate_token_balance(&asset_mint.to_string(), &args.user_pubkey, args.amount)
            .map_err(JupiterLendEarnDepositError::from)
        {
            Ok(()) => {
                info!(
                    "✅ Balance validation passed: requested {} for mint {}",
                    args.amount, asset_mint
                );

                // Log the available balance for debugging
                if let Ok(available) =
                    balance_validator.get_token_balance(&asset_mint.to_string(), &args.user_pubkey)
                {
                    info!("Available balance: {}", available);
                }
            }
            Err(e) => {
                warn!(
                    "❌ Balance validation failed for mint {}: {}",
                    asset_mint, e
                );

                // Provide helpful guidance for insufficient funds errors
                if let JupiterLendEarnDepositError::BalanceValidation(boxed_err) = &e {
                    if let BalanceValidationError::InsufficientFunds {
                        requested,
                        available,
                    } = boxed_err.as_ref()
                    {
                        warn!(
                            "💡 Suggestion: Use get_account_balance tool to check available balance before depositing. \
                            Available: {}, Requested: {}",
                            available, requested
                        );
                    }
                }

                return Err(e);
            }
        }

        // Call the protocol handler
        let protocol_start_time = Instant::now();
        let raw_instructions = handle_jupiter_lend_deposit(user_pubkey, asset_mint, args.amount)
            .await
            .map_err(JupiterLendEarnDepositError::ProtocolCall)?;
        let protocol_execution_time = protocol_start_time.elapsed().as_millis() as u32;
        let total_execution_time = start_time.elapsed().as_millis() as u32;

        info!(
            "[JupiterLendEarnDepositTool] Protocol execution completed - protocol_time: {}ms, total_time: {}ms, instructions: {}",
            protocol_execution_time, total_execution_time, raw_instructions.len()
        );

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
