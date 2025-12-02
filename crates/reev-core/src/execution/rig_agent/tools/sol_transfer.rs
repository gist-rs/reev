//! SOL Transfer Tool Implementation
//!
//! This module contains the implementation of the SOL transfer tool.

use anyhow::{anyhow, Result};
use reev_protocols::native::handle_sol_transfer;
use reev_types::flow::WalletContext;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

/// Execute SOL transfer
pub async fn execute_sol_transfer(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<serde_json::Value> {
    let recipient = params
        .get("recipient")
        .ok_or_else(|| anyhow!("recipient parameter is required"))?;

    let amount_str = params
        .get("amount")
        .ok_or_else(|| anyhow!("amount parameter is required"))?;

    // Handle "all" keyword case
    let amount: f64 = if amount_str.to_lowercase() == "all" {
        // Calculate transfer amount as wallet balance minus gas reserve
        // Reserve 0.05 SOL for transaction fees (5,000,000 lamports)
        let gas_reserve = 5_000_000u64;

        // Ensure we don't try to transfer more than available
        let available_balance = if wallet_context.sol_balance > gas_reserve {
            wallet_context.sol_balance - gas_reserve
        } else {
            // If balance is less than or equal to gas reserve, transfer half
            wallet_context.sol_balance / 2
        };

        available_balance as f64 / 1_000_000_000.0
    } else {
        amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?
    };

    // Parse recipient and sender pubkeys
    let recipient_pubkey =
        Pubkey::from_str(recipient).map_err(|e| anyhow!("Invalid recipient address: {e}"))?;
    let sender_pubkey = Pubkey::from_str(&wallet_context.owner)
        .map_err(|e| anyhow!("Invalid sender address: {e}"))?;

    // Convert amount to lamports for the protocol handler
    let amount_lamports = (amount * 1_000_000_000.0) as u64;

    // For "all" keyword, pass u64::MAX to let the protocol handler calculate the amount
    let transfer_amount = if amount_str.to_lowercase() == "all" {
        u64::MAX
    } else {
        amount_lamports
    };

    // Use the newer protocol handler from reev-protocols
    let instructions =
        handle_sol_transfer(sender_pubkey, recipient_pubkey, transfer_amount).await?;

    // Get the default keypair for signing
    let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

    // Execute the transaction
    let transaction_signature =
        reev_lib::execute_transaction(instructions, sender_pubkey, &keypair)
            .await
            .map_err(|e| anyhow!("Failed to execute transaction: {e}"))?;

    // Log the successful transaction
    info!(
        "SOL transfer executed with signature: {}",
        transaction_signature
    );

    Ok(json!({
        "tool_name": "sol_transfer",
        "params": {
            "recipient": recipient,
            "amount": amount,
            "amount_lamports": amount_lamports,
            "wallet": wallet_context.owner
        },
        "transaction_signature": transaction_signature,
        "success": true
    }))
}
