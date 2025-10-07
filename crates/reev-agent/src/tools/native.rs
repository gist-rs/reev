//! Native Solana operations tool wrappers
//!
//! These tools provide AI agent access to native Solana operations
//! including SOL transfers and SPL token transfers, acting as thin wrappers
//! around the protocol handlers.

use crate::protocols::native::{handle_sol_transfer, handle_spl_transfer};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// The arguments for the native transfer tool, which will be provided by the AI model.
#[derive(Serialize, Deserialize, Debug)]
pub struct NativeTransferArgs {
    pub user_pubkey: String,
    pub recipient_pubkey: String,
    pub amount: u64,
    #[serde(default)]
    pub operation: NativeTransferOperation,
    pub mint_address: Option<String>, // Required for SPL transfers
}

/// Native transfer operations
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum NativeTransferOperation {
    #[default]
    Sol,
    Spl,
}

/// A custom error type for the native transfer tool.
#[derive(Debug, Error)]
pub enum NativeTransferError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Mint address required for SPL transfers")]
    MintAddressRequired,
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Native protocol call failed: {0}")]
    ProtocolCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing native Solana transfers.
/// This tool provides access to both SOL and SPL token transfers.
#[derive(Deserialize, Serialize)]
pub struct SolTransferTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for SolTransferTool {
    const NAME: &'static str = "sol_transfer";
    type Error = NativeTransferError;
    type Args = NativeTransferArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer SOL or SPL tokens between Solana accounts. This tool can perform native SOL transfers or SPL token transfers with proper instruction generation.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet sending the transfer."
                    },
                    "recipient_pubkey": {
                        "type": "string",
                        "description": "The public key of the recipient wallet."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount to transfer in the smallest denomination (lamports for SOL)."
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["sol", "spl"],
                        "description": "The type of transfer: 'sol' for native SOL, 'spl' for SPL tokens."
                    },
                    "mint_address": {
                        "type": "string",
                        "description": "The mint address of the SPL token (required for SPL transfers)."
                    }
                },
                "required": ["user_pubkey", "recipient_pubkey", "amount", "operation"],
            }),
        }
    }

    /// Executes the tool's logic: validates arguments and calls the appropriate protocol handler.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate and parse arguments
        let user_pubkey = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.user_pubkey)
            .clone();

        let user_pubkey_parsed = Pubkey::from_str(&user_pubkey)
            .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

        let recipient_pubkey_parsed = Pubkey::from_str(&args.recipient_pubkey)
            .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

        // Validate business logic
        if args.amount == 0 {
            return Err(NativeTransferError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Call the appropriate protocol handler
        let raw_instructions = match args.operation {
            NativeTransferOperation::Sol => handle_sol_transfer(
                user_pubkey_parsed,
                recipient_pubkey_parsed,
                args.amount,
                &self.key_map,
            )
            .await
            .map_err(NativeTransferError::ProtocolCall)?,
            NativeTransferOperation::Spl => {
                let mint_address = args
                    .mint_address
                    .clone()
                    .ok_or_else(|| NativeTransferError::MintAddressRequired)?;
                let _mint_pubkey = Pubkey::from_str(&mint_address)
                    .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

                // For SPL transfers, we need to determine the source and destination token accounts
                // using the mint to find associated token accounts
                let source = spl_associated_token_account::get_associated_token_address(
                    &user_pubkey_parsed,
                    &_mint_pubkey,
                );

                // The agent should provide the correct recipient ATA address directly
                // Use recipient_pubkey as the destination ATA without recalculating
                let destination = recipient_pubkey_parsed;

                handle_spl_transfer(
                    source,
                    destination,
                    user_pubkey_parsed,
                    args.amount,
                    &self.key_map,
                )
                .await
                .map_err(NativeTransferError::ProtocolCall)?
            }
        };

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}

/// A `rig` tool for performing SPL token transfers.
#[derive(Deserialize, Serialize)]
pub struct SplTransferTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for SplTransferTool {
    const NAME: &'static str = "spl_transfer";
    type Error = NativeTransferError;
    type Args = NativeTransferArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer SPL tokens between Solana accounts. This tool handles SPL token transfers with proper associated token account management.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet sending the transfer."
                    },
                    "recipient_pubkey": {
                        "type": "string",
                        "description": "The public key of the recipient wallet."
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount to transfer in the smallest denomination of the token."
                    },
                    "mint_address": {
                        "type": "string",
                        "description": "The mint address of the SPL token to transfer."
                    }
                },
                "required": ["user_pubkey", "recipient_pubkey", "amount", "mint_address"],
            }),
        }
    }

    /// Executes the tool's logic: creates SPL transfer instructions.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Force SPL operation and validate mint address
        if args.mint_address.is_none() {
            return Err(NativeTransferError::MintAddressRequired);
        }

        let mut spl_args = args;
        spl_args.operation = NativeTransferOperation::Spl;

        // Delegate to the SOL transfer tool with SPL operation
        let sol_tool = SolTransferTool {
            key_map: self.key_map.clone(),
        };
        sol_tool.call(spl_args).await
    }
}
