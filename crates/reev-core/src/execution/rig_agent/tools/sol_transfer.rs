//! SOL Transfer Tool Implementation
//!
//! This module contains the implementation of the SOL transfer tool.

use anyhow::{anyhow, Result};
use reev_protocols::native::handle_sol_transfer;
use reev_types::flow::WalletContext;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::info;

use super::tool_params::SolTransferParams;
use super::tool_results::{SolTransferResult, ToolResult};

/// Execute SOL transfer
pub async fn execute_sol_transfer(
    params: &std::collections::HashMap<String, String>,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Parse parameters into typed struct
    let transfer_params = SolTransferParams::from_hashmap(params)?;
    let recipient = &transfer_params.recipient;
    let amount_str = &transfer_params.amount;

    // Handle "all" keyword case
    let amount: f64 = if amount_str.to_lowercase() == "all" {
        // Calculate transfer amount using standardized gas reserve
        let gas_reserve = crate::gas_reserve::get_gas_reserve_for_action(
            crate::prompt_processor::types::PromptAction::Transfer,
        );

        // Use the standardized function to calculate max transferable amount
        let max_transferable = crate::gas_reserve::calculate_max_sol_transferable(
            wallet_context.sol_balance,
            Some(gas_reserve),
        );

        max_transferable as f64 / 1_000_000_000.0
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

    Ok(ToolResult::SolTransfer(SolTransferResult {
        tool_name: "sol_transfer".to_string(),
        recipient: recipient.to_string(),
        amount,
        amount_lamports,
        wallet: wallet_context.owner.clone(),
        transaction_signature: Some(transaction_signature),
        success: true,
        error: None,
    }))
}

/// Execute SOL transfer using typed parameters
pub async fn execute_sol_transfer_with_params(
    params: &SolTransferParams,
    wallet_context: &WalletContext,
) -> Result<ToolResult> {
    // Convert the typed params back to HashMap to reuse the main function
    let mut params_map = std::collections::HashMap::new();
    params_map.insert("recipient".to_string(), params.recipient.clone());
    params_map.insert("amount".to_string(), params.amount.clone());

    execute_sol_transfer(&params_map, wallet_context).await
}
