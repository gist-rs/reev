//! Dynamic Context Updates for Enhanced Context Passing
//!
//! This module implements dynamic context updates that can be applied
//! after each operation to ensure proper wallet state updates and
//! accurate constraints for subsequent operations.

use anyhow::Result;
use reev_types::flow::{StepResult, TokenBalance, WalletContext};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, instrument};

use super::constraints::{ConstraintBuilder, StepConstraint};
use super::operation_history::{BalanceCalculator, OperationHistory, OperationHistoryBuilder};

/// Dynamic context updater that modifies the operation context based on tool execution results
pub struct DynamicContextUpdater {
    /// Wallet context to update
    wallet_context: WalletContext,
    /// Balance calculator for tracking changes
    balance_calculator: BalanceCalculator,
    /// Operation history builder
    operation_history_builder: OperationHistoryBuilder,
}

impl DynamicContextUpdater {
    /// Create a new dynamic context updater
    pub fn new(wallet_context: WalletContext) -> Self {
        // Create initial balances map from wallet context
        let mut initial_balances = HashMap::new();
        for (mint, token_balance) in &wallet_context.token_balances {
            initial_balances.insert(mint.clone(), token_balance.balance as f64);
        }

        // Include SOL balance
        initial_balances.insert(
            "So11111111111111111111111111111111111111112".to_string(),
            wallet_context.sol_balance as f64,
        );

        Self {
            wallet_context,
            balance_calculator: BalanceCalculator::new(initial_balances),
            operation_history_builder: OperationHistoryBuilder::new(),
        }
    }

    /// Update context after tool execution
    #[instrument(skip(self, tool_result, tool_name))]
    pub fn update_context_after_execution(
        &mut self,
        tool_result: &Value,
        tool_name: &str,
        step_result: &StepResult,
    ) -> Result<ContextUpdateResult> {
        info!("Updating context after {} execution", tool_name);

        // Extract balance changes from tool result
        let balance_changes = self.extract_balance_changes(tool_result, tool_name)?;
        info!("Extracted {} balance changes", balance_changes.len());

        // Create operation history entries
        let operation_history = OperationHistory::from_step_result(step_result);
        info!(
            "Created {} operation history entries",
            operation_history.len()
        );

        // Add to history builder
        for operation in operation_history {
            self.operation_history_builder.add_operation(operation);
        }

        // Update balance calculator with new operations
        let operations = self.operation_history_builder.clone().build();
        self.balance_calculator.add_operations(operations);

        // Update wallet context with new balances
        self.update_wallet_context()?;

        // Generate constraints for next steps
        let next_step_constraints =
            self.generate_constraints_for_next_step(tool_result, tool_name)?;
        info!(
            "Generated {} constraints for next step",
            next_step_constraints.len()
        );

        Ok(ContextUpdateResult {
            balance_changes,
            next_step_constraints,
            updated_wallet_context: self.wallet_context.clone(),
            operation_history: self.operation_history_builder.clone().build(),
        })
    }

    /// Extract balance changes from tool result
    fn extract_balance_changes(
        &self,
        tool_result: &Value,
        tool_name: &str,
    ) -> Result<Vec<BalanceChange>> {
        let mut changes = Vec::new();

        match tool_name {
            "jupiter_swap" => {
                if let Some(swap_info) = tool_result.get("jupiter_swap") {
                    if let (
                        Some(input_mint),
                        Some(output_mint),
                        Some(input_amount),
                        Some(output_amount),
                    ) = (
                        swap_info.get("input_mint").and_then(|v| v.as_str()),
                        swap_info.get("output_mint").and_then(|v| v.as_str()),
                        swap_info.get("input_amount").and_then(|v| v.as_u64()),
                        swap_info.get("output_amount").and_then(|v| v.as_u64()),
                    ) {
                        // Add input change (negative)
                        let input_symbol = self
                            .wallet_context
                            .token_balances
                            .get(input_mint)
                            .and_then(|t| t.symbol.clone());

                        changes.push(BalanceChange {
                            mint: input_mint.to_string(),
                            balance_before: input_amount,
                            balance_after: 0,
                            change_amount: -(input_amount as i64),
                            symbol: input_symbol,
                        });

                        // Add output change (positive)
                        let output_symbol = self
                            .wallet_context
                            .token_balances
                            .get(output_mint)
                            .and_then(|t| t.symbol.clone());

                        changes.push(BalanceChange {
                            mint: output_mint.to_string(),
                            balance_before: 0,
                            balance_after: output_amount,
                            change_amount: output_amount as i64,
                            symbol: output_symbol,
                        });
                    }
                }
            }
            "jupiter_lend" => {
                if let Some(lend_info) = tool_result.get("jupiter_lend") {
                    if let (Some(asset_mint), Some(amount)) = (
                        lend_info.get("asset_mint").and_then(|v| v.as_str()),
                        lend_info.get("amount").and_then(|v| v.as_u64()),
                    ) {
                        let symbol = self
                            .wallet_context
                            .token_balances
                            .get(asset_mint)
                            .and_then(|t| t.symbol.clone());

                        changes.push(BalanceChange {
                            mint: asset_mint.to_string(),
                            balance_before: amount,
                            balance_after: 0,
                            change_amount: -(amount as i64),
                            symbol,
                        });
                    }
                }
            }
            "sol_transfer" => {
                if let Some(transfer_info) = tool_result.get("sol_transfer") {
                    if let Some(amount) = transfer_info.get("amount").and_then(|v| v.as_u64()) {
                        changes.push(BalanceChange {
                            mint: "So11111111111111111111111111111111111111112".to_string(),
                            balance_before: amount,
                            balance_after: 0,
                            change_amount: -(amount as i64),
                            symbol: Some("SOL".to_string()),
                        });
                    }
                }
            }
            _ => {
                info!("No specific balance extraction for tool: {}", tool_name);
            }
        }

        Ok(changes)
    }

