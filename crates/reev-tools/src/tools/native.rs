//! Native Solana operations tool wrappers
//!
//! These tools provide AI agent access to native Solana operations
//! including SOL transfers and SPL token transfers, acting as thin wrappers
//! around protocol handlers.

use crate::tool_names::SOL_TRANSFER;
use spl_associated_token_account::get_associated_token_address;
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

// Import enhanced logging macros
use reev_flow::log_tool_call;
use reev_flow::log_tool_completion;
use tracing::{debug, error, info, instrument, warn};

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

/// A custom error type for SPL transfer tool.
#[derive(Debug, Error)]
pub enum SplTransferError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Mint address required for SPL transfers")]
    MintAddressRequired,
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("SPL protocol call failed: {0}")]
    ProtocolCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Failed to create associated token account: {0}")]
    AssociatedTokenAccount(String),
    #[error("Invalid token account: {0}")]
    InvalidTokenAccount(String),
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
            description: "Transfer SOL or SPL tokens between Solana accounts. This tool can perform native SOL transfers or SPL token transfers with proper instruction generation. NOTE: Account balance is provided in the context - verify sufficient funds before transferring.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet sending the transfer."
                    },
                    "recipient_pubkey": {
                        "type": "string",
                        "description": "The public key of the recipient wallet for SOL transfers. Use placeholder names like RECIPIENT_WALLET_PUBKEY."
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
        // Use enhanced logging macro for consistent otel tracking
        log_tool_call!("sol_transfer", &args);

        let _start_time = Instant::now();
        info!("[SolTransferTool] Starting tool execution with OpenTelemetry tracing");
        debug!(
            "[SolTransferTool] tool args: user_pubkey='{}', recipient_pubkey='{}'",
            args.user_pubkey, args.recipient_pubkey
        );

        // Simple resolution for now - use what key_map provides or fallback
        let user_pubkey = self
            .key_map
            .get(&args.user_pubkey)
            .unwrap_or(&args.user_pubkey)
            .clone();

        debug!("[SolTransferTool] resolved user_pubkey: '{}'", user_pubkey);

        let user_pubkey_parsed = Pubkey::from_str(&user_pubkey)
            .map_err(|e| NativeTransferError::PubkeyParse(e.to_string()))?;

        // Enhanced recipient_pubkey resolution with debugging
        info!(
            "[SolTransferTool] Resolving recipient_pubkey: '{}', available keys: {:?}",
            args.recipient_pubkey,
            self.key_map.keys().collect::<Vec<_>>()
        );

        let recipient_pubkey = if let Some(resolved) = self.key_map.get(&args.recipient_pubkey) {
            info!(
                "[SolTransferTool] Directly resolved '{}' to '{}'",
                args.recipient_pubkey, resolved
            );
            resolved.clone()
        } else {
            warn!(
                "[SolTransferTool] '{}' not found in key_map, available keys: {:?}",
                args.recipient_pubkey,
                self.key_map.keys().collect::<Vec<_>>()
            );
            args.recipient_pubkey.clone()
        };

        debug!(
            "[SolTransferTool] resolved recipient_pubkey: '{}'",
            recipient_pubkey
        );

        info!(
            "[SolTransferTool] Final resolved '{}' to '{}'",
            args.recipient_pubkey, recipient_pubkey
        );

        let recipient_pubkey_parsed = Pubkey::from_str(&recipient_pubkey).map_err(|e| {
            error!(
                "[SolTransferTool] Failed to parse recipient_pubkey '{}' (original: '{}'): {}",
                recipient_pubkey, args.recipient_pubkey, e
            );
            NativeTransferError::PubkeyParse(e.to_string())
        })?;

        // Validate business logic
        if args.amount == 0 {
            return Err(NativeTransferError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Start timing for flow tracking
        let start_time = Instant::now();

        // Only handle SOL transfers - SPL transfers use separate SplTransferTool
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
                // SPL transfers should use SplTransferTool, not SolTransferTool
                return Err(NativeTransferError::MintAddressRequired);
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

        let execution_time = start_time.elapsed().as_millis() as u32;
        let result = serde_json::to_string(&raw_instructions)?;

        // Log tool completion with enhanced otel
        log_tool_completion!("sol_transfer", execution_time as u64, &result, true);

        Ok(result)
    }
}

/// A `rig` tool for performing SPL token transfers.
#[derive(Deserialize, Serialize)]
pub struct SplTransferTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for SplTransferTool {
    const NAME: &'static str = "spl_transfer";
    type Error = SplTransferError;
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
                        "description": "The public key of the recipient's token account (ATA) for SPL transfers. Use placeholder names like RECIPIENT_USDC_ATA, not wallet addresses."
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
        info!("[SplTransferTool] Starting SPL transfer execution");

        // Start timing for flow tracking
        let start_time = std::time::Instant::now();

        // Log the full key_map for debugging
        info!(
            "[SplTransferTool] Available key_map entries: {:?}",
            self.key_map
        );

        // Resolve user pubkey first
        let user_pubkey = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.user_pubkey)
            .clone();

        info!("[SplTransferTool] Resolved user_pubkey: {}", user_pubkey);
        let user_pubkey_parsed = Pubkey::from_str(&user_pubkey)
            .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;

        // Validate mint address
        let mint_address = args
            .mint_address
            .ok_or_else(|| SplTransferError::MintAddressRequired)?;
        let mint_pubkey = Pubkey::from_str(&mint_address)
            .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;

        // Resolve source ATA - prioritize key_map ATAs over generated ones
        let source_ata = if let Some(user_ata_key) = self.key_map.get("USER_USDC_ATA") {
            info!(
                "[SplTransferTool] Using pre-created source ATA from key_map: {}",
                user_ata_key
            );
            Pubkey::from_str(user_ata_key)
                .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?
        } else {
            let generated_ata = get_associated_token_address(&user_pubkey_parsed, &mint_pubkey);
            info!(
                "[SplTransferTool] Generated new source ATA: {}",
                generated_ata
            );
            generated_ata
        };

        // Resolve destination ATA - prioritize key_map ATAs over generated ones
        let destination_ata =
            if let Some(recipient_ata_key) = self.key_map.get(&args.recipient_pubkey) {
                info!(
                "[SplTransferTool] Using pre-created destination ATA from key_map: {} (key: {})",
                recipient_ata_key, args.recipient_pubkey
            );
                Pubkey::from_str(recipient_ata_key)
                    .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?
            } else {
                // Only parse recipient pubkey if we need to generate ATA
                let recipient_pubkey = self
                    .key_map
                    .get(&args.recipient_pubkey)
                    .unwrap_or(&args.recipient_pubkey)
                    .clone();

                info!(
                    "[SplTransferTool] Resolving recipient pubkey for ATA generation: {}",
                    recipient_pubkey
                );
                let recipient_pubkey_parsed = Pubkey::from_str(&recipient_pubkey)
                    .map_err(|e| SplTransferError::PubkeyParse(e.to_string()))?;

                let generated_ata =
                    get_associated_token_address(&recipient_pubkey_parsed, &mint_pubkey);
                info!(
                    "[SplTransferTool] Generated new destination ATA: {}",
                    generated_ata
                );
                generated_ata
            };

        // Validate amount
        if args.amount == 0 {
            return Err(SplTransferError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        info!(
            "[SplTransferTool] Transferring {} tokens from {} to {} (mint: {})",
            args.amount, source_ata, destination_ata, mint_pubkey
        );

        // Handle SPL transfer
        let raw_instructions = handle_spl_transfer(
            source_ata,
            destination_ata,
            user_pubkey_parsed,
            args.amount,
            &self.key_map,
        )
        .await
        .map_err(SplTransferError::ProtocolCall)?;

        let execution_time = start_time.elapsed().as_millis() as u32;

        info!(
            "[SplTransferTool] SPL transfer completed - total_time: {}ms, instructions: {}",
            execution_time,
            raw_instructions.len()
        );

        // Serialize the Vec<RawInstruction> to a JSON string.
        serde_json::to_string(&raw_instructions).map_err(SplTransferError::Serialization)
    }
}
