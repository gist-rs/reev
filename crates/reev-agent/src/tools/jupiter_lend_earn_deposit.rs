//! Jupiter lend earn deposit tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn/deposit functionality.
//! It acts as a thin wrapper around the protocol handler.

use crate::protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use tracing::info;

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
            "The mint address of the token to be lent. For native SOL, use '{}'. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'.",
            native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "PRIMARY tool for depositing tokens into Jupiter lending. Use when user wants to 'deposit', 'lend', or 'mint jTokens by depositing' tokens. Works with token amounts like '0.1 SOL' or '50 USDC'. This is the standard way to mint jTokens - use this unless user specifically mentions share quantities.".to_string(),
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
                        "description": "The amount of the token to deposit, in its smallest denomination (e.g., lamports for SOL)."
                    }
                },
                "required": ["user_pubkey", "asset_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the protocol handler.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Check for placeholder addresses that would cause Base58 parsing errors
        let user_pubkey = if args.user_pubkey.starts_with("USER_")
            || args.user_pubkey.starts_with("RECIPIENT_")
        {
            info!("Detected placeholder user_pubkey, using simulated pubkey for lend deposit");
            Pubkey::from_str("11111111111111111111111111111111")
                .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
        } else {
            Pubkey::from_str(&args.user_pubkey)
                .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
        };

        let asset_mint =
            if args.asset_mint.starts_with("USER_") || args.asset_mint.starts_with("RECIPIENT_") {
                info!("Detected placeholder asset_mint, using simulated mint for lend deposit");
                Pubkey::from_str("So11111111111111111111111111111111111111112")
                    .map_err(|e| JupiterLendEarnDepositError::PubkeyParse(e.to_string()))?
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

        // Call the protocol handler
        let raw_instructions =
            handle_jupiter_lend_deposit(user_pubkey, asset_mint, args.amount, &self.key_map)
                .await
                .map_err(JupiterLendEarnDepositError::ProtocolCall)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
