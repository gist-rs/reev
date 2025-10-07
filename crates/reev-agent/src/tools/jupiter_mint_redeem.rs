//! Jupiter mint and redeem tools wrapper
//!
//! This tool provides AI agent access to Jupiter's mint and redeem functionality
//! for lending positions.

use jup_sdk::api::{get_mint_instructions, get_redeem_instructions};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// Helper function to convert Jupiter InstructionData to JSON string
fn convert_instructions_to_json(
    instructions: &[jup_sdk::models::InstructionData],
) -> Result<String, JupiterMintRedeemError> {
    let converted: Vec<serde_json::Value> = instructions
        .iter()
        .map(|inst| {
            serde_json::json!({
                "program_id": inst.program_id,
                "accounts": inst.accounts.iter().map(|acc| {
                    serde_json::json!({
                        "pubkey": acc.pubkey,
                        "is_signer": acc.is_signer,
                        "is_writable": acc.is_writable
                    })
                }).collect::<Vec<_>>(),
                "data": inst.data
            })
        })
        .collect();

    serde_json::to_string(&converted).map_err(JupiterMintRedeemError::JsonError)
}

/// The arguments for the Jupiter mint tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterMintArgs {
    pub asset: String,
    pub signer: String,
    pub shares: u64,
}

/// The arguments for the Jupiter redeem tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterRedeemArgs {
    pub asset: String,
    pub signer: String,
    pub shares: u64,
}

/// A custom error type for the Jupiter mint/redeem tools.
#[derive(Debug, Error)]
pub enum JupiterMintRedeemError {
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for accessing Jupiter's mint functionality.
#[derive(Deserialize, Serialize)]
pub struct JupiterMintTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterMintTool {
    const NAME: &'static str = "jupiter_mint";
    type Error = JupiterMintRedeemError;
    type Args = JupiterMintArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Mint Jupiter lending positions. This tool creates instructions to mint jTokens representing lending positions in Jupiter's lending markets.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who will own the minted position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of shares to mint in the smallest unit (for USDC with 6 decimals, 1000000 = 1 USDC)"
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
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Asset cannot be empty".to_string(),
            ));
        }
        if args.signer.is_empty() {
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Signer cannot be empty".to_string(),
            ));
        }
        if args.shares == 0 {
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Shares must be greater than 0".to_string(),
            ));
        }

        // Get the resolved signer from key_map or use the provided one
        let signer = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.signer)
            .clone();

        // Call the Jupiter API to get mint instructions
        let response = get_mint_instructions(args.asset.clone(), signer.clone(), args.shares)
            .await
            .map_err(JupiterMintRedeemError::ProtocolError)?;

        // Convert InstructionData to JSON string
        let instructions_json = convert_instructions_to_json(&response.instructions)?;
        let output = instructions_json;

        // Create the final response with context
        let response = json!({
            "tool": "jupiter_mint",
            "asset": args.asset,
            "signer": signer,
            "shares": args.shares,
            "instructions": serde_json::from_str::<serde_json::Value>(&output)?,
            "note": "These instructions mint jTokens representing lending positions. Execute them to create the position."
        });

        Ok(response.to_string())
    }
}

/// A `rig` tool for accessing Jupiter's redeem functionality.
#[derive(Deserialize, Serialize)]
pub struct JupiterRedeemTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterRedeemTool {
    const NAME: &'static str = "jupiter_redeem";
    type Error = JupiterMintRedeemError;
    type Args = JupiterRedeemArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Redeem Jupiter lending positions. This tool creates instructions to redeem jTokens and withdraw the underlying assets from Jupiter's lending markets.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "asset": {
                        "type": "string",
                        "description": "The token mint address (e.g., USDC mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "signer": {
                        "type": "string",
                        "description": "The public key of the user who owns the lending position"
                    },
                    "shares": {
                        "type": "integer",
                        "description": "The amount of shares to redeem in the smallest unit (for USDC with 6 decimals, 1000000 = 1 USDC)"
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
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Asset cannot be empty".to_string(),
            ));
        }
        if args.signer.is_empty() {
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Signer cannot be empty".to_string(),
            ));
        }
        if args.shares == 0 {
            return Err(JupiterMintRedeemError::InvalidArguments(
                "Shares must be greater than 0".to_string(),
            ));
        }

        // Get the resolved signer from key_map or use the provided one
        let signer = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.signer)
            .clone();

        // Call the Jupiter API to get redeem instructions
        let response = get_redeem_instructions(args.asset.clone(), signer.clone(), args.shares)
            .await
            .map_err(JupiterMintRedeemError::ProtocolError)?;

        // Convert InstructionData to JSON string
        let instructions_json = convert_instructions_to_json(&response.instructions)?;
        let output = instructions_json;

        // Create the final response with context
        let response = json!({
            "tool": "jupiter_redeem",
            "asset": args.asset,
            "signer": signer,
            "shares": args.shares,
            "instructions": serde_json::from_str::<serde_json::Value>(&output)?,
            "note": "These instructions redeem jTokens and withdraw the underlying assets from lending positions. Execute them to close the position."
        });

        Ok(response.to_string())
    }
}
