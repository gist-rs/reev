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
    params: &HashMap<String, String>,
    _wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let balance_params = AccountBalanceParams::from_hashmap(params)?;
    let _account = &balance_params.account;
    let mint = &balance_params.mint;

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
        account: balance_params.account.clone(),
        mint: balance_params.mint.clone(),
        balance,
        success: true,
        error: None,
    }))
}

/// Execute get account balance using typed parameters
pub async fn execute_get_account_balance_with_params(
    params: &AccountBalanceParams,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Convert the typed params back to HashMap to reuse the main function
    let mut params_map = HashMap::new();
    params_map.insert("account".to_string(), params.account.clone());
    params_map.insert("mint".to_string(), params.mint.clone());

    execute_get_account_balance(&params_map, wallet_context).await
}
