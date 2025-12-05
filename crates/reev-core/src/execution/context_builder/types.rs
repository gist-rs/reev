//! Typed structs for tool results used in context builder

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Typed representation of Jupiter swap tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapResult {
    /// Input token mint address
    pub input_mint: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount of input token swapped
    pub input_amount: u64,
    /// Amount of output token received
    pub output_amount: u64,
    /// Slippage tolerance in basis points
    pub slippage_bps: Option<u16>,
    /// Number of instructions generated
    pub instruction_count: Option<usize>,
    /// Operation type identifier
    pub operation_type: String,
    /// Current status of operation
    pub status: String,
    /// Whether operation is completed
    pub completed: bool,
    /// Optional transaction signature
    pub transaction_signature: Option<String>,
    /// Message describing the result
    pub message: String,
}

/// Typed representation of Jupiter lend tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLendResult {
    /// Asset mint address that was lent
    pub asset_mint: String,
    /// Amount of asset lent
    pub amount: u64,
    /// Lending protocol used
    pub protocol: Option<String>,
    /// Operation type identifier
    pub operation_type: String,
    /// Current status of operation
    pub status: String,
    /// Whether operation is completed
    pub completed: bool,
    /// Optional transaction signature
    pub transaction_signature: Option<String>,
    /// Message describing the result
    pub message: String,
}

/// Generic wrapper for tool results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultWrapper {
    /// Name of the tool that generated the result
    pub tool_name: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Raw result data (will be deserialized to specific types)
    pub data: serde_json::Value,
    /// Error message if operation failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Enum representing different types of tool results
#[derive(Debug, Clone)]
pub enum TypedToolResult {
    JupiterSwap(JupiterSwapResult),
    JupiterLend(JupiterLendResult),
    // Add other tool result types as needed
    Other(String, serde_json::Value), // tool_name, raw_value
}

impl ToolResultWrapper {
    /// Try to deserialize into a specific tool result type
    pub fn into_typed_result(self) -> TypedToolResult {
        match self.tool_name.as_str() {
            "jupiter_swap" => {
                // Clone data to avoid moving
                let data_clone = self.data.clone();
                if let Ok(swap_result) = serde_json::from_value::<JupiterSwapResult>(data_clone) {
                    TypedToolResult::JupiterSwap(swap_result)
                } else {
                    TypedToolResult::Other(self.tool_name, self.data)
                }
            }
            "jupiter_lend" => {
                // Clone data to avoid moving
                let data_clone = self.data.clone();
                if let Ok(lend_result) = serde_json::from_value::<JupiterLendResult>(data_clone) {
                    TypedToolResult::JupiterLend(lend_result)
                } else {
                    TypedToolResult::Other(self.tool_name, self.data)
                }
            }
            _ => TypedToolResult::Other(self.tool_name, self.data),
        }
    }
}

/// Extract key information from different tool result types
pub trait ExtractKeyInfo {
    /// Extract key information for context builder
    fn extract_key_info(&self) -> HashMap<String, serde_json::Value>;
    /// Extract balance changes for context builder
    fn extract_balance_changes(&self) -> Vec<crate::execution::context_builder::BalanceChange>;
    /// Extract constraints for next step
    fn extract_next_step_constraints(&self) -> Vec<String>;
    /// Extract available tokens for next step
    fn extract_available_tokens(&self) -> HashMap<String, u64>;
}

impl ExtractKeyInfo for JupiterSwapResult {
    fn extract_key_info(&self) -> HashMap<String, serde_json::Value> {
        let mut key_info = HashMap::new();
        key_info.insert(
            "swap".to_string(),
            serde_json::json!({
                "input_mint": self.input_mint,
                "output_mint": self.output_mint,
                "input_amount": self.input_amount,
                "output_amount": self.output_amount,
                "output_amount_for_lend": self.output_amount,
            }),
        );
        key_info
    }

    fn extract_balance_changes(&self) -> Vec<crate::execution::context_builder::BalanceChange> {
        // Note: In a real implementation, we would need access to wallet_context
        // For now, this is a placeholder that would be adjusted in the context builder
        vec![
            crate::execution::context_builder::BalanceChange {
                mint: self.input_mint.clone(),
                balance_before: self.input_amount,
                balance_after: 0,
                change_amount: -(self.input_amount as i64),
                symbol: None,
            },
            crate::execution::context_builder::BalanceChange {
                mint: self.output_mint.clone(),
                balance_before: 0,
                balance_after: self.output_amount,
                change_amount: self.output_amount as i64,
                symbol: None,
            },
        ]
    }

    fn extract_next_step_constraints(&self) -> Vec<String> {
        vec![format!(
            "Use exactly {} units of {} from previous swap",
            self.output_amount, self.output_mint
        )]
    }

    fn extract_available_tokens(&self) -> HashMap<String, u64> {
        let mut available_tokens = HashMap::new();
        available_tokens.insert(self.output_mint.clone(), self.output_amount);
        available_tokens
    }
}

impl ExtractKeyInfo for JupiterLendResult {
    fn extract_key_info(&self) -> HashMap<String, serde_json::Value> {
        let mut key_info = HashMap::new();
        key_info.insert(
            "lend".to_string(),
            serde_json::json!({
                "asset_mint": self.asset_mint,
                "amount": self.amount,
            }),
        );
        key_info
    }

    fn extract_balance_changes(&self) -> Vec<crate::execution::context_builder::BalanceChange> {
        vec![crate::execution::context_builder::BalanceChange {
            mint: self.asset_mint.clone(),
            balance_before: self.amount,
            balance_after: 0,
            change_amount: -(self.amount as i64),
            symbol: None,
        }]
    }

    fn extract_next_step_constraints(&self) -> Vec<String> {
        vec![format!(
            "{} units of {} are now lent and unavailable",
            self.amount, self.asset_mint
        )]
    }

    fn extract_available_tokens(&self) -> HashMap<String, u64> {
        HashMap::new() // No tokens available after lending
    }
}
