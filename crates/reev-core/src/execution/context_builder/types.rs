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

/// Typed representation of key_info for swap operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapKeyInfo {
    /// Input token mint address
    pub input_mint: String,
    /// Output token mint address
    pub output_mint: String,
    /// Amount of input token swapped
    pub input_amount: u64,
    /// Amount of output token received
    pub output_amount: u64,
    /// Amount of output available for lending
    pub output_amount_for_lend: u64,
}

/// Typed representation of key_info for lend operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendKeyInfo {
    /// Asset mint address that was lent
    pub asset_mint: String,
    /// Amount of asset lent
    pub amount: u64,
}

/// Typed representation of key_info for generic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationKeyInfo {
    /// Type of operation
    pub operation_type: String,
    /// Additional operation details
    pub details: serde_json::Value,
}

/// Enum representing different types of key_info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KeyInfo {
    Swap(SwapKeyInfo),
    Lend(LendKeyInfo),
    Operation(OperationKeyInfo),
    Error(ErrorKeyInfo),
}

/// Typed representation of key_info for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorKeyInfo {
    /// Error message
    pub message: String,
    /// Error type (insufficient, slippage, etc.)
    pub error_type: String,
    /// Additional error details
    pub details: serde_json::Value,
}

/// Typed representation of available tokens for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableTokens {
    /// Map of token mint addresses to available amounts
    pub tokens: HashMap<String, u64>,
}

impl AvailableTokens {
    /// Create a new available tokens instance
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Add a token with its available amount
    pub fn add_token(mut self, mint: String, amount: u64) -> Self {
        self.tokens.insert(mint, amount);
        self
    }

    /// Get available amount for a specific token
    pub fn get_amount(&self, mint: &str) -> Option<u64> {
        self.tokens.get(mint).copied()
    }

    /// Convert to HashMap
    pub fn to_hashmap(&self) -> HashMap<String, u64> {
        self.tokens.clone()
    }
}

impl Default for AvailableTokens {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorKeyInfo {
    /// Check if error is of a specific type
    pub fn is_error_type(&self, error_type: &str) -> bool {
        self.error_type == error_type
    }
}

/// Helper methods for working with key_info
impl KeyInfo {
    /// Convert to a display-friendly prompt string
    pub fn to_prompt_string(&self) -> String {
        match self {
            KeyInfo::Swap(swap) => {
                format!(
                    "Key info: Swapped for {} units of {}",
                    swap.output_amount, swap.output_mint
                )
            }
            KeyInfo::Lend(lend) => {
                format!(
                    "Key info: Lent {} units of {}",
                    lend.amount, lend.asset_mint
                )
            }
            KeyInfo::Operation(op) => {
                format!("Key info: Completed operation: {}", op.operation_type)
            }
            KeyInfo::Error(error) => {
                format!("Key info: Error - {}", error.message)
            }
        }
    }

    /// Extract error message from ErrorKeyInfo
    pub fn get_error_message(&self) -> Option<String> {
        match self {
            KeyInfo::Error(error) => Some(error.message.clone()),
            _ => None,
        }
    }
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

/// Enum representing different types of tool results with proper serde deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool_name", content = "result")]
pub enum TypedToolResult {
    /// Jupiter swap operation result
    #[serde(rename = "jupiter_swap")]
    JupiterSwap(JupiterSwapResult),
    /// Jupiter lend operation result
    #[serde(rename = "jupiter_lend")]
    JupiterLend(JupiterLendResult),
    /// Generic operation result for tools without specific typed results
    #[serde(untagged)]
    GenericOperation {
        /// Tool name
        tool_name: String,
        /// Raw result data
        result: serde_json::Value,
        /// Whether the operation succeeded
        success: bool,
        /// Error message if operation failed
        error: Option<String>,
        /// Available tokens for next operations
        available_tokens: Option<AvailableTokens>,
        /// Execution time in milliseconds
        execution_time_ms: Option<u64>,
        /// Additional metadata
        metadata: HashMap<String, serde_json::Value>,
    },
}

/// Struct representing a collection of typed tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedToolResults {
    /// Vector of typed tool results
    pub results: Vec<TypedToolResult>,
}

impl TypedToolResults {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Add a tool result to the collection
    pub fn add_result(&mut self, result: TypedToolResult) {
        self.results.push(result);
    }

