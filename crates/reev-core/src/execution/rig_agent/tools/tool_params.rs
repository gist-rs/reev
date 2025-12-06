//! Tool Parameters
//!
//! This module contains typed parameter structs for each tool implementation
//! to replace the untyped HashMap<String, String> parameter passing.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for SOL transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolTransferParams {
    /// Recipient address
    pub recipient: String,
    /// Amount to transfer (in SOL or "all")
    pub amount: String,
}

impl SolTransferParams {
    /// Create from a HashMap of parameters
    pub fn from_hashmap(params: &HashMap<String, String>) -> Result<Self> {
        let recipient = params
            .get("recipient")
            .ok_or_else(|| anyhow!("recipient parameter is required"))?
            .clone();

        let amount = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?
            .clone();

        Ok(Self { recipient, amount })
    }
}

/// Parameters for SPL token transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplTransferParams {
    /// Recipient address
    pub recipient: String,
    /// Amount to transfer (in token units or "all")
    pub amount: String,
    /// Mint address of the token
    pub mint_address: String,
}

impl SplTransferParams {
    /// Create from a HashMap of parameters
    pub fn from_hashmap(params: &HashMap<String, String>) -> Result<Self> {
        let recipient = params
            .get("recipient")
            .ok_or_else(|| anyhow!("recipient parameter is required"))?
            .clone();

        let amount = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?
            .clone();

        // Accept both token and mint_address parameter names
        let mint_address = params
            .get("mint_address")
            .or_else(|| params.get("token"))
            .ok_or_else(|| anyhow!("mint_address parameter is required"))?
            .clone();

        Ok(Self {
            recipient,
            amount,
            mint_address,
        })
    }
}

/// Parameters for Jupiter swap operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapParams {
    /// Input token mint address
    pub input_mint: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount to swap (in lamports)
    pub input_amount: String,
}

impl JupiterSwapParams {
    /// Create from a HashMap of parameters
    pub fn from_hashmap(params: &HashMap<String, String>) -> Result<Self> {
        let input_mint = params
            .get("input_mint")
            .ok_or_else(|| anyhow!("input_mint parameter is required"))?
            .clone();

        let output_mint = params
            .get("output_mint")
            .ok_or_else(|| anyhow!("output_mint parameter is required"))?
            .clone();

        // Accept both input_amount and amount parameter names
        let input_amount = params
            .get("input_amount")
            .or_else(|| params.get("amount"))
            .ok_or_else(|| anyhow!("input_amount parameter is required"))?
            .clone();

        Ok(Self {
            input_mint,
            output_mint,
            input_amount,
        })
    }
}

/// Parameters for Jupiter lend/earn operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLendParams {
    /// Token mint address
    pub mint: String,
    /// Amount to deposit (in smallest denomination)
    pub amount: String,
}

impl JupiterLendParams {
    /// Create from a HashMap of parameters
    pub fn from_hashmap(params: &HashMap<String, String>) -> Result<Self> {
        let mint = params
            .get("mint")
            .ok_or_else(|| anyhow!("mint parameter is required"))?
            .clone();

        let amount = params
            .get("amount")
            .ok_or_else(|| anyhow!("amount parameter is required"))?
            .clone();

        Ok(Self { mint, amount })
    }
}

/// Parameters for account balance queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceParams {
    /// Account address to query
    pub account: String,
    /// Token mint address (default is SOL)
    pub mint: String,
}

impl AccountBalanceParams {
    /// Create from a HashMap of parameters
    pub fn from_hashmap(params: &HashMap<String, String>) -> Result<Self> {
        let account = params
            .get("account")
            .ok_or_else(|| anyhow!("account parameter is required"))?
            .clone();

        let default_mint = "So11111111111111111111111111111111111111112".to_string();
        let mint = params.get("mint").cloned().unwrap_or(default_mint);

        Ok(Self { account, mint })
    }
}

/// Enum to represent all possible tool parameters
#[derive(Debug, Clone)]
pub enum ToolParams {
    /// SOL transfer parameters
    SolTransfer(SolTransferParams),
    /// SPL transfer parameters
    SplTransfer(SplTransferParams),
    /// Jupiter swap parameters
    JupiterSwap(JupiterSwapParams),
    /// Jupiter lend/earn parameters
    JupiterLend(JupiterLendParams),
    /// Account balance parameters
    AccountBalance(AccountBalanceParams),
}

impl ToolParams {
    /// Create ToolParams from tool name and parameter hashmap
    pub fn from_tool_name_and_params(
        tool_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<Self> {
        match tool_name {
            "sol_transfer" => Ok(Self::SolTransfer(SolTransferParams::from_hashmap(params)?)),
            "spl_transfer" => Ok(Self::SplTransfer(SplTransferParams::from_hashmap(params)?)),
            "jupiter_swap" => Ok(Self::JupiterSwap(JupiterSwapParams::from_hashmap(params)?)),
            "jupiter_lend_earn_deposit" => {
                Ok(Self::JupiterLend(JupiterLendParams::from_hashmap(params)?))
            }
            "get_account_balance" => Ok(Self::AccountBalance(AccountBalanceParams::from_hashmap(
                params,
            )?)),
            _ => Err(anyhow!("Unknown tool name: {tool_name}")),
        }
    }

    // The tool_name method is currently unused but might be useful in the future
    // pub fn tool_name(&self) -> &str {
    //     match self {
    //         ToolParams::SolTransfer(_) => "sol_transfer",
    //         ToolParams::SplTransfer(_) => "spl_transfer",
    //         ToolParams::JupiterSwap(_) => "jupiter_swap",
    //         ToolParams::JupiterLend(_) => "jupiter_lend_earn_deposit",
    //         ToolParams::AccountBalance(_) => "get_account_balance",
    //     }
    // }
}
