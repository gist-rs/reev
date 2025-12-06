//! Account Balance Tool Implementation
//!
//! This module contains the implementation of the account balance tool.

use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use std::collections::HashMap;

use super::tool_results::{AccountBalanceResult, ToolResult};

/// Execute get account balance
pub async fn execute_get_account_balance(
    params: &HashMap<String, String>,
    _wallet_context: &WalletContext,
) -> Result<ToolResult> {
    let account = params
        .get("account")
        .ok_or_else(|| anyhow!("account parameter is required"))?;

    let default_mint = "So11111111111111111111111111111111111111112".to_string();
    let mint = params.get("mint").unwrap_or(&default_mint);

    // Mock balance for now
    // In a real implementation, this would query the blockchain
    let balance = match mint.as_str() {
        "So11111111111111111111111111111111111111112" => {
            // Mock SOL balance
            rand::random::<u64>() % 10_000_000_000
        }
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => {
            // Mock USDC balance
            rand::random::<u64>() % 1_000_000_000
        }
        _ => {
            // Mock other token balance
            rand::random::<u64>() % 1_000_000_000
        }
    };

    Ok(ToolResult::AccountBalance(AccountBalanceResult {
        tool_name: "get_account_balance".to_string(),
        account: account.to_string(),
        mint: mint.to_string(),
        balance,
        success: true,
        error: None,
    }))
}
