//! Jupiter lend deposit tool wrapper
//!
//! This tool provides AI agent access to Jupiter's lend/deposit functionality.
//! It correctly handles the on-chain prerequisites for depositing native SOL,
//! which involves creating instructions to wrap SOL into WSOL before calling the
//! Jupiter Program.

use anyhow::Result;
use bs58;
use jup_sdk::{models::DepositParams, Jupiter};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_instruction};
use spl_associated_token_account;
use spl_token;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

/// The arguments for the Jupiter lend deposit tool, which will be provided by the AI model.
/// This matches the pattern of other working Jupiter tools.
#[derive(Deserialize, Debug)]
pub struct JupiterLendDepositArgs {
    pub user_pubkey: String,
    pub asset_mint: String,
    pub amount: u64,
}

/// A custom error type for the Jupiter lend deposit tool.
#[derive(Debug, Error)]
pub enum JupiterLendDepositError {
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for performing lend deposit operations using the Jupiter API.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendDepositTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendDepositTool {
    const NAME: &'static str = "jupiter_lend_deposit";
    type Error = JupiterLendDepositError;
    type Args = JupiterLendDepositArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let asset_mint_description = format!(
            "The mint address of the token to be lent. For native SOL, use '{}'. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'.",
            spl_token::native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Deposit a token to earn yield using the Jupiter LST aggregator. This tool handles the entire process, including wrapping native SOL if required.".to_string(),
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

    /// Executes the tool's logic: calls the internal handler to get instructions.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendDepositError::InvalidPubkey(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendDepositError::InvalidPubkey(e.to_string()))?;

        let raw_instructions = self
            .handle_jupiter_deposit(user_pubkey, asset_mint, args.amount)
            .await
            .map_err(JupiterLendDepositError::ProtocolError)?;

        let output =
            serde_json::to_string(&raw_instructions).map_err(JupiterLendDepositError::JsonError)?;

        Ok(output)
    }
}

impl JupiterLendDepositTool {
    /// Internal handler for Jupiter lend deposit operations.
    /// This correctly initializes the `jup-sdk` client and handles prerequisite
    /// instructions like wrapping SOL.
    async fn handle_jupiter_deposit(
        &self,
        user_pubkey: Pubkey,
        asset_mint: Pubkey,
        amount: u64,
    ) -> Result<Vec<RawInstruction>> {
        let mut setup_instructions: Vec<Instruction> = Vec::new();

        // If depositing native SOL, prerequisite instructions to wrap it are required.
        if asset_mint == spl_token::native_mint::ID {
            let wsol_ata = spl_associated_token_account::get_associated_token_address(
                &user_pubkey,
                &spl_token::native_mint::ID,
            );

            setup_instructions = vec![
                // 1. Create the associated token account for WSOL if it doesn't exist.
                spl_associated_token_account::instruction::create_associated_token_account(
                    &user_pubkey,
                    &user_pubkey,
                    &spl_token::native_mint::ID,
                    &spl_token::ID,
                ),
                // 2. Transfer native SOL to the ATA, which wraps it.
                system_instruction::transfer(&user_pubkey, &wsol_ata, amount),
                // 3. Sync the ATA to ensure the WSOL balance is recognized.
                spl_token::instruction::sync_native(&spl_token::ID, &wsol_ata)?,
            ];
        }

        // The jup-sdk's client is designed to work with a local validator.
        let jupiter_client = Jupiter::surfpool().with_user_pubkey(user_pubkey);

        let deposit_params = DepositParams { asset_mint, amount };

        // The sdk's deposit builder will handle instruction generation
        // against the local surfpool instance.
        let (jupiter_sdk_instructions, _alt_accounts) = jupiter_client
            .deposit(deposit_params)
            .prepare_transaction_components()
            .await?;

        // Combine setup instructions with Jupiter instructions and convert them to the agent's format.
        let all_sdk_instructions = [setup_instructions, jupiter_sdk_instructions].concat();

        let raw_instructions = all_sdk_instructions
            .into_iter()
            .map(|inst| {
                let accounts = inst
                    .accounts
                    .into_iter()
                    .map(|acc| RawAccountMeta {
                        pubkey: acc.pubkey.to_string(),
                        is_signer: acc.is_signer,
                        is_writable: acc.is_writable,
                    })
                    .collect();

                RawInstruction {
                    program_id: inst.program_id.to_string(),
                    accounts,
                    data: bs58::encode(inst.data).into_string(),
                }
            })
            .collect();

        Ok(raw_instructions)
    }
}
