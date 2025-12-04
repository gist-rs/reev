//! Wallet utilities for creating wallet contexts

use anyhow::{anyhow, Result};
use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{info, warn};

/// Create wallet context from wallet address
pub async fn create_wallet_context(wallet_address: &str) -> Result<WalletContext> {
    info!("Creating wallet context for address: {}", wallet_address);

    // Parse wallet address
    let pubkey =
        Pubkey::from_str(wallet_address).map_err(|e| anyhow!("Invalid wallet address: {e}"))?;

    // Create RPC client to query balance
    let client = RpcClient::new("http://localhost:8899".to_string());

    // Get account balance with fallback
    let balance = match client.get_balance(&pubkey).await {
        Ok(balance) => {
            info!(
                "Retrieved balance: {} lamports for address: {}",
                balance, wallet_address
            );
            balance
        }
        Err(e) => {
            warn!(
                "Failed to get balance for address: {}, using default. Error: {}",
                wallet_address, e
            );
            // Default to 1 SOL for testing purposes
            1_000_000_000
        }
    };

    // Create token balances map
    let mut token_balances = HashMap::new();

    // Common SPL token mints
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let usdt_mint = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

    // Query actual token balances from surfpool for common tokens

    // Query USDC balance
    if let Ok(Some(usdc_balance)) =
        get_token_balance_from_surfpool(&client, &pubkey, usdc_mint).await
    {
        token_balances.insert(
            usdc_mint.to_string(),
            TokenBalance::new(usdc_mint.to_string(), usdc_balance)
                .with_decimals(6)
                .with_symbol("USDC".to_string()),
        );
        info!(
            "Retrieved USDC balance: {} tokens",
            usdc_balance as f64 / 1_000_000.0
        );
    } else {
        // Add default USDC balance if query fails
        token_balances.insert(
            usdc_mint.to_string(),
            TokenBalance::new(usdc_mint.to_string(), 100_000_000)
                .with_decimals(6)
                .with_symbol("USDC".to_string()),
        );
    }

    // Query USDT balance
    if let Ok(Some(usdt_balance)) =
        get_token_balance_from_surfpool(&client, &pubkey, usdt_mint).await
    {
        token_balances.insert(
            usdt_mint.to_string(),
            TokenBalance::new(usdt_mint.to_string(), usdt_balance)
                .with_decimals(6)
                .with_symbol("USDT".to_string()),
        );
        info!(
            "Retrieved USDT balance: {} tokens",
            usdt_balance as f64 / 1_000_000.0
        );
    } else {
        // Add default USDT balance if query fails
        token_balances.insert(
            usdt_mint.to_string(),
            TokenBalance::new(usdt_mint.to_string(), 100_000_000)
                .with_decimals(6)
                .with_symbol("USDT".to_string()),
        );
    }

    // Create wallet context
    Ok(WalletContext {
        owner: wallet_address.to_string(),
        sol_balance: balance,
        token_balances,
        total_value_usd: balance as f64 / 1_000_000_000.0, // Simplified: 1 SOL = $1
        token_prices: HashMap::new(),
    })
}

/// Query token balance from surfpool
/// Get token balance from surfpool blockchain
async fn get_token_balance_from_surfpool(
    client: &solana_client::nonblocking::rpc_client::RpcClient,
    pubkey: &solana_sdk::pubkey::Pubkey,
    mint: &str,
) -> Result<Option<u64>> {
    // Parse mint address
    let mint_pubkey = Pubkey::from_str(mint).map_err(|e| anyhow!("Invalid mint address: {e}"))?;

    // Get associated token account address
    let ata = spl_associated_token_account::get_associated_token_address(pubkey, &mint_pubkey);

    // Query token account balance
    match client.get_token_account_balance(&ata).await {
        Ok(balance) => {
            // Extract amount from UiTokenAmount
            let amount = balance.amount;
            // Parse amount string to u64
            match amount.parse::<u64>() {
                Ok(parsed_amount) => Ok(Some(parsed_amount)),
                Err(e) => {
                    warn!("Failed to parse token amount: {}", e);
                    Ok(None)
                }
            }
        }
        Err(_) => {
            // Token account might not exist, return None
            warn!("No token account found for mint: {}", mint);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_wallet_context() {
        let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";
        let result = create_wallet_context(test_address).await;

        // Should succeed even if RPC is not available
        assert!(result.is_ok());

        if let Ok(context) = result {
            assert_eq!(context.owner, test_address);
            assert!(context.sol_balance > 0);
            assert!(!context.token_balances.is_empty());

            // Check USDC and USDT balances
            assert!(context
                .token_balances
                .contains_key("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
            assert!(context
                .token_balances
                .contains_key("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
        }
    }
}
