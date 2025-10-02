use crate::jupiter::lend::handle_jupiter_deposit;
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

/// The arguments for the Jupiter lend deposit tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendDepositArgs {
    pub user_pubkey: String,
    pub asset_mint: String,
    pub amount: u64,
}

/// A custom error type for the Jupiter lend deposit tool.
#[derive(Debug, Error)]
pub enum JupiterLendDepositError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter API call failed: {0}")]
    ApiCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing lend deposit operations using the Jupiter API.
/// This tool requires the on-chain context (`key_map`) to be provided during its construction.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendDepositTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendDepositTool {
    const NAME: &'static str = "jupiter_lend_deposit";
    type Error = JupiterLendDepositError;
    type Args = JupiterLendDepositArgs;
    type Output = String; // The tool will return a JSON string of `Vec<RawInstruction>`.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let asset_mint_description = format!(
            "The mint address of the token to be lent. For native SOL, use '{}'. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'.",
            native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Deposit a token to earn yield using the Jupiter LST aggregator. This finds the best yield across many protocols and prepares the local forked environment for the transaction.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the deposit. This wallet must sign the transaction."
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

    /// Executes the tool's logic: calls the centralized `handle_jupiter_deposit` function,
    /// which transparently handles account pre-loading for the `surfpool` environment.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendDepositError::PubkeyParse(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendDepositError::PubkeyParse(e.to_string()))?;

        // Call the centralized handler, passing the key_map from the struct.
        // This ensures the local surfpool environment is correctly prepared.
        let raw_instructions =
            handle_jupiter_deposit(user_pubkey, asset_mint, args.amount, &self.key_map).await?;

        // Serialize the Vec<RawInstruction> to a JSON string. This is the final output of the tool.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
