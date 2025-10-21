//! Native Solana operations tool wrappers
//!
//! These tools provide AI agent access to native Solana operations
//! including SOL transfers and SPL token transfers, acting as thin wrappers
//! around the protocol handlers.

use crate::tool_names::{SOL_TRANSFER, SPL_TRANSFER};
// Tool tracking is now handled by OpenTelemetry + rig framework
// No manual tool wrapper imports needed
use reev_protocols::native::{handle_sol_transfer, handle_spl_transfer};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, instrument};

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
    const NAME: &'static str = SOL_TRANSFER;
    type Error = NativeTransferError;
    type Args = NativeTransferArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer SOL or SPL tokens between Solana accounts. This tool can perform native SOL transfers or SPL token transfers with proper instruction generation. NOTE: If you don't see account balance information in the context, use get_account_balance tool first to verify sufficient funds before transferring.".to_string(),
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
    #[instrument(
        name = "sol_transfer_tool_call",
        skip(self),
        fields(
            tool_name = "sol_transfer",
            user_pubkey = %args.user_pubkey,
            recipient_pubkey = %args.recipient_pubkey,
            amount = args.amount,
            operation = ?args.operation
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[SolTransferTool] Starting tool execution with OpenTelemetry tracing");
        // Validate and parse arguments
        let user_pubkey = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.user_pubkey)
            .clone();

        let user_pubkey_parsed = Pubkey::from_str(&user_pubkey)
            .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

        let recipient_pubkey = self
            .key_map
            .get(&args.recipient_pubkey)
            .unwrap_or(&args.recipient_pubkey)
            .clone();

        let recipient_pubkey_parsed = Pubkey::from_str(&recipient_pubkey)
            .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

        // Validate business logic
        if args.amount == 0 {
            return Err(NativeTransferError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Start timing for flow tracking
        let start_time = Instant::now();

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

        let execution_time = start_time.elapsed().as_millis() as u32;

        info!(
            "[SolTransferTool] Tool execution completed - total_time: {}ms, operation: {:?}",
            execution_time, args.operation
        );

        // Record flow data
        let _tool_args = json!({
            "user_pubkey": args.user_pubkey,
            "recipient_pubkey": args.recipient_pubkey,
            "amount": args.amount,
            "operation": args.operation,
            "mint_address": args.mint_address
        })
        .to_string();

        // Tool calls are now automatically tracked by OpenTelemetry + rig framework
        // No manual tracking needed anymore

        // Tool execution completed successfully
        info!(
            "[SolTransferTool] Successfully created transfer with {} instructions",
            raw_instructions.len()
        );

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
    const NAME: &'static str = SPL_TRANSFER;
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
    #[instrument(
        name = "spl_transfer_tool_call",
        skip(self),
        fields(
            tool_name = "spl_transfer",
            user_pubkey = %args.user_pubkey,
            recipient_pubkey = %args.recipient_pubkey,
            amount = args.amount,
            mint_address = ?args.mint_address
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[SplTransferTool] Starting tool execution with OpenTelemetry tracing");
        // Force SPL operation and validate mint address
        if args.mint_address.is_none() {
            return Err(NativeTransferError::MintAddressRequired);
        }

        let mut spl_args = args;
        spl_args.operation = NativeTransferOperation::Spl;

        info!("[SplTransferTool] Delegating to SOL transfer tool with SPL operation");

        // Delegate to the SOL transfer tool with SPL operation
        let sol_tool = SolTransferTool {
            key_map: self.key_map.clone(),
        };
        sol_tool.call(spl_args).await
    }
}