    /// Update wallet context with new balances
    fn update_wallet_context(&mut self) -> Result<()> {
        // Get all available balances after operations
        let updated_balances = self.balance_calculator.calculate_all_balances();

        // Update token balances in wallet context
        for (mint, balance) in updated_balances {
            if mint == "So11111111111111111111111111111111111111112" {
                // Update SOL balance
                self.wallet_context.sol_balance = balance as u64;
            } else if let Some(token_balance) = self.wallet_context.token_balances.get_mut(&mint) {
                // Update existing token balance
                token_balance.balance = balance as u64;
            } else {
                // Add new token if it wasn't in the original context
                info!(
                    "Adding new token {} to wallet context with balance {}",
                    mint, balance
                );
                self.wallet_context.token_balances.insert(
                    mint.clone(),
                    TokenBalance {
                        mint,
                        balance: balance as u64,
                        decimals: Some(6),      // Default decimals
                        symbol: None,           // We don't have symbol information
                        formatted_amount: None, // We don't have formatted amount
                        owner: Some(self.wallet_context.owner.clone()),
                    },
                );
            }
        }

        Ok(())
    }

    /// Generate constraints for the next step based on current operation result
    fn generate_constraints_for_next_step(
        &self,
        tool_result: &Value,
        tool_name: &str,
    ) -> Result<Vec<StepConstraint>> {
        let mut constraint_builder = ConstraintBuilder::new();

        match tool_name {
            "jupiter_swap" => {
                if let Some(swap_info) = tool_result.get("jupiter_swap") {
                    if let (Some(output_mint), Some(output_amount)) = (
                        swap_info.get("output_mint").and_then(|v| v.as_str()),
                        swap_info.get("output_amount").and_then(|v| v.as_u64()),
                    ) {
                        // Add constraint to use exactly the output amount from previous swap
                        constraint_builder.max_amount(output_amount as f64, Some("amount"));
                        constraint_builder.required_mint(output_mint, Some("input_mint"));
                    }
                }
            }
            "jupiter_lend" => {
                if let Some(lend_info) = tool_result.get("jupiter_lend") {
                    if let (Some(asset_mint), Some(_amount)) = (
                        lend_info.get("asset_mint").and_then(|v| v.as_str()),
                        lend_info.get("amount").and_then(|v| v.as_u64()),
                    ) {
                        // Add constraint to exclude the lent asset as it's no longer available
                        constraint_builder.excluded_mint(asset_mint, Some("input_mint"));
                    }
                }
            }
            _ => {
                // No specific constraints for other tools
            }
        }

        // Add general constraints based on wallet state
        self.add_general_constraints(&mut constraint_builder)?;

        Ok(constraint_builder.build())
    }

    /// Add general constraints based on current wallet state
    fn add_general_constraints(&self, builder: &mut ConstraintBuilder) -> Result<()> {
        // Add constraints based on available balances
        for token_balance in self.wallet_context.token_balances.values() {
            if token_balance.balance > 0 {
                // Can't use more than available balance
                builder.max_amount(token_balance.balance as f64, Some("amount"));
            }
        }

        // Add constraint for SOL balance
        if self.wallet_context.sol_balance > 0 {
            builder.max_amount(self.wallet_context.sol_balance as f64, Some("amount"));
        }

        Ok(())
    }

    /// Get the current wallet context
    pub fn get_wallet_context(&self) -> &WalletContext {
        &self.wallet_context
    }

    /// Get the current operation history
    pub fn get_operation_history(&self) -> Vec<OperationHistory> {
        self.operation_history_builder.clone().build()
    }

    /// Get the current balance calculator
    pub fn get_balance_calculator(&self) -> &BalanceCalculator {
        &self.balance_calculator
    }
}

