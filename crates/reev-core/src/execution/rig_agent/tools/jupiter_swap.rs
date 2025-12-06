//! Jupiter Swap Tool Implementation
//!
//! This module contains the implementation of the Jupiter swap tool.

use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;

use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

use super::tool_params::JupiterSwapParams;
use super::tool_results::{JupiterSwapResult, ToolResult};

/// Execute Jupiter swap
pub async fn execute_jupiter_swap(
    params: &HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let swap_params = JupiterSwapParams::from_hashmap(params)?;

    // Parse the amount directly from parameters
    // The structured response should have already replaced "all" with the calculated amount
    let amount: f64 = swap_params
        .input_amount
        .parse()
        .map_err(|_| anyhow!("Invalid amount: {}", swap_params.input_amount))?;

    // Debug logging to track amount values
    let is_all_amount = swap_params.input_amount.to_lowercase() == "all";
    info!(
        "Jupiter swap: amount_str='{}', is_all_amount={}, parsed_amount={}",
        swap_params.input_amount, is_all_amount, amount
    );

    // Convert amount to lamports (1 SOL = 1,000,000,000 lamports)
    let _amount_lamports = (amount * 1_000_000_000.0) as u64;

    // Parse the mint addresses
    let input_mint_pubkey = Pubkey::from_str(&swap_params.input_mint)
        .map_err(|e| anyhow!("Invalid input mint: {e}"))?;
    let output_mint_pubkey = Pubkey::from_str(&swap_params.output_mint)
        .map_err(|e| anyhow!("Invalid output mint: {e}"))?;

    // Parse user pubkey
    let user_pubkey =
        Pubkey::from_str(&wallet_context.owner).map_err(|e| anyhow!("Invalid user pubkey: {e}"))?;

    // Convert amount to lamports
    let final_amount_lamports = (amount * 1_000_000_000.0) as u64;

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

    Ok(ToolResult::JupiterSwap(JupiterSwapResult {
        tool_name: "jupiter_swap".to_string(),
        input_mint: swap_params.input_mint.clone(),
        output_mint: swap_params.output_mint.clone(),
        amount: final_amount_lamports,
        wallet: wallet_context.owner.clone(),
        transaction_signature: Some(transaction_signature),
        success: true,
        error: None,
    }))
}
