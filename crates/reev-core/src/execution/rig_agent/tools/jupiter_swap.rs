//! Jupiter Swap Tool Implementation
//!
//! This module contains the implementation of the Jupiter swap tool.

use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

/// Execute Jupiter swap
pub async fn execute_jupiter_swap(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<serde_json::Value> {
    let input_mint = params
        .get("input_mint")
        .ok_or_else(|| anyhow!("input_mint parameter is required"))?;

    let output_mint = params
        .get("output_mint")
        .ok_or_else(|| anyhow!("output_mint parameter is required"))?;

    let amount_str = params
        .get("input_amount")
        .or_else(|| params.get("amount"))
        .ok_or_else(|| anyhow!("input_amount parameter is required"))?;

    // Check if amount is "all" before parsing to float
    let is_all_amount = amount_str.to_lowercase() == "all";

    let amount: f64 = if is_all_amount {
        // For "all", use the SOL balance directly
        wallet_context.sol_balance as f64 / 1_000_000_000.0
    } else {
        amount_str
            .parse()
            .map_err(|_| anyhow!("Invalid amount: {amount_str}"))?
    };

    // Convert amount to lamports (1 SOL = 1,000,000,000 lamports)
    let amount_lamports = (amount * 1_000_000_000.0) as u64;

    // Parse the mint addresses
    let input_mint_pubkey =
        Pubkey::from_str(input_mint).map_err(|e| anyhow!("Invalid input mint: {e}"))?;
    let output_mint_pubkey =
        Pubkey::from_str(output_mint).map_err(|e| anyhow!("Invalid output mint: {e}"))?;

    // Parse user pubkey
    let user_pubkey =
        Pubkey::from_str(&wallet_context.owner).map_err(|e| anyhow!("Invalid user pubkey: {e}"))?;

    // Use full balance if amount is "all", otherwise use specified amount
    let final_amount_lamports = if is_all_amount {
        // Reserve 0.01 SOL for gas fees
        wallet_context
            .sol_balance
            .saturating_sub(reev_lib::constants::amounts::tokens::sol::JUPITER_SWAP_FEE_RESERVE)
        // Reserve 0.01 SOL for fees
    } else {
        amount_lamports
    };

    // Use the newer protocol handler from reev-protocols
    let instructions = reev_protocols::jupiter::swap::handle_jupiter_swap(
        user_pubkey,
        input_mint_pubkey,
        output_mint_pubkey,
        final_amount_lamports,
        100, // Default 1% slippage
    )
    .await
    .map_err(|e| anyhow!("Failed to prepare swap transaction: {e}"))?;

    // Get the default keypair for signing
    let keypair = reev_lib::get_keypair().map_err(|e| anyhow!("Failed to get keypair: {e}"))?;

    // Execute the transaction
    let transaction_signature = reev_lib::execute_transaction(instructions, user_pubkey, &keypair)
        .await
        .map_err(|e| anyhow!("Failed to execute transaction: {e}"))?;

    info!(
        "Jupiter swap executed with signature: {}",
        transaction_signature
    );

    Ok(json!({
        "tool_name": "jupiter_swap",
        "input_mint": input_mint,
        "output_mint": output_mint,
        "amount": final_amount_lamports,
        "wallet": wallet_context.owner,
        "transaction_signature": transaction_signature,
        "success": true
    }))
}