/// Result of a context update
#[derive(Debug, Clone)]
pub struct ContextUpdateResult {
    /// Balance changes detected
    pub balance_changes: Vec<BalanceChange>,
    /// Constraints for the next step
    pub next_step_constraints: Vec<StepConstraint>,
    /// Updated wallet context
    pub updated_wallet_context: WalletContext,
    /// Operation history
    pub operation_history: Vec<OperationHistory>,
}

/// Balance change information
#[derive(Debug, Clone)]
pub struct BalanceChange {
    /// Mint address of the token
    pub mint: String,
    /// Balance before the operation
    pub balance_before: u64,
    /// Balance after the operation
    pub balance_after: u64,
    /// Amount of change (can be negative)
    pub change_amount: i64,
    /// Token symbol if available
    pub symbol: Option<String>,
}

/// Utility for creating context update prompts for the AI
pub struct ContextPromptBuilder {
    operation_history: Vec<OperationHistory>,
    wallet_context: WalletContext,
    next_step_constraints: Vec<StepConstraint>,
}

impl ContextPromptBuilder {
    /// Create a new context prompt builder
    pub fn new(
        operation_history: Vec<OperationHistory>,
        wallet_context: WalletContext,
        next_step_constraints: Vec<StepConstraint>,
    ) -> Self {
        Self {
            operation_history,
            wallet_context,
            next_step_constraints,
        }
    }

    /// Build a prompt describing the current context
    pub fn build_context_prompt(&self) -> String {
        let mut prompt = String::new();

        // Add wallet state
        prompt.push_str("## Current Wallet State\n\n");
        prompt.push_str(&format!("Public Key: {}\n", self.wallet_context.owner));
        prompt.push_str(&format!(
            "SOL Balance: {}\n\n",
            self.wallet_context.sol_balance
        ));

        prompt.push_str("### Token Balances\n\n");
        for (mint, token) in &self.wallet_context.token_balances {
            if let Some(symbol) = &token.symbol {
                prompt.push_str(&format!("- {}: {} ({})\n", symbol, token.balance, mint));
            } else {
                prompt.push_str(&format!("- Unknown ({}): {}\n", mint, token.balance));
            }
        }

        // Add operation history
        if !self.operation_history.is_empty() {
            prompt.push_str("\n## Previous Operations\n\n");
            for (i, operation) in self.operation_history.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, operation.get_summary()));
            }
        }

        // Add constraints for next step
        if !self.next_step_constraints.is_empty() {
            prompt.push_str("\n## Constraints for Next Operation\n\n");
            for (i, constraint) in self.next_step_constraints.iter().enumerate() {
                prompt.push_str(&format!(
                    "{}. {}\n",
                    i + 1,
                    constraint.constraint_type.description()
                ));
            }
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reev_types::flow::StepResult;

    #[test]
    fn test_dynamic_context_updater() {
        let mut wallet_context = WalletContext::new("test_pubkey".to_string());
        // Already set with WalletContext::new above
        wallet_context.sol_balance = 1000;

        // Add a token to the wallet
        wallet_context.token_balances.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                balance: 500,
                symbol: Some("USDC".to_string()),
                decimals: Some(6),
                formatted_amount: Some("500 USDC".to_string()),
                owner: Some("test_pubkey".to_string()),
            },
        );

        let mut updater = DynamicContextUpdater::new(wallet_context);

        // Simulate a swap operation result
        let tool_result = serde_json::json!({
            "jupiter_swap": {
                "input_mint": "So11111111111111111111111111111111111111112",
                "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "input_amount": 100,
                "output_amount": 500
            }
        });

        let step_result = StepResult {
            step_id: "step1".to_string(),
            success: true,
            error_message: None,
            tool_calls: vec!["jupiter_swap".to_string()],
            output: serde_json::json!({ "tool_results": [tool_result.clone()] }),
            execution_time_ms: 100,
        };

        // Update context
        let update_result = updater
            .update_context_after_execution(&tool_result, "jupiter_swap", &step_result)
            .unwrap();

        // Verify balance changes
        assert_eq!(update_result.balance_changes.len(), 2);

        // Verify constraints were generated
        assert!(!update_result.next_step_constraints.is_empty());

        // Verify wallet context was updated
        assert_eq!(update_result.updated_wallet_context.sol_balance, 900);

        // Find USDC balance in updated context
        let usdc_balance = update_result
            .updated_wallet_context
            .token_balances
            .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            .map(|t| t.balance)
            .unwrap_or(0);

        assert_eq!(usdc_balance, 1000); // Original 500 + 500 from swap
    }

    #[test]
    fn test_context_prompt_builder() {
        let wallet_context = WalletContext::new("test_pubkey".to_string());
        let operation_history = Vec::new();
        let next_step_constraints = Vec::new();

        let builder =
            ContextPromptBuilder::new(operation_history, wallet_context, next_step_constraints);

        let prompt = builder.build_context_prompt();
        assert!(prompt.contains("Current Wallet State"));
        assert!(prompt.contains("Public Key:"));
        assert!(prompt.contains("SOL Balance:"));
    }
}
