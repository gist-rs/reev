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
            description: "EXCLUSIVE tool for MINTING jTokens by SHARES in Jupiter lending. Use ONLY when user specifically says 'mint' or mentions 'minting' jTokens. DO NOT use for 'deposit' operations - use jupiter_lend_earn_deposit instead. Works with share quantities, not token amounts. If user mentions 'deposit', use jupiter_lend_earn_deposit.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address to mint (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who will own the minted position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of SHARES to mint (not token amounts). For token amounts like '0.1 SOL', use jupiter_lend_earn_deposit instead."
                    }
                },
                "required": ["asset", "signer", "shares"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the Jupiter API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate arguments
        tracing::debug!("[jupiter_lend_earn_mint] Starting mint execution with args: asset={}, signer={}, shares={}", args.asset, args.signer, args.shares);

        if args.asset.is_empty() {
            tracing::error!("[jupiter_lend_earn_mint] Asset cannot be empty");
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Asset cannot be empty".to_string(),
            ));
        }
        if args.signer.is_empty() {
            tracing::error!("[jupiter_lend_earn_mint] Signer cannot be empty");
            return Err(JupiterLendEarnMintRedeemError::InvalidArguments(
                "Signer cannot be empty".to_string(),
            ));
        }
        if args.shares == 0 {
            tracing::error!("[jupiter_lend_earn_mint] Shares must be greater than 0");
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

        tracing::debug!("[jupiter_lend_earn_mint] Resolved signer: {}", signer);

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
            description: "EXCLUSIVE tool for REDEEMING jTokens by SHARES from Jupiter lending. Use ONLY when user specifically says 'redeem' or mentions 'redeeming' jTokens. DO NOT use for 'withdraw' operations - use jupiter_lend_earn_withdraw instead. Works with share quantities, not token amounts. If user mentions 'withdraw', use jupiter_lend_earn_withdraw.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address to redeem (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who owns the lending position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of jTokens SHARES to redeem (not token amounts). For token amounts, use jupiter_lend_earn_withdraw instead."
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
