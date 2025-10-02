use crate::jupiter::lend::handle_jupiter_withdraw;
use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

/// The arguments for the Jupiter lend withdraw tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterLendWithdrawArgs {
    pub user_pubkey: String,
    pub asset_mint: String,
    pub amount: u64,
}

/// A custom error type for the Jupiter lend withdraw tool.
#[derive(Debug, Error)]
pub enum JupiterLendWithdrawError {
    #[error("Failed to parse pubkey: {0}")]
    PubkeyParse(String),
    #[error("Jupiter API call failed: {0}")]
    ApiCall(#[from] anyhow::Error),
    #[error("Failed to serialize instruction: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A `rig` tool for performing lend withdrawal operations using the Jupiter API.
#[derive(Deserialize, Serialize)]
pub struct JupiterLendWithdrawTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterLendWithdrawTool {
    const NAME: &'static str = "jupiter_lend_withdraw";
    type Error = JupiterLendWithdrawError;
    type Args = JupiterLendWithdrawArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        let asset_mint_description = format!(
            "The mint address of the token to be withdrawn. For native SOL, use '{}'. For USDC, use 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'.",
            native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Withdraw a token that was previously deposited via Jupiter Lend. This prepares the local forked environment for the transaction.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the withdrawal. This wallet must sign the transaction."
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

    /// Executes the tool's logic by calling the centralized `handle_jupiter_withdraw` function.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendWithdrawError::PubkeyParse(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendWithdrawError::PubkeyParse(e.to_string()))?;

        let raw_instructions =
            handle_jupiter_withdraw(user_pubkey, asset_mint, args.amount, &self.key_map).await?;

        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}
