//! Context Updater for Wallet State
//!
//! This module handles updating the wallet context after various operations
//! like swaps and lending.

use crate::execution::SharedExecutor;
use anyhow::Result;
use reev_types::flow::{StepResult, WalletContext};
use solana_client::rpc_client::RpcClient;
use tracing::{debug, info, warn};

/// Handles updating wallet context after step execution
pub struct ContextUpdater;

impl ContextUpdater {
    /// Update wallet context after a step execution
    pub async fn update_context_after_step(
        _tool_executor: &SharedExecutor,
        current_context: &WalletContext,
        step_result: &StepResult,
    ) -> Result<WalletContext> {
        info!(
            "Updating wallet context after step: {}",
            step_result.step_id
        );

        // Create a new context based on the current one
        let mut updated_context = current_context.clone();

        // Extract tool results from step output
        // Handle the direct structure returned by RigAgent
        if step_result.output.get("tool_name").is_some() {
            // Handle direct tool result from RigAgent
            if let Some(tool_name) = step_result.output.get("tool_name").and_then(|v| v.as_str()) {
                match tool_name {
                    "jupiter_swap" => {
                        Self::update_context_after_jupiter_swap(&mut updated_context, step_result)
                            .await?;
                    }
                    "jupiter_lend" => {
                        Self::update_context_after_jupiter_lend(&mut updated_context, step_result)?;
                    }
                    _ => {
                        // Unknown tool, skip
                    }
                }
            }
        }
        // Handle nested structure from tool_results array
        else if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    // Check tool type by tool_name field
                    if let Some(tool_name) = result.get("tool_name").and_then(|v| v.as_str()) {
                        match tool_name {
                            "jupiter_swap" => {
                                Self::process_jupiter_swap_result(&mut updated_context, result)
                                    .await?;
                            }
                            "jupiter_lend_earn_deposit" => {
                                Self::process_jupiter_lend_result(&mut updated_context, result)?;
                            }
                            _ => {
                                // Unknown tool, skip
                            }
                        }
                    }
                }
            }
        }

        // Recalculate total value
        updated_context.calculate_total_value();

        info!(
            "Context update completed. SOL: {}, Total value: ${:.2}",
            updated_context.sol_balance_sol(),
            updated_context.total_value_usd
        );

        Ok(updated_context)
    }

    /// Process a Jupiter swap result from a tool_results array entry
    async fn process_jupiter_swap_result(
        updated_context: &mut WalletContext,
        result: &serde_json::Value,
    ) -> Result<()> {
        // Check if swap was successful
        if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
            if success {
                // Extract swap details
                if let (Some(input_mint), Some(output_mint), Some(_tx_signature)) = (
                    result.get("input_mint").and_then(|v| v.as_str()),
                    result.get("output_mint").and_then(|v| v.as_str()),
                    result.get("transaction_signature").and_then(|v| v.as_str()),
                ) {
                    // Get input amount in lamports
                    let input_amount = result
                        .get("input_amount_lamports")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    // Get wallet pubkey
                    let wallet_pubkey = result
                        .get("wallet")
                        .and_then(|v| v.as_str())
                        .unwrap_or_else(|| &updated_context.owner);

                    // Query the blockchain to get actual output amount
                    let rpc_client = solana_client::rpc_client::RpcClient::new(
                        "http://localhost:8899".to_string(),
                    );

                    let actual_output_amount =
                        Self::get_swap_output_amount(&rpc_client, wallet_pubkey, output_mint)
                            .await
                            .unwrap_or(0);

                    info!(
                        "Actual output amount for swap: {} of {}",
                        actual_output_amount, output_mint
                    );

                    // Update SOL balance if SOL was swapped
                    if input_mint == "So11111111111111111111111111111111111111112" {
                        info!(
                            "Updating SOL balance from {} to {} (subtracting {})",
                            updated_context.sol_balance,
                            updated_context.sol_balance.saturating_sub(input_amount),
                            input_amount
                        );
                        updated_context.sol_balance =
                            updated_context.sol_balance.saturating_sub(input_amount);
                    }

                    // Update output token balance
                    if let Some(token_balance) = updated_context.token_balances.get_mut(output_mint)
                    {
                        info!(
                            "Updating {} token balance from {} to {} (adding {})",
                            output_mint,
                            token_balance.balance,
                            token_balance.balance.saturating_add(actual_output_amount),
                            actual_output_amount
                        );
                        debug!(
                            "DEBUG: After updating, USDC balance in context: {}",
                            token_balance.balance.saturating_add(actual_output_amount)
                        );
                        token_balance.balance =
                            token_balance.balance.saturating_add(actual_output_amount);
                    } else {
                        // Create new token entry if it doesn't exist
                        info!(
                            "Creating new token entry for {} with balance {}",
                            output_mint, actual_output_amount
                        );
                        updated_context.token_balances.insert(
                            output_mint.to_string(),
                            reev_types::flow::TokenBalance {
                                balance: actual_output_amount,
                                decimals: Some(6), // Default to 6 decimals for most tokens
                                formatted_amount: None,
                                mint: output_mint.to_string(),
                                owner: Some(updated_context.owner.clone()),
                                symbol: None,
                            },
                        );
                    }

                    info!(
                        "Updated context: swapped {} of {} for {} of {}",
                        input_amount, input_mint, actual_output_amount, output_mint
                    );
                }
            }
        }
        Ok(())
    }

    /// Process a Jupiter Lend result from a tool_results array entry
    fn process_jupiter_lend_result(
        updated_context: &mut WalletContext,
        result: &serde_json::Value,
    ) -> Result<()> {
        // Update context after Jupiter Lend Earn Deposit
        if let (Some(success), Some(asset_mint)) = (
            result.get("success").and_then(|v| v.as_bool()),
            result.get("mint").and_then(|v| v.as_str()),
        ) {
            if success {
                if let Some(amount) = result.get("amount") {
                    let amount = if let Some(i) = amount.as_u64() {
                        i
                    } else if let Some(f) = amount.as_f64() {
                        (f * 1_000_000.0) as u64 // Convert from USDC if needed
                    } else {
                        0
                    };

                    // Update token balance after lending
                    if let Some(token_balance) = updated_context.token_balances.get_mut(asset_mint)
                    {
                        token_balance.balance = token_balance.balance.saturating_sub(amount);
                        info!("Updated context: lent {} of {}", amount, asset_mint);
                    }
                }
            }
        }
        Ok(())
    }

    /// Update wallet context after a Jupiter swap transaction
    async fn update_context_after_jupiter_swap(
        updated_context: &mut WalletContext,
        step_result: &StepResult,
    ) -> Result<()> {
        // First, check if the step was successful
        if !step_result.success {
            warn!("Jupiter swap step failed, not updating context");
            return Ok(());
        }

        // First, try to extract the values from the tool_results array
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    // Check if this is a Jupiter swap result
                    if let Some(jupiter_swap) = result.get("jupiter_swap") {
                        // Extract values from the jupiter_swap object
                        let input_mint = jupiter_swap.get("input_mint").and_then(|v| v.as_str());
                        let output_mint = jupiter_swap.get("output_mint").and_then(|v| v.as_str());

                        // Check if there's an error field in the response
                        // This is the most reliable indicator of failure
                        let has_error = jupiter_swap.get("error").is_some();
                        if has_error {
                            warn!("Jupiter swap failed with error, not updating context");
                            return Ok(());
                        }

                        // Check for transaction signature as another indicator of success
                        let has_tx_signature = jupiter_swap.get("transaction_signature").is_some();

                        if let (Some(input_mint), Some(output_mint)) = (input_mint, output_mint) {
                            // Only update context if we have a transaction signature (indicating success)
                            if has_tx_signature {
                                // Get input amount from swap result
                                let input_amount =
                                    if let Some(amount) = jupiter_swap.get("input_amount") {
                                        if let Some(f) = amount.as_f64() {
                                            (f * 1_000_000_000.0) as u64
                                        } else {
                                            amount.as_u64().unwrap_or_default()
                                        }
                                    } else {
                                        0
                                    };

                                // Get transaction signature from step result
                                let _tx_signature = jupiter_swap
                                    .get("transaction_signature")
                                    .and_then(|v| v.as_str());

                                // Get wallet pubkey
                                let wallet_pubkey = jupiter_swap
                                    .get("wallet")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_else(|| &updated_context.owner);

                                // tx_signature is already an Option<&str> from the destructuring above,
                                // so we don't need to check if it's Some again
                                {
                                    // Query the blockchain to get the actual output amount
                                    let rpc_client = solana_client::rpc_client::RpcClient::new(
                                        "http://localhost:8899".to_string(),
                                    );

                                    // Get the actual output amount from the swap transaction
                                    let actual_output_amount = Self::get_swap_output_amount(
                                        &rpc_client,
                                        wallet_pubkey,
                                        output_mint,
                                    )
                                    .await
                                    .unwrap_or(0);

                                    info!(
                                        "DEBUG: Blockchain query result - {} balance: {}",
                                        output_mint, actual_output_amount
                                    );

                                    info!(
                                        "Actual output amount for swap: {} of {}",
                                        actual_output_amount, output_mint
                                    );

                                    // Update SOL balance if SOL was swapped
                                    if input_mint == "So11111111111111111111111111111111111111112" {
                                        info!(
                                            "Updating SOL balance from {} to {} (subtracting {})",
                                            updated_context.sol_balance,
                                            updated_context
                                                .sol_balance
                                                .saturating_sub(input_amount),
                                            input_amount
                                        );
                                        updated_context.sol_balance = updated_context
                                            .sol_balance
                                            .saturating_sub(input_amount);
                                    }

                                    // Update output token balance - use actual on-chain balance
                                    info!("Checking if token {} exists in context", output_mint);

                                    // Query the current on-chain balance for the token
                                    let current_on_chain_balance = Self::get_swap_output_amount(
                                        &rpc_client,
                                        wallet_pubkey,
                                        output_mint,
                                    )
                                    .await
                                    .unwrap_or(0);

                                    info!(
                                        "Current on-chain balance for {}: {} (using this instead of adding to previous balance)",
                                        output_mint, current_on_chain_balance
                                    );

                                    if let Some(token_balance) =
                                        updated_context.token_balances.get_mut(output_mint)
                                    {
                                        info!(
                                            "Updating {} token balance from {} to {} (using actual on-chain balance)",
                                            output_mint,
                                            token_balance.balance,
                                            current_on_chain_balance
                                        );
                                        token_balance.balance = current_on_chain_balance;
                                    } else {
                                        // Create new token entry if it doesn't exist
                                        info!(
                                            "Creating new token entry for {} with balance {}",
                                            output_mint, current_on_chain_balance
                                        );
                                        updated_context.token_balances.insert(
                                            output_mint.to_string(),
                                            reev_types::flow::TokenBalance {
                                                balance: current_on_chain_balance,
                                                decimals: Some(6), // Default to 6 decimals for most tokens
                                                formatted_amount: None,
                                                mint: output_mint.to_string(),
                                                owner: Some(updated_context.owner.clone()),
                                                symbol: None,
                                            },
                                        );
                                        info!(
                                            "New token entry created: {} -> {}",
                                            output_mint, current_on_chain_balance
                                        );
                                    }

                                    info!(
                                        "Updated context: swapped {} of {} for {} of {}",
                                        input_amount, input_mint, actual_output_amount, output_mint
                                    );
                                }
                            } else {
                                warn!("Jupiter swap failed, not updating context");
                            }
                        }
                    }
                    // Check if this is a regular swap result with tool_name field
                    else if let Some(tool_name) = result.get("tool_name").and_then(|v| v.as_str())
                    {
                        if tool_name == "jupiter_swap" {
                            Self::process_jupiter_swap_result(updated_context, result).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update wallet context after a Jupiter lend transaction
    fn update_context_after_jupiter_lend(
        updated_context: &mut WalletContext,
        step_result: &StepResult,
    ) -> Result<()> {
        if let (Some(asset_mint), Some(success)) = (
            step_result
                .output
                .get("asset_mint")
                .and_then(|v| v.as_str()),
            step_result.output.get("success").and_then(|v| v.as_bool()),
        ) {
            if success {
                let amount = if let Some(a) = step_result.output.get("amount") {
                    a.as_u64().unwrap_or(0)
                } else {
                    0
                };

                // Update token balance after lending (subtract from available balance)
                if let Some(token_balance) = updated_context.token_balances.get_mut(asset_mint) {
                    token_balance.balance = token_balance.balance.saturating_sub(amount);
                    info!("Updated context: lent {} of {}", amount, asset_mint);
                }
            }
        }
        Ok(())
    }

    /// Get actual output amount from a swap transaction by querying token balance
    async fn get_swap_output_amount(
        rpc_client: &RpcClient,
        wallet_pubkey: &str,
        output_mint: &str,
    ) -> Result<u64> {
        info!(
            "Querying token balance for wallet {} and mint {}",
            wallet_pubkey, output_mint
        );

        // Add a small delay to ensure blockchain has fully processed the transaction
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Use token balance query function from reev-lib
        reev_lib::utils::solana::query_token_balance(rpc_client, wallet_pubkey, output_mint)
    }
}