    /// Get an iterator over the results
    pub fn iter(&self) -> impl Iterator<Item = &TypedToolResult> {
        self.results.iter()
    }

    /// Get a mutable iterator over the results
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TypedToolResult> {
        self.results.iter_mut()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the number of results
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Extract all key information from all results
    pub fn extract_all_key_info(&self) -> HashMap<String, serde_json::Value> {
        let mut key_info = HashMap::new();
        for result in &self.results {
            let result_key_info = result.extract_key_info();
            key_info.extend(result_key_info);
        }
        key_info
    }

    /// Extract all balance changes from all results
    pub fn extract_all_balance_changes(
        &self,
    ) -> Vec<crate::execution::context_builder::BalanceChange> {
        let mut balance_changes = Vec::new();
        for result in &self.results {
            let result_balance_changes = result.extract_balance_changes();
            balance_changes.extend(result_balance_changes);
        }
        balance_changes
    }

    /// Extract all constraints from all results
    pub fn extract_all_constraints(&self) -> Vec<String> {
        let mut constraints = Vec::new();
        for result in &self.results {
            let result_constraints = result.extract_next_step_constraints();
            constraints.extend(result_constraints);
        }
        constraints
    }

    /// Extract all available tokens from all results
    pub fn extract_all_available_tokens(&self) -> HashMap<String, u64> {
        let mut available_tokens = HashMap::new();
        for result in &self.results {
            let result_tokens = result.extract_available_tokens();
            available_tokens.extend(result_tokens);
        }
        available_tokens
    }
}

impl Default for TypedToolResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic wrapper for tool results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultWrapper {
    /// The typed result of the tool
    #[serde(flatten)]
    pub result: TypedToolResult,
}

impl ToolResultWrapper {
    /// Create a new wrapper from a Jupiter swap result
    pub fn from_jupiter_swap(result: JupiterSwapResult) -> Self {
        Self {
            result: TypedToolResult::JupiterSwap(result),
        }
    }

    /// Create a new wrapper from a Jupiter lend result
    pub fn from_jupiter_lend(result: JupiterLendResult) -> Self {
        Self {
            result: TypedToolResult::JupiterLend(result),
        }
    }

    /// Create a new wrapper from a generic operation result
    pub fn from_generic_operation(
        tool_name: String,
        result: serde_json::Value,
        success: bool,
        error: Option<String>,
        execution_time_ms: Option<u64>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            result: TypedToolResult::GenericOperation {
                tool_name,
                result,
                success,
                error,
                available_tokens: None,
                execution_time_ms,
                metadata,
            },
        }
    }

    /// Create a new wrapper from a generic operation result with available tokens
    pub fn from_generic_operation_with_tokens(
        tool_name: String,
        result: serde_json::Value,
        success: bool,
        error: Option<String>,
        available_tokens: AvailableTokens,
        execution_time_ms: Option<u64>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            result: TypedToolResult::GenericOperation {
                tool_name,
                result,
                success,
                error,
                available_tokens: Some(available_tokens),
                execution_time_ms,
                metadata,
            },
        }
    }

