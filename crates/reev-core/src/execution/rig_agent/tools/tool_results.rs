//! Tool Result Types
//!
//! This module contains typed result structs for each tool implementation
//! to replace the untyped serde_json::Value returns.

use crate::execution::context_builder::ToolResultWrapper;
use serde::{Deserialize, Serialize};

/// Result for SOL transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolTransferResult {
    /// Tool name identifier
    pub tool_name: String,
    /// Recipient address
    pub recipient: String,
    /// Amount transferred in SOL (human-readable)
    pub amount: f64,
    /// Amount transferred in lamports
    pub amount_lamports: u64,
    /// Wallet address that sent the transfer
    pub wallet: String,
    /// Transaction signature
    pub transaction_signature: Option<String>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Result for SPL token transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplTransferResult {
    /// Tool name identifier
    pub tool_name: String,
    /// Recipient address
    pub recipient: String,
    /// Amount transferred (human-readable)
    pub amount: String,
    /// Mint address of the token
    pub mint_address: String,
    /// Token mint pubkey
    pub token_mint: String,
    /// Wallet address that sent the transfer
    pub wallet: String,
    /// Transaction signature
    pub transaction_signature: Option<String>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Result for Jupiter swap operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapResult {
    /// Tool name identifier
    pub tool_name: String,
    /// Input token mint address
    pub input_mint: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount of input token in lamports
    pub amount: u64,
    /// Wallet address that performed the swap
    pub wallet: String,
    /// Transaction signature
    pub transaction_signature: Option<String>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Result for Jupiter lend/earn operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLendResult {
    /// Tool name identifier
    pub tool_name: String,
    /// Token mint address
    pub mint: String,
    /// Amount in the smallest denomination (lamports)
    pub amount: u64,
    /// Wallet address that performed the operation
    pub wallet: String,
    /// Transaction signature
    pub transaction_signature: Option<String>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Result for account balance queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceResult {
    /// Tool name identifier
    pub tool_name: String,
    /// Account address queried
    pub account: String,
    /// Token mint address (default is SOL)
    pub mint: String,
    /// Balance in the smallest denomination
    pub balance: u64,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Enum to represent all possible tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool_name")]
pub enum ToolResult {
    /// SOL transfer result
    #[serde(rename = "sol_transfer")]
    SolTransfer(SolTransferResult),
    /// SPL transfer result
    #[serde(rename = "spl_transfer")]
    SplTransfer(SplTransferResult),
    /// Jupiter swap result
    #[serde(rename = "jupiter_swap")]
    JupiterSwap(JupiterSwapResult),
    /// Jupiter lend/earn result
    #[serde(rename = "jupiter_lend_earn_deposit")]
    JupiterLend(JupiterLendResult),
    /// Account balance result
    #[serde(rename = "get_account_balance")]
    AccountBalance(AccountBalanceResult),
}

impl ToolResult {
    /// Get the tool name as a string
    pub fn tool_name(&self) -> &str {
        match self {
            ToolResult::SolTransfer(_) => "sol_transfer",
            ToolResult::SplTransfer(_) => "spl_transfer",
            ToolResult::JupiterSwap(_) => "jupiter_swap",
            ToolResult::JupiterLend(_) => "jupiter_lend_earn_deposit",
            ToolResult::AccountBalance(_) => "get_account_balance",
        }
    }

    /// Check if the operation was successful
    pub fn success(&self) -> bool {
        match self {
            ToolResult::SolTransfer(r) => r.success,
            ToolResult::SplTransfer(r) => r.success,
            ToolResult::JupiterSwap(r) => r.success,
            ToolResult::JupiterLend(r) => r.success,
            ToolResult::AccountBalance(r) => r.success,
        }
    }

    /// Get the error message if any
    pub fn error(&self) -> Option<&String> {
        match self {
            ToolResult::SolTransfer(r) => r.error.as_ref(),
            ToolResult::SplTransfer(r) => r.error.as_ref(),
            ToolResult::JupiterSwap(r) => r.error.as_ref(),
            ToolResult::JupiterLend(r) => r.error.as_ref(),
            ToolResult::AccountBalance(r) => r.error.as_ref(),
        }
    }

    /// Get the transaction signature if any
    pub fn transaction_signature(&self) -> Option<&String> {
        match self {
            ToolResult::SolTransfer(r) => r.transaction_signature.as_ref(),
            ToolResult::SplTransfer(r) => r.transaction_signature.as_ref(),
            ToolResult::JupiterSwap(r) => r.transaction_signature.as_ref(),
            ToolResult::JupiterLend(r) => r.transaction_signature.as_ref(),
            ToolResult::AccountBalance(_) => None,
        }
    }
}

/// Collection of tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResults {
    /// Vector of tool results
    pub results: Vec<ToolResult>,
}

impl ToolResults {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Add a result to the collection
    #[allow(dead_code)]
    pub fn add_result(&mut self, result: ToolResult) {
        self.results.push(result);
    }

    /// Get all successful results
    #[allow(dead_code)]
    pub fn successful_results(&self) -> Vec<&ToolResult> {
        self.results.iter().filter(|r| r.success()).collect()
    }

    /// Get all failed results
    #[allow(dead_code)]
    pub fn failed_results(&self) -> Vec<&ToolResult> {
        self.results.iter().filter(|r| !r.success()).collect()
    }

    /// Check if all operations were successful
    #[allow(dead_code)]
    pub fn all_successful(&self) -> bool {
        self.results.iter().all(|r| r.success())
    }
}

impl Default for ToolResults {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ToolResult> for ToolResultWrapper {
    fn from(result: ToolResult) -> Self {
        // Create a generic tool result wrapper with operation info
        let operation_info = serde_json::json!({
            "tool_name": result.tool_name(),
            "success": result.success(),
            "error": result.error(),
            "transaction_signature": result.transaction_signature(),
            "metadata": serde_json::to_value(&result).unwrap_or_default()
        });

        // Create wrapper with minimal required fields
        ToolResultWrapper {
            result: crate::execution::context_builder::TypedToolResult::GenericOperation {
                tool_name: result.tool_name().to_string(),
                result: operation_info,
                success: result.success(),
                error: result.error().cloned(),
                available_tokens: None,
                execution_time_ms: None,
                metadata: std::collections::HashMap::new(),
            },
        }
    }
}
