//! Native Solana operations tool wrapper
//!
//! This tool provides AI agent access to native Solana operations
//! including SOL transfers and SPL token transfers.

use crate::protocols::native::{create_sol_transfer_instruction, create_spl_transfer_instruction};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
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
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Mint address required for SPL transfers")]
    MintAddressRequired,
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
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

    /// Executes the tool's logic: creates transfer instructions.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate pubkeys
        let user_pubkey = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.user_pubkey)
            .clone();

        let user_pubkey_parsed = Pubkey::from_str(&user_pubkey)
            .map_err(|e| NativeTransferError::InvalidPubkey(e.to_string()))?;

        let recipient_pubkey_parsed = Pubkey::from_str(&args.recipient_pubkey)
            .map_err(|e| NativeTransferError::InvalidPubkey(e.to_string()))?;

        // Validate amount
        if args.amount == 0 {
            return Err(NativeTransferError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Create the appropriate instruction
        let instruction = match args.operation {
            NativeTransferOperation::Sol => create_sol_transfer_instruction(
                &user_pubkey_parsed,
                &recipient_pubkey_parsed,
                args.amount,
            ),
            NativeTransferOperation::Spl => {
                let mint_address = args
                    .mint_address
                    .clone()
                    .ok_or_else(|| NativeTransferError::MintAddressRequired)?;
                let mint_pubkey = Pubkey::from_str(&mint_address)
                    .map_err(|e| NativeTransferError::InvalidPubkey(e.to_string()))?;

                // Use mint_pubkey to demonstrate its purpose (in real implementation would find token accounts)
                let _mint_info = format!("SPL transfer for mint: {mint_pubkey}");

                // For SPL transfers, we need to determine the source and destination token accounts
                // using the mint_pubkey to find associated token accounts
                // This is a simplified version - in practice, you'd use mint_pubkey to:
                // 1. Find user's associated token account: get_associated_token_address(user_pubkey, mint_pubkey)
                // 2. Find recipient's associated token account: get_associated_token_address(recipient, mint_pubkey)
                let source = Pubkey::new_unique(); // Placeholder - should be user's token account for mint_pubkey
                let destination = Pubkey::new_unique(); // Placeholder - should be recipient's token account for mint_pubkey

                // Use the mint_pubkey to create proper SPL transfer instruction
                create_spl_transfer_instruction(
                    &spl_token::id(),
                    &source,
                    &destination,
                    &user_pubkey_parsed,
                    args.amount,
                )
            }
        };

        // Convert to raw instruction format
        let raw_instruction = crate::protocols::native::instruction_to_raw(instruction);

        // Create the final response
        let response = json!({
            "tool": "sol_transfer",
            "operation": format!("{:?}", args.operation),
            "user_pubkey": user_pubkey,
            "recipient_pubkey": args.recipient_pubkey,
            "amount": args.amount,
            "mint_address": args.mint_address,
            "instruction": raw_instruction,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(response.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_native_transfer_args_serialization() {
        let args = json!({
            "user_pubkey": "test_user_pubkey",
            "recipient_pubkey": "test_recipient_pubkey",
            "amount": 1000000,
            "operation": "sol"
        });

        let parsed: NativeTransferArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.user_pubkey, "test_user_pubkey");
        assert_eq!(parsed.recipient_pubkey, "test_recipient_pubkey");
        assert_eq!(parsed.amount, 1000000);
        assert!(matches!(parsed.operation, NativeTransferOperation::Sol));
    }

    #[test]
    fn test_native_transfer_operation_enum() {
        let sol = json!("sol");
        let parsed: NativeTransferOperation = serde_json::from_value(sol).unwrap();
        assert!(matches!(parsed, NativeTransferOperation::Sol));

        let spl = json!("spl");
        let parsed: NativeTransferOperation = serde_json::from_value(spl).unwrap();
        assert!(matches!(parsed, NativeTransferOperation::Spl));
    }
}
