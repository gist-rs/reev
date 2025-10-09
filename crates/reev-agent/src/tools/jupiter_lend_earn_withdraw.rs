//! Jupiter lend earn withdraw tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn/withdraw functionality.
//! It acts as a thin wrapper around the protocol handler.

use crate::protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

/// The arguments for the Jupiter lend earn withdraw tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendEarnWithdrawArgs {
    pub user_pubkey: String,
    pub asset_mint: String,
    pub amount: u64,
}

/// A custom error type for the Jupiter lend earn withdraw tool.
#[derive(Debug, Error)]
pub enum JupiterLendEarnWithdrawError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter protocol call failed: {0}")]
    ProtocolCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
}

/// A `rig` tool for performing lend earn withdrawal operations using the Jupiter API.
/// This tool acts as a thin wrapper around the protocol handler.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendEarnWithdrawTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendEarnWithdrawTool {
    const NAME: &'static str = "jupiter_lend_earn_withdraw";
    type Error = JupiterLendEarnWithdrawError;
    type Args = JupiterLendEarnWithdrawArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let asset_mint_description = format!(
            "The mint address of the token to be withdrawn. For native SOL, use '{}'. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'.",
            native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Withdraw tokens from Jupiter lending position. Use when user wants to 'withdraw', 'remove', or 'take out' a specific amount of tokens. Works with token amounts (e.g., 50000000 for 50 USDC).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the withdrawal. This should be a valid 44-character base58 encoded Solana public key. This wallet must sign the transaction."
                    },
                    "asset_mint": {
                        "type": "string",
                        "description": asset_mint_description
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the token to withdraw, in its smallest denomination (e.g., lamports for SOL). This corresponds to the amount of the underlying asset, not the L-token."
                    }
                },
                "required": ["user_pubkey", "asset_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the protocol handler.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate and parse arguments
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?;

        // Validate business logic
        if args.amount == 0 {
            return Err(JupiterLendEarnWithdrawError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Call the protocol handler
        let raw_instructions =
            handle_jupiter_lend_withdraw(user_pubkey, asset_mint, args.amount, &self.key_map)
                .await
                .map_err(JupiterLendEarnWithdrawError::ProtocolCall)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
