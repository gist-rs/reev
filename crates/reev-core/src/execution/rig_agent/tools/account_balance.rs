//! Account Balance Tool Implementation
//!
//! This module contains the implementation of the account balance tool.

use anyhow::Result;
use reev_types::flow::WalletContext;
use std::collections::HashMap;

use super::tool_params::AccountBalanceParams;
use super::tool_results::{AccountBalanceResult, ToolResult};

/// Execute get account balance
pub async fn execute_get_account_balance(
    params: &AccountBalanceParams,
    _wallet_context: &WalletContext,
) -> Result<ToolResult> {
    let _account = &params.account;
    let mint = &params.mint;

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
        account: params.account.clone(),
        mint: params.mint.clone(),
        balance,
        success: true,
        error: None,
    }))
}

/// Execute get account balance using HashMap parameters
pub async fn execute_get_account_balance_with_hashmap(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let balance_params = AccountBalanceParams::from_hashmap(params)?;
    execute_get_account_balance(&balance_params, wallet_context).await
}
