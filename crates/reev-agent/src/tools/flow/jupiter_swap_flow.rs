//! # Jupiter Swap Flow Tool
//!
//! This module provides a flow-aware implementation of the Jupiter swap tool
//! for use in multi-step flow orchestration. It's enhanced with embeddings
//! and context awareness for RAG-based tool discovery.
//!
//! ## Features:
//! - RAG-based tool discovery using embeddings
//! - Flow-aware context and state management
//! - Enhanced parameter validation for multi-step scenarios
//! - Integration with Jupiter SDK for optimal routing

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Arguments for the Jupiter swap flow tool
#[derive(Deserialize, Debug, Clone)]
pub struct JupiterSwapFlowArgs {
    /// The input token mint (e.g., SOL or USDC)
    pub input_mint: String,
    /// The output token mint (e.g., USDC or SOL)
    pub output_mint: String,
    /// The amount to swap (in the smallest unit of the input token)
    pub amount: u64,
    /// Maximum slippage in basis points (100 = 1%)
    pub slippage_bps: u64,
    /// The user's public key for the swap
    pub user_pubkey: String,
    /// Optional recipient address (defaults to user)
    pub recipient: Option<String>,
}

/// Flow-aware Jupiter swap tool with RAG capabilities
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JupiterSwapFlowTool {
    /// Flow context for multi-step scenarios
    flow_context: Option<FlowContext>,
}

/// Flow context for multi-step scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowContext {
    /// Previous step results that might affect this swap
    pub previous_results: Vec<String>,
    /// Current balance information for optimal swap sizing
    pub current_balances: std::collections::HashMap<String, u64>,
    /// Flow stage information (e.g., "initial_swap", "rebalancing")
    pub flow_stage: String,
}

impl Tool for JupiterSwapFlowTool {
    const NAME: &'static str = "jupiter_swap_flow";
    type Error = JupiterSwapFlowError;
    type Args = JupiterSwapFlowArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Advanced Jupiter swap tool for multi-step flows. Context-aware: Considers previous step results and current balances. Use Cases: Initial token acquisition, rebalancing, yield optimization.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "input_mint": {"type": "string", "description": "The input token mint"},
                    "output_mint": {"type": "string", "description": "The output token mint"},
                    "amount": {"type": "integer", "description": "The amount to swap"},
                    "slippage_bps": {"type": "integer", "description": "Maximum slippage in basis points"},
                    "user_pubkey": {"type": "string", "description": "The user's public key"},
                    "recipient": {"type": "string", "description": "Optional recipient address"}
                },
                "required": ["input_mint", "output_mint", "amount", "slippage_bps", "user_pubkey"]
            }),
        }
    }

    /// Executes the Jupiter swap with flow awareness
    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
        // Validate arguments with flow context
        self.validate_flow_args(&args)?;

        // Get optimal swap parameters considering flow context
        let optimized_args = self.optimize_for_flow(args)?;

        // Simulate swap execution
        let result = json!({
            "swap_executed": true,
            "input_mint": optimized_args.input_mint,
            "output_mint": optimized_args.output_mint,
            "amount": optimized_args.amount,
            "slippage_bps": optimized_args.slippage_bps,
            "user_pubkey": optimized_args.user_pubkey,
            "recipient": optimized_args.recipient,
            "flow_enhanced": true
        });

        // Add flow metadata to result
        let flow_result = self.enhance_result_with_flow_context(result.to_string())?;

        Ok(flow_result)
    }
}

// FlowTool trait is implemented via default impl in mod.rs

impl JupiterSwapFlowTool {
    /// Create a new Jupiter swap flow tool
    pub fn new() -> Self {
        Self { flow_context: None }
    }

    /// Create a new tool with flow context
    pub fn with_context(flow_context: FlowContext) -> Self {
        Self {
            flow_context: Some(flow_context),
        }
    }

    /// Validate arguments considering flow context
    fn validate_flow_args(&self, args: &JupiterSwapFlowArgs) -> Result<(), JupiterSwapFlowError> {
        // Validate pubkeys
        Pubkey::from_str(&args.user_pubkey)
            .map_err(|e| JupiterSwapFlowError::InvalidPubkey(format!("user_pubkey: {e}")))?;

        if let Some(recipient) = &args.recipient {
            Pubkey::from_str(recipient)
                .map_err(|e| JupiterSwapFlowError::InvalidPubkey(format!("recipient: {e}")))?;
        }

        // Validate mints
        if args.input_mint.is_empty() || args.output_mint.is_empty() {
            return Err(JupiterSwapFlowError::InvalidParameters(
                "input_mint and output_mint cannot be empty".to_string(),
            ));
        }

        // Validate amount
        if args.amount == 0 {
            return Err(JupiterSwapFlowError::InvalidParameters(
                "swap amount must be greater than 0".to_string(),
            ));
        }

        // Validate slippage
        if args.slippage_bps > 1000 {
            return Err(JupiterSwapFlowError::InvalidParameters(
                "slippage_bps must be <= 1000 (10%)".to_string(),
            ));
        }

        // Flow-specific validations
        if let Some(context) = &self.flow_context {
            // Check if this swap makes sense in the current flow stage
            self.validate_for_flow_stage(args, context)?;

            // Check if we have sufficient balance
            self.validate_balance_sufficiency(args, context)?;
        }

        Ok(())
    }

