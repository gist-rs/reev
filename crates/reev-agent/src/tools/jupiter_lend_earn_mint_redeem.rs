//! Jupiter lend earn mint and redeem tools wrapper
//!
//! This tool provides AI agent access to Jupiter's earn/mint and earn/redeem functionality
//! for lending positions.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// The arguments for the Jupiter lend earn mint tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendEarnMintArgs {
    pub asset: String,
    pub signer: String,
    pub shares: u64,
}

/// The arguments for the Jupiter lend earn redeem tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendEarnRedeemArgs {
    pub asset: String,
    pub signer: String,
    pub shares: u64,
}

/// A custom error type for the Jupiter lend earn mint/redeem tools.
#[derive(Debug, Error)]
pub enum JupiterLendEarnMintRedeemError {
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for accessing Jupiter's lend earn mint functionality.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendEarnMintTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendEarnMintTool {
    const NAME: &'static str = "jupiter_lend_earn_mint";
    type Error = JupiterLendEarnMintRedeemError;
    type Args = JupiterLendEarnMintArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Mint jTokens (shares) in Jupiter lending. Use when user wants to 'mint jTokens', 'create shares', or 'mint positions' based on share count, not token amounts. Works with share counts.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address to deposit (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who will own the minted position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of tokens to deposit/mint in the smallest unit (for USDC with 6 decimals, 50000000 = 50 USDC)"
                    }
                },
                "required": ["asset", "signer", "shares"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the Jupiter API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        if args.asset.is_empty() {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Asset cannot be empty".to_string(),
            ));
        }
        if args.signer.is_empty() {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Signer cannot be empty".to_string(),
            ));
        }
        if args.shares == 0 {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Shares must be greater than 0".to_string(),
            ));
        }

        // Get the resolved signer from key_map or use the provided one
        let signer = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.signer)
            .clone();

        // Use the new lend_mint protocol handler which handles Base58 conversion
        use crate::protocols::jupiter;
        let asset = Pubkey::from_str(&args.asset).map_err(|e| {
            JupiterLendEarnMintRedeemError::ProtocolError(anyhow::anyhow!(
                "Invalid asset pubkey: {e}"
            ))
        })?;
        let shares = args.shares;
        let mut key_map = self.key_map.clone();

        // Ensure USER_WALLET_PUBKEY is in the key_map
        if !key_map.contains_key("USER_WALLET_PUBKEY") {
            key_map.insert("USER_WALLET_PUBKEY".to_string(), signer.clone());
        }

        // Call the centralized lend_mint protocol handler
        let raw_instructions = jupiter::execute_jupiter_lend_mint(&asset, shares, &key_map)
            .await
            .map_err(JupiterLendEarnMintRedeemError::ProtocolError)?;

        // Convert RawInstruction to JSON string
        let instructions_json = serde_json::to_string(&raw_instructions)?;

        // Create the final response with context
        let response = json!({
            "tool": "jupiter_lend_earn_mint",
            "asset": args.asset,
            "signer": signer,
            "shares": args.shares,
            "instructions": serde_json::from_str::<serde_json::Value>(&instructions_json)?,
            "note": "These instructions mint jTokens representing lending positions. Execute them to create the position."
        });

        Ok(response.to_string())
    }
}

/// A `rig` tool for accessing Jupiter's lend earn redeem functionality.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendEarnRedeemTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendEarnRedeemTool {
    const NAME: &'static str = "jupiter_lend_earn_redeem";
    type Error = JupiterLendEarnMintRedeemError;
    type Args = JupiterLendEarnRedeemArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Redeem/burn jTokens (shares) from Jupiter lending. Use when user wants to 'redeem jTokens', 'burn shares', or 'close positions' based on share count, not token amounts. Works with share counts.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address to withdraw (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who owns the lending position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of jTokens to redeem in the smallest unit (for USDC with 6 decimals, 50000000 = 50 jUSDC)"
                    }
                },
                "required": ["asset", "signer", "shares"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the Jupiter API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        if args.asset.is_empty() {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Asset cannot be empty".to_string(),
            ));
        }
        if args.signer.is_empty() {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Signer cannot be empty".to_string(),
            ));
        }
        if args.shares == 0 {
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Shares must be greater than 0".to_string(),
            ));
        }

        // Get the resolved signer from key_map or use the provided one
        let signer = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.signer)
            .clone();

        // Use the new lend_redeem protocol handler which handles Base58 conversion
        use crate::protocols::jupiter;
        let asset = Pubkey::from_str(&args.asset).map_err(|e| {
            JupiterLendEarnMintRedeemError::ProtocolError(anyhow::anyhow!(
                "Invalid asset pubkey: {e}"
            ))
        })?;
        let shares = args.shares;
        let mut key_map = self.key_map.clone();

        // Ensure USER_WALLET_PUBKEY is in the key_map
        if !key_map.contains_key("USER_WALLET_PUBKEY") {
            key_map.insert("USER_WALLET_PUBKEY".to_string(), signer.clone());
        }

        // Call the centralized lend_redeem protocol handler
        let raw_instructions = jupiter::execute_jupiter_lend_redeem(&asset, shares, &key_map)
            .await
            .map_err(JupiterLendEarnMintRedeemError::ProtocolError)?;

        // Convert RawInstruction to JSON string
        let instructions_json = serde_json::to_string(&raw_instructions)?;
        let output = instructions_json;

        // Create the final response with context
        let response = json!({
            "tool": "jupiter_lend_earn_redeem",
            "asset": args.asset,
            "signer": signer,
            "shares": args.shares,
            "instructions": serde_json::from_str::<serde_json::Value>(&output)?,
            "note": "These instructions redeem jTokens and withdraw the underlying assets from lending positions. Execute them to close the position."
        });

        Ok(response.to_string())
    }
}
