//! Jupiter lend earn withdraw tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn/withdraw functionality.
//! It acts as a thin wrapper around the protocol handler.

use crate::protocols::jupiter::lend_withdraw::handle_jupiter_lend_withdraw;

use reev_lib::constants::usdc_mint;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use tracing::info;

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
            "The mint address of the token to be withdrawn. For native SOL, use '{}'. For USDC, use '{}'.",
            native_mint::ID,
            usdc_mint()
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "EXCLUSIVE tool for WITHDRAWING tokens from Jupiter lending. This is a COMPLETE operation - returns ALL instructions needed to withdraw tokens to your wallet. Use ONLY when user specifically says 'withdraw', 'withdrawing', or mentions withdrawing token amounts. DO NOT use for 'redeem' operations - use jupiter_lend_earn_redeem instead. Works with token amounts (e.g., lamports for SOL). If user mentions 'redeem', use jupiter_lend_earn_redeem.".to_string(),
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
                    .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
            } else {
                info!(
                    "Could not resolve {} from key_map, using simulated pubkey for lend withdraw",
                    args.user_pubkey
                );
                Pubkey::from_str("11111111111111111111111111111111")
                    .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
            }
        } else {
            Pubkey::from_str(&args.user_pubkey)
                .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
        };

        let asset_mint =
            if args.asset_mint.starts_with("USER_") || args.asset_mint.starts_with("RECIPIENT_") {
                if let Some(resolved_mint) = self.key_map.get(&args.asset_mint) {
                    info!(
                        "Resolved {} from key_map: {}",
                        args.asset_mint, resolved_mint
                    );
                    Pubkey::from_str(resolved_mint)
                        .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
                } else {
                    info!(
                        "Could not resolve {} from key_map, using simulated mint for lend withdraw",
                        args.asset_mint
                    );
                    Pubkey::from_str("So11111111111111111111111111111111111111112")
                        .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
                }
            } else {
                Pubkey::from_str(&args.asset_mint)
                    .map_err(|e| JupiterLendEarnWithdrawError::PubkeyParse(e.to_string()))?
            };

        // Validate business logic
        if args.amount == 0 {
            return Err(JupiterLendEarnWithdrawError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Call the protocol handler
        let raw_instructions = handle_jupiter_lend_withdraw(user_pubkey, asset_mint, args.amount)
            .await
            .map_err(JupiterLendEarnWithdrawError::ProtocolCall)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