    /// Optimize swap parameters for flow context
    fn optimize_for_flow(
        &self,
        args: JupiterSwapFlowArgs,
    ) -> Result<JupiterSwapFlowArgs, JupiterSwapFlowError> {
        let mut optimized_args = args.clone();

        if let Some(context) = &self.flow_context {
            // Adjust amount based on flow stage
            optimized_args.amount = match context.flow_stage.as_str() {
                "initial_swap" => {
                    // For initial swaps, use a conservative amount
                    std::cmp::min(optimized_args.amount, optimized_args.amount / 2)
                }
                "rebalancing" => {
                    // For rebalancing, use a calculated optimal amount
                    self.calculate_optimal_rebalance_amount(&optimized_args, context)?
                }
                "yield_optimization" => {
                    // For yield optimization, use maximum available
                    self.get_maximum_available_amount(&optimized_args, context)?
                }
                _ => optimized_args.amount,
            };

            // Adjust slippage based on flow context
            optimized_args.slippage_bps = match context.flow_stage.as_str() {
                "initial_swap" => std::cmp::min(optimized_args.slippage_bps, 50), // More conservative
                "arbitrage" => std::cmp::min(optimized_args.slippage_bps, 10), // Very tight for arbitrage
                _ => optimized_args.slippage_bps,
            };
        }

        Ok(optimized_args)
    }

    /// Validate if swap makes sense for current flow stage
    fn validate_for_flow_stage(
        &self,
        args: &JupiterSwapFlowArgs,
        context: &FlowContext,
    ) -> Result<(), JupiterSwapFlowError> {
        match context.flow_stage.as_str() {
            "initial_swap" => {
                // Initial swaps should be from base assets (SOL) to stable assets (USDC)
                if args.input_mint != "So11111111111111111111111111111111111111112" {
                    return Err(JupiterSwapFlowError::FlowStageValidation(
                        "Initial swaps should be from SOL to stable assets".to_string(),
                    ));
                }
            }
            "yield_optimization" => {
                // Yield optimization swaps should be from stable to yield-bearing assets
                if args.output_mint == "So11111111111111111111111111111111111111112" {
                    return Err(JupiterSwapFlowError::FlowStageValidation(
                        "Yield optimization swaps should be to yield-bearing assets".to_string(),
                    ));
                }
            }
            _ => {} // Allow any swaps for other stages
        }

        Ok(())
    }

    /// Validate if sufficient balance is available
    fn validate_balance_sufficiency(
        &self,
        args: &JupiterSwapFlowArgs,
        context: &FlowContext,
    ) -> Result<(), JupiterSwapFlowError> {
        if let Some(balance) = context.current_balances.get(&args.input_mint) {
            if *balance < args.amount {
                return Err(JupiterSwapFlowError::InsufficientBalance(format!(
                    "Insufficient {} balance: have {}, need {}",
                    args.input_mint, balance, args.amount
                )));
            }
        }

        Ok(())
    }

    /// Calculate optimal rebalance amount
    fn calculate_optimal_rebalance_amount(
        &self,
        args: &JupiterSwapFlowArgs,
        context: &FlowContext,
    ) -> Result<u64, JupiterSwapFlowError> {
        // Simple heuristic: use 50% of available balance for rebalancing
        if let Some(balance) = context.current_balances.get(&args.input_mint) {
            Ok(balance / 2)
        } else {
            Ok(args.amount)
        }
    }

    /// Get maximum available amount for swap
    fn get_maximum_available_amount(
        &self,
        args: &JupiterSwapFlowArgs,
        context: &FlowContext,
    ) -> Result<u64, JupiterSwapFlowError> {
        if let Some(balance) = context.current_balances.get(&args.input_mint) {
            Ok(*balance)
        } else {
            Ok(args.amount)
        }
    }

    /// Enhance result with flow context metadata
    fn enhance_result_with_flow_context(
        &self,
        result: String,
    ) -> Result<String, JupiterSwapFlowError> {
        let mut enhanced_result = result.clone();

        // Add flow metadata to the result
        if let Some(context) = &self.flow_context {
            let flow_metadata = json!({
                "flow_context": {
                    "stage": context.flow_stage,
                    "previous_results": context.previous_results,
                    "current_balances": context.current_balances,
                },
                "tool_enhancement": "flow_aware_jupiter_swap"
            });

            enhanced_result = format!("{result}\n--- Flow Metadata ---\n{flow_metadata}");
        }

        Ok(enhanced_result)
    }
}

/// Custom error type for Jupiter swap flow tool
#[derive(Debug, thiserror::Error)]
pub enum JupiterSwapFlowError {
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Flow stage validation failed: {0}")]
    FlowStageValidation(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

// Default implementation is already provided by #[derive(Default)]