    /// Get the tool name from the wrapped result
    pub fn get_tool_name(&self) -> String {
        match &self.result {
            TypedToolResult::JupiterSwap(_) => "jupiter_swap".to_string(),
            TypedToolResult::JupiterLend(_) => "jupiter_lend".to_string(),
            TypedToolResult::GenericOperation { tool_name, .. } => tool_name.clone(),
        }
    }

    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        match &self.result {
            TypedToolResult::JupiterSwap(result) => result.completed,
            TypedToolResult::JupiterLend(result) => result.completed,
            TypedToolResult::GenericOperation { success, .. } => *success,
        }
    }

    /// Get the error message if the operation failed
    pub fn get_error(&self) -> Option<String> {
        match &self.result {
            TypedToolResult::JupiterSwap(_) | TypedToolResult::JupiterLend(_) => None,
            TypedToolResult::GenericOperation { error, .. } => error.clone(),
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

/// Implement ExtractKeyInfo for TypedToolResult to handle all tool types uniformly
impl ExtractKeyInfo for TypedToolResult {
    fn extract_key_info(&self) -> HashMap<String, serde_json::Value> {
        match self {
            TypedToolResult::JupiterSwap(swap_result) => swap_result.extract_key_info(),
            TypedToolResult::JupiterLend(lend_result) => lend_result.extract_key_info(),
            TypedToolResult::GenericOperation {
                tool_name, result, ..
            } => {
                // Create typed OperationKeyInfo for generic operations
                let op_info = OperationKeyInfo {
                    operation_type: tool_name.clone(),
                    details: result.clone(),
                };

                let mut key_info = HashMap::new();
                key_info.insert(
                    "operation".to_string(),
                    serde_json::to_value(KeyInfo::Operation(op_info)).unwrap(),
                );
                key_info
            }
        }
    }

    fn extract_balance_changes(&self) -> Vec<crate::execution::context_builder::BalanceChange> {
        match self {
            TypedToolResult::JupiterSwap(swap_result) => swap_result.extract_balance_changes(),
            TypedToolResult::JupiterLend(lend_result) => lend_result.extract_balance_changes(),
            TypedToolResult::GenericOperation { .. } => Vec::new(), // No balance changes for generic operations
        }
    }

    fn extract_next_step_constraints(&self) -> Vec<String> {
        match self {
            TypedToolResult::JupiterSwap(swap_result) => {
                swap_result.extract_next_step_constraints()
            }
            TypedToolResult::JupiterLend(lend_result) => {
                lend_result.extract_next_step_constraints()
            }
            TypedToolResult::GenericOperation { result, .. } => {
                // Extract any constraints from the generic operation result
                result
                    .get("constraints")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default()
            }
        }
    }

    fn extract_available_tokens(&self) -> HashMap<String, u64> {
        match self {
            TypedToolResult::JupiterSwap(swap_result) => swap_result.extract_available_tokens(),
            TypedToolResult::JupiterLend(lend_result) => lend_result.extract_available_tokens(),
            TypedToolResult::GenericOperation {
                available_tokens, ..
            } => {
                // Extract available tokens from the structured type
                available_tokens
                    .as_ref()
                    .map(|tokens| tokens.to_hashmap())
                    .unwrap_or_default()
            }
        }
    }
}

impl ExtractKeyInfo for JupiterSwapResult {
    fn extract_key_info(&self) -> HashMap<String, serde_json::Value> {
        let mut key_info = HashMap::new();

        // Create typed SwapKeyInfo
        let swap_info = SwapKeyInfo {
            input_mint: self.input_mint.clone(),
            output_mint: self.output_mint.clone(),
            input_amount: self.input_amount,
            output_amount: self.output_amount,
            output_amount_for_lend: self.output_amount,
        };

        // Store as KeyInfo enum
        key_info.insert(
            "swap".to_string(),
            serde_json::to_value(KeyInfo::Swap(swap_info)).unwrap(),
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

        // Create typed LendKeyInfo
        let lend_info = LendKeyInfo {
            asset_mint: self.asset_mint.clone(),
            amount: self.amount,
        };

        // Store as KeyInfo enum
        key_info.insert(
            "lend".to_string(),
            serde_json::to_value(KeyInfo::Lend(lend_info)).unwrap(),
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
