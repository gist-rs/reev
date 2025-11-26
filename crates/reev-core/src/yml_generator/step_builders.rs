//! Step Builders for Individual Operations
//!
//! This module implements individual step builders for each operation type
//! as specified in the V3 plan. These step builders create individual steps
//! that can be composed into flows by the unified flow builder.

use crate::refiner::RefinedPrompt;
// Operation is used in doc comments only, removed unused import
use crate::yml_schema::{YmlAssertion, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo};
use anyhow::Result;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use uuid::Uuid;

/// Builder for creating individual swap steps
pub struct SwapStepBuilder;

impl SwapStepBuilder {
    /// Create a swap step from parameters
    pub async fn create_step(
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
        from: &str,
        to: &str,
        amount: f64,
    ) -> Result<YmlStep> {
        // Calculate display amount (account for gas reserve for SOL)
        let display_amount = if from == "SOL" {
            // Account for gas reserve when calculating display amount
            let gas_reserve_lamports = 50_000_000u64; // 0.05 SOL
            let amount_in_lamports = amount * 1_000_000_000.0;

            // Check if this is a "sell all" operation (amount == 0.0)
            if amount == 0.0 {
                // Use most of the balance, leaving gas reserve
                let display_amount =
                    if wallet_context.sol_balance as f64 > gas_reserve_lamports as f64 {
                        wallet_context.sol_balance as f64 - gas_reserve_lamports as f64
                    } else {
                        wallet_context.sol_balance as f64 / 2.0
                    };
                display_amount / 1_000_000_000.0
            } else {
                let display_amount = if amount_in_lamports > gas_reserve_lamports as f64 {
                    amount_in_lamports - gas_reserve_lamports as f64
                } else {
                    amount_in_lamports / 2.0
                };
                display_amount / 1_000_000_000.0
            }
        } else {
            amount
        };

        // Create step prompt
        let step_prompt = if refined_prompt.refined.to_lowercase().contains("all") {
            format!("swap ALL {display_amount} {from} to {to}")
        } else {
            format!("swap {display_amount} {from} to {to}")
        };

        // Create the step
        let step = YmlStep::new(
            Uuid::now_v7().to_string(),
            step_prompt.clone(),
            format!("Exchange {display_amount} {from} for {to}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_expected_tools(vec![ToolName::JupiterSwap])
        .with_critical(true);

        Ok(step)
    }

    /// Create a ground truth assertion for swap steps
    pub async fn create_ground_truth(
        wallet_context: &WalletContext,
        from: &str,
        _to: &str,
        amount: f64,
    ) -> Result<YmlGroundTruth> {
        // Calculate the amount in SOL for balance change
        let amount_sol = if from == "SOL" {
            // Check if this is a "sell all" operation
            if amount == 0.0 {
                // Use most of the balance, leaving gas reserve
                let gas_reserve_lamports = 50_000_000u64; // 0.05 SOL
                if wallet_context.sol_balance as f64 > gas_reserve_lamports as f64 {
                    (wallet_context.sol_balance as f64 - gas_reserve_lamports as f64)
                        / 1_000_000_000.0
                } else {
                    wallet_context.sol_balance as f64 / (1_000_000_000.0 * 2.0)
                }
            } else {
                amount
            }
        } else {
            0.0 // Non-SOL tokens don't affect SOL balance directly
        };

        let ground_truth = YmlGroundTruth::new()
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_gte(
                        -(amount_sol * 1_000_000_000.0 + 50_000_000.0 + 10_000_000.0),
                    ), // Account for swap amount + gas reserve + transaction fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
            .with_error_tolerance(0.01);

        Ok(ground_truth)
    }
}

/// Builder for creating individual transfer steps
pub struct TransferStepBuilder;

impl TransferStepBuilder {
    /// Create a transfer step from parameters
    pub async fn create_step(
        _refined_prompt: &RefinedPrompt,
        _wallet_context: &WalletContext,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<YmlStep> {
        // Create step prompt
        let step_prompt = format!("transfer {amount} {mint} to {to}");

        // Create the step
        let step = YmlStep::new(
            Uuid::now_v7().to_string(),
            step_prompt.clone(),
            format!("Transfer {amount} {mint} to recipient {to}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
        .with_expected_tools(vec![ToolName::SolTransfer])
        .with_critical(true);

        Ok(step)
    }

    /// Create a ground truth assertion for transfer steps
    pub async fn create_ground_truth(
        wallet_context: &WalletContext,
        mint: &str,
        _to: &str,
        amount: f64,
    ) -> Result<YmlGroundTruth> {
        // Only create balance change assertion for SOL transfers
        let ground_truth = if mint == "SOL" {
            YmlGroundTruth::new()
                .with_assertion(
                    YmlAssertion::new("SolBalanceChange".to_string())
                        .with_pubkey(wallet_context.owner.clone())
                        .with_expected_change_lte(-(amount * 1_000_000_000.0 + 5_000_000.0)), // Account for fees
                )
                .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
                .with_error_tolerance(0.01)
        } else {
            // For non-SOL transfers, just verify the tool call was made
            YmlGroundTruth::new()
                .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
                .with_error_tolerance(0.01)
        };

        Ok(ground_truth)
    }
}

/// Builder for creating individual lend steps
pub struct LendStepBuilder;

impl LendStepBuilder {
    /// Create a lend step from parameters
    pub async fn create_step(
        refined_prompt: &RefinedPrompt,
        _wallet_context: &WalletContext,
        mint: &str,
        amount: f64,
    ) -> Result<YmlStep> {
        // Create step prompt
        let step_prompt = if refined_prompt.refined.to_lowercase().contains("all") {
            format!("lend ALL {mint} to jupiter")
        } else {
            format!("lend {amount} {mint} to jupiter")
        };

        // Create the step
        let step = YmlStep::new(
            Uuid::now_v7().to_string(),
            step_prompt.clone(),
            format!("Deposit {amount} {mint} in Jupiter earn for yield"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_expected_tools(vec![ToolName::JupiterLendEarnDeposit])
        .with_critical(true);

        Ok(step)
    }

    /// Create a ground truth assertion for lend steps
    pub async fn create_ground_truth(
        _wallet_context: &WalletContext,
        mint: &str,
        amount: f64,
    ) -> Result<YmlGroundTruth> {
        // For lending, we primarily verify the tool call was made
        // Balance changes depend on the specific lend protocol and interest rates
        let ground_truth = YmlGroundTruth::new()
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
            .with_error_tolerance(0.01);

        // If we're lending SOL, we can expect a balance change
        // But we can't predict the exact amount due to gas fees
        if mint == "SOL" {
            // Add a generic balance change assertion with wide tolerance
            let ground_truth = ground_truth.with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(_wallet_context.owner.clone())
                    .with_expected_change_lte(-(amount * 1_000_000_000.0 + 10_000_000.0)), // Account for fees
            );
            return Ok(ground_truth);
        }

        Ok(ground_truth)
    }
}

/// Helper function to create wallet info from wallet context
pub fn create_wallet_info(wallet_context: &WalletContext) -> YmlWalletInfo {
    let mut wallet_info =
        YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
            .with_total_value(wallet_context.total_value_usd);

    // Add each token balance to the wallet info
    for token in wallet_context.token_balances.values() {
        wallet_info = wallet_info.with_token(token.clone());
    }

    wallet_info
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refiner::RefinedPrompt;
    // TokenBalance not used in tests, removed import

    #[tokio::test]
    async fn test_swap_step_creation() {
        // Create a mock wallet context
        let mut wallet_context = WalletContext::new("test_wallet".to_string());
        wallet_context.sol_balance = 5_000_000_000; // 5 SOL

        // Create a mock refined prompt
        let refined_prompt = RefinedPrompt::new_for_test(
            "swap 1 SOL to USDC".to_string(),
            "swap 1 SOL to USDC".to_string(),
            false,
        );

        // Create a swap step
        let step =
            SwapStepBuilder::create_step(&refined_prompt, &wallet_context, "SOL", "USDC", 1.0)
                .await
                .unwrap();

        // Verify the step
        assert!(step.prompt.contains("swap 0.95 SOL to USDC")); // Account for gas reserve
        assert_eq!(step.context, "Exchange 0.95 SOL for USDC");
        assert_eq!(step.expected_tools, Some(vec![ToolName::JupiterSwap]));
        assert_eq!(step.critical, Some(true));
    }

    #[tokio::test]
    async fn test_transfer_step_creation() {
        // Create a mock wallet context
        let wallet_context = WalletContext::new("test_wallet".to_string());

        // Create a mock refined prompt
        let refined_prompt = RefinedPrompt::new_for_test(
            "transfer 1 SOL to recipient".to_string(),
            "transfer 1 SOL to recipient".to_string(),
            false,
        );

        // Create a transfer step
        let step = TransferStepBuilder::create_step(
            &refined_prompt,
            &wallet_context,
            "SOL",
            "recipient_address",
            1.0,
        )
        .await
        .unwrap();

        // Verify the step
        assert_eq!(step.prompt, "transfer 1 SOL to recipient_address");
        assert_eq!(
            step.context,
            "Transfer 1 SOL to recipient recipient_address"
        );
        assert_eq!(step.expected_tools, Some(vec![ToolName::SolTransfer]));
        assert_eq!(step.critical, Some(true));
    }

    #[tokio::test]
    async fn test_lend_step_creation() {
        // Create a mock wallet context
        let wallet_context = WalletContext::new("test_wallet".to_string());

        // Create a mock refined prompt
        let refined_prompt = RefinedPrompt::new_for_test(
            "lend 100 USDC".to_string(),
            "lend 100 USDC".to_string(),
            false,
        );

        // Create a lend step
        let step = LendStepBuilder::create_step(&refined_prompt, &wallet_context, "USDC", 100.0)
            .await
            .unwrap();

        // Verify the step
        assert_eq!(step.prompt, "lend 100 USDC to jupiter");
        assert_eq!(step.context, "Deposit 100 USDC in Jupiter earn for yield");
        assert_eq!(
            step.expected_tools,
            Some(vec![ToolName::JupiterLendEarnDeposit])
        );
        assert_eq!(step.critical, Some(true));
    }
}
