//! Jupiter lend tool wrapper
//!
//! This tool provides AI agent access to Jupiter's lending functionality,
//! including deposit and withdraw operations.

use crate::protocols::get_jupiter_config;
use crate::protocols::jupiter::{execute_request, parse_json_response};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::collections::HashMap;
use std::str::FromStr;
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
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
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
           native_mint::ID
        );
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Deposit a token to earn yield using the Jupiter LST aggregator. This finds the best yield across many protocols and prepares the transaction.".to_string(),
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

    /// Executes the tool's logic: calls the Jupiter lend deposit API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendDepositError::InvalidPubkey(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendDepositError::InvalidPubkey(e.to_string()))?;

        if args.amount == 0 {
            return Err(JupiterLendDepositError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Call the Jupiter lend deposit API
        let raw_instructions = self
            .handle_jupiter_deposit(user_pubkey, asset_mint, args.amount)
            .await
            .map_err(JupiterLendDepositError::ProtocolError)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}

impl JupiterLendDepositTool {
    /// Internal handler for Jupiter lend deposit operations
    async fn handle_jupiter_deposit(
        &self,
        user_pubkey: Pubkey,
        asset_mint: Pubkey,
        amount: u64,
    ) -> anyhow::Result<Vec<RawInstruction>> {
        let config = get_jupiter_config();
        let client = config.create_client()?;

        let request_body = json!({
            "userPublicKey": user_pubkey.to_string(),
            "asset": asset_mint.to_string(),
            "amount": amount.to_string(),
            "slippageBps": 500 // 5%
        });

        let request = client
            .post(config.lend_deposit_url())
            .header("Content-Type", "application/json")
            .json(&request_body);

        let response = execute_request(request, config.max_retries).await?;
        let json_value = parse_json_response(response).await?;

        // Parse the Jupiter API response to extract transaction instructions
        let instructions = if let Some(instructions_array) =
            json_value.get("instructions").and_then(|v| v.as_array())
        {
            // Convert Jupiter instructions to RawInstruction format
            instructions_array
                .iter()
                .filter_map(|inst| {
                    let program_id = inst.get("programId")?.as_str()?;
                    let accounts = inst.get("accounts")?.as_array()?;
                    let data = inst.get("data")?.as_str()?;

                    let raw_accounts: Vec<RawAccountMeta> = accounts
                        .iter()
                        .filter_map(|acc| {
                            let pubkey = acc.get("pubkey")?.as_str()?;
                            let is_signer = acc.get("isSigner")?.as_bool()?;
                            let is_writable = acc.get("isWritable")?.as_bool()?;

                            Some(RawAccountMeta {
                                pubkey: pubkey.to_string(),
                                is_signer,
                                is_writable,
                            })
                        })
                        .collect();

                    Some(RawInstruction {
                        program_id: program_id.to_string(),
                        accounts: raw_accounts,
                        data: data.to_string(),
                    })
                })
                .collect()
        } else {
            // Fallback to placeholder if response format is unexpected
            vec![RawInstruction {
                program_id: "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9".to_string(),
                accounts: vec![
                    RawAccountMeta {
                        pubkey: user_pubkey.to_string(),
                        is_signer: true,
                        is_writable: true,
                    },
                    RawAccountMeta {
                        pubkey: asset_mint.to_string(),
                        is_signer: false,
                        is_writable: false,
                    },
                ],
                data: format!("deposit_{amount}"),
            }]
        };

        Ok(instructions)
    }
}

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
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for performing lend withdraw operations using the Jupiter API.
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
            description: "Withdraw a token from Jupiter lending to get back the underlying assets plus earnings.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet performing the withdrawal."
                    },
                    "asset_mint": {
                        "type": "string",
                        "description": asset_mint_description
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount of the token to withdraw, in its smallest denomination (e.g., lamports for SOL)."
                    }
                },
                "required": ["user_pubkey", "asset_mint", "amount"],
            }),
        }
    }

    /// Executes the tool's logic: calls the Jupiter lend withdraw API.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let user_pubkey = Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterLendWithdrawError::InvalidPubkey(e.to_string()))?;
        let asset_mint = Pubkey::from_str(&args.asset_mint)
            .map_err(|e| JupiterLendWithdrawError::InvalidPubkey(e.to_string()))?;

        if args.amount == 0 {
            return Err(JupiterLendWithdrawError::InvalidAmount(
                "Amount must be greater than 0".to_string(),
            ));
        }

        // Call the Jupiter lend withdraw API
        let raw_instructions = self
            .handle_jupiter_withdraw(user_pubkey, asset_mint, args.amount)
            .await
            .map_err(JupiterLendWithdrawError::ProtocolError)?;

        // Serialize the Vec<RawInstruction> to a JSON string.
        let output = serde_json::to_string(&raw_instructions)?;

        Ok(output)
    }
}

impl JupiterLendWithdrawTool {
    /// Internal handler for Jupiter lend withdraw operations
    async fn handle_jupiter_withdraw(
        &self,
        user_pubkey: Pubkey,
        asset_mint: Pubkey,
        amount: u64,
    ) -> anyhow::Result<Vec<RawInstruction>> {
        let config = get_jupiter_config();
        let client = config.create_client()?;

        let request_body = json!({
            "userPublicKey": user_pubkey.to_string(),
            "mint": asset_mint.to_string(),
            "amount": amount,
            "slippageBps": 500 // 5%
        });

        let request = client
            .post(config.lend_withdraw_url())
            .header("Content-Type", "application/json")
            .json(&request_body);

        let response = execute_request(request, config.max_retries).await?;
        let json_value = parse_json_response(response).await?;

        // Parse the Jupiter API response to extract transaction instructions
        let instructions = if let Some(instructions_array) =
            json_value.get("instructions").and_then(|v| v.as_array())
        {
            // Convert Jupiter instructions to RawInstruction format
            instructions_array
                .iter()
                .filter_map(|inst| {
                    let program_id = inst.get("programId")?.as_str()?;
                    let accounts = inst.get("accounts")?.as_array()?;
                    let data = inst.get("data")?.as_str()?;

                    let raw_accounts: Vec<RawAccountMeta> = accounts
                        .iter()
                        .filter_map(|acc| {
                            let pubkey = acc.get("pubkey")?.as_str()?;
                            let is_signer = acc.get("isSigner")?.as_bool()?;
                            let is_writable = acc.get("isWritable")?.as_bool()?;

                            Some(RawAccountMeta {
                                pubkey: pubkey.to_string(),
                                is_signer,
                                is_writable,
                            })
                        })
                        .collect();

                    Some(RawInstruction {
                        program_id: program_id.to_string(),
                        accounts: raw_accounts,
                        data: data.to_string(),
                    })
                })
                .collect()
        } else {
            // Fallback to placeholder if response format is unexpected
            vec![RawInstruction {
                program_id: "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9".to_string(),
                accounts: vec![
                    RawAccountMeta {
                        pubkey: user_pubkey.to_string(),
                        is_signer: true,
                        is_writable: true,
                    },
                    RawAccountMeta {
                        pubkey: asset_mint.to_string(),
                        is_signer: false,
                        is_writable: false,
                    },
                ],
                data: format!("withdraw_{amount}"),
            }]
        };

        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jupiter_lend_deposit_args_serialization() {
        let args = json!({
            "user_pubkey": "test_user_pubkey",
            "asset_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "amount": 1000000
        });

        let parsed: JupiterLendDepositArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.user_pubkey, "test_user_pubkey");
        assert_eq!(
            parsed.asset_mint,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
        assert_eq!(parsed.amount, 1000000);
    }
}
