//! Enhanced Operation History Tracking for RigAgent
//!
//! This module implements comprehensive operation history tracking
//! to improve context passing between operations in multi-step flows.

// Result is used in function signatures but not directly
use chrono::{DateTime, Utc};
use reev_types::flow::StepResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Enhanced operation history entry with detailed input/output tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationHistory {
    /// Type of operation (swap, lend, transfer, etc.)
    pub operation_type: String,
    /// Input amount if applicable
    pub input_amount: Option<f64>,
    /// Input mint address if applicable
    pub input_mint: Option<String>,
    /// Output amount if applicable
    pub output_amount: Option<f64>,
    /// Output mint address if applicable
    pub output_mint: Option<String>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if operation failed
    pub error_message: Option<String>,
    /// Timestamp when operation was executed
    pub timestamp: DateTime<Utc>,
    /// Additional metadata about the operation
    pub metadata: HashMap<String, Value>,
}

impl OperationHistory {
    /// Create a new operation history entry from a step result
    pub fn from_step_result(step_result: &StepResult) -> Vec<Self> {
        let mut history_entries = Vec::new();
        let timestamp = Utc::now();

        if step_result.success {
            // Extract operation information from successful step
            if let Some(tool_results) = step_result.output.get("tool_results") {
                if let Some(results_array) = tool_results.as_array() {
                    for tool_result in results_array {
                        // Handle swap operations
                        if let Some(swap_info) = tool_result.get("jupiter_swap") {
                            let operation = Self {
                                operation_type: "swap".to_string(),
                                input_amount: swap_info
                                    .get("input_amount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as f64),
                                input_mint: swap_info
                                    .get("input_mint")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                output_amount: swap_info
                                    .get("output_amount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as f64),
                                output_mint: swap_info
                                    .get("output_mint")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                success: true,
                                error_message: None,
                                timestamp,
                                metadata: {
                                    let mut metadata = HashMap::new();
                                    metadata.insert("tool_result".to_string(), tool_result.clone());
                                    metadata
                                },
                            };
                            history_entries.push(operation);
                        }
                        // Handle lend operations
                        else if let Some(lend_info) = tool_result.get("jupiter_lend") {
                            let operation = Self {
                                operation_type: "lend".to_string(),
                                input_amount: lend_info
                                    .get("amount")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as f64),
                                input_mint: lend_info
                                    .get("asset_mint")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                output_amount: None,
                                output_mint: None,
                                success: true,
                                error_message: None,
                                timestamp,
                                metadata: {
                                    let mut metadata = HashMap::new();
                                    metadata.insert("tool_result".to_string(), tool_result.clone());
                                    metadata
                                },
                            };
                            history_entries.push(operation);
                        }
                        // Handle generic operations
                        else if let Some(operation_type) =
                            tool_result.get("operation_type").and_then(|v| v.as_str())
                        {
                            let operation = Self {
                                operation_type: operation_type.to_string(),
                                input_amount: None,
                                input_mint: None,
                                output_amount: None,
                                output_mint: None,
                                success: true,
                                error_message: None,
                                timestamp,
                                metadata: {
                                    let mut metadata = HashMap::new();
                                    metadata.insert("tool_result".to_string(), tool_result.clone());
                                    metadata
                                },
                            };
                            history_entries.push(operation);
                        }
                    }
                }
            }
        } else {
            // Create entry for failed operation
            let operation = Self {
                operation_type: "unknown".to_string(),
                input_amount: None,
                input_mint: None,
                output_amount: None,
                output_mint: None,
                success: false,
                error_message: step_result.error_message.clone(),
                timestamp,
                metadata: {
                    let mut metadata = HashMap::new();
                    let tool_names = Value::Array(
                        step_result
                            .tool_calls
                            .iter()
                            .map(|name| Value::String(name.clone()))
                            .collect(),
                    );
                    metadata.insert("attempted_tools".to_string(), tool_names);
                    metadata
                },
            };
            history_entries.push(operation);
        }

        history_entries
    }

    /// Get the net balance change for a specific mint
    pub fn get_balance_change_for_mint(&self, mint: &str) -> f64 {
        let mut change = 0.0;

        // Subtract input amount if it matches the mint
        if let (Some(input_mint), Some(input_amount)) = (&self.input_mint, self.input_amount) {
            if input_mint == mint {
                change -= input_amount;
            }
        }

        // Add output amount if it matches the mint
        if let (Some(output_mint), Some(output_amount)) = (&self.output_mint, self.output_amount) {
            if output_mint == mint {
                change += output_amount;
            }
        }

        change
    }

    /// Get a summary of the operation
    pub fn get_summary(&self) -> String {
        if self.success {
            match self.operation_type.as_str() {
                "swap" => {
                    if let (
                        Some(input_amount),
                        Some(input_mint),
                        Some(output_amount),
                        Some(output_mint),
                    ) = (
                        self.input_amount,
                        &self.input_mint,
                        self.output_amount,
                        &self.output_mint,
                    ) {
                        format!(
                            "Swapped {input_amount} of {input_mint} for {output_amount} of {output_mint}"
                        )
                    } else {
                        "Swap operation completed".to_string()
                    }
                }
                "lend" => {
                    if let (Some(amount), Some(mint)) = (self.input_amount, &self.input_mint) {
                        format!("Lent {amount} of {mint}")
                    } else {
                        "Lend operation completed".to_string()
                    }
                }
                _ => format!("{} operation completed", self.operation_type),
            }
        } else {
            format!(
                "{} operation failed: {}",
                self.operation_type,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

/// Calculator for available balances after considering operation history
pub struct BalanceCalculator {
    initial_balances: HashMap<String, f64>,
    operation_history: Vec<OperationHistory>,
}

impl BalanceCalculator {
    /// Create a new balance calculator with initial balances
    pub fn new(initial_balances: HashMap<String, f64>) -> Self {
        Self {
            initial_balances,
            operation_history: Vec::new(),
        }
    }

    /// Add operation history entries
    pub fn add_operations(&mut self, operations: Vec<OperationHistory>) -> &mut Self {
        self.operation_history.extend(operations);
        self
    }

    /// Calculate available balance for a specific mint
    pub fn calculate_available_balance(&self, mint: &str) -> f64 {
        let mut balance = self.initial_balances.get(mint).copied().unwrap_or(0.0);

        // Apply balance changes from operation history
        for operation in &self.operation_history {
            if operation.success {
                balance += operation.get_balance_change_for_mint(mint);
            }
        }

        balance
    }

    /// Get all available balances
    pub fn calculate_all_balances(&self) -> HashMap<String, f64> {
        let mut balances = self.initial_balances.clone();

        // Apply balance changes from operation history
        for operation in &self.operation_history {
            if operation.success {
                for (mint, initial_balance) in &mut balances {
                    *initial_balance += operation.get_balance_change_for_mint(mint);
                }

                // Add new mints that weren't in the initial balances
                if let Some(output_mint) = &operation.output_mint {
                    if !balances.contains_key(output_mint) {
                        balances.insert(
                            output_mint.clone(),
                            operation.get_balance_change_for_mint(output_mint),
                        );
                    }
                }
            }
        }

        balances
    }

    /// Get a summary of balance changes
    pub fn get_balance_changes_summary(&self) -> HashMap<String, f64> {
        let mut changes = HashMap::new();

        for operation in &self.operation_history {
            if operation.success {
                // Track input changes
                if let (Some(input_mint), Some(input_amount)) =
                    (&operation.input_mint, operation.input_amount)
                {
                    let entry = changes.entry(input_mint.clone()).or_insert(0.0);
                    *entry -= input_amount;
                }

                // Track output changes
                if let (Some(output_mint), Some(output_amount)) =
                    (&operation.output_mint, operation.output_amount)
                {
                    let entry = changes.entry(output_mint.clone()).or_insert(0.0);
                    *entry += output_amount;
                }
            }
        }

        changes
    }
}

/// Builder for operation history tracking
#[derive(Clone)]
pub struct OperationHistoryBuilder {
    history: Vec<OperationHistory>,
}

impl OperationHistoryBuilder {
    /// Create a new operation history builder
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Add step results to the history
    pub fn add_step_results(&mut self, step_results: &[StepResult]) -> &mut Self {
        for step_result in step_results {
            let operations = OperationHistory::from_step_result(step_result);
            self.history.extend(operations);
        }
        self
    }

    /// Add a single operation to the history
    pub fn add_operation(&mut self, operation: OperationHistory) -> &mut Self {
        self.history.push(operation);
        self
    }

    /// Build the operation history
    pub fn build(self) -> Vec<OperationHistory> {
        self.history
    }

    /// Create a balance calculator from this history
    pub fn create_balance_calculator(
        &self,
        initial_balances: HashMap<String, f64>,
    ) -> BalanceCalculator {
        let mut calculator = BalanceCalculator::new(initial_balances);
        calculator.add_operations(self.history.clone());
        calculator
    }
}

impl Default for OperationHistoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TokenBalance is not used directly in tests

    #[test]
    fn test_swap_operation_history() {
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), Value::String("value".to_string()));

        let swap_operation = OperationHistory {
            operation_type: "swap".to_string(),
            input_amount: Some(100.0),
            input_mint: Some("mintA".to_string()),
            output_amount: Some(50.0),
            output_mint: Some("mintB".to_string()),
            success: true,
            error_message: None,
            timestamp: Utc::now(),
            metadata,
        };

        assert_eq!(swap_operation.get_balance_change_for_mint("mintA"), -100.0);
        assert_eq!(swap_operation.get_balance_change_for_mint("mintB"), 50.0);
        assert_eq!(swap_operation.get_balance_change_for_mint("mintC"), 0.0);
    }

    #[test]
    fn test_balance_calculator() {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("mintA".to_string(), 200.0);
        initial_balances.insert("mintB".to_string(), 100.0);

        let swap_operation = OperationHistory {
            operation_type: "swap".to_string(),
            input_amount: Some(100.0),
            input_mint: Some("mintA".to_string()),
            output_amount: Some(50.0),
            output_mint: Some("mintB".to_string()),
            success: true,
            error_message: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let mut calculator = BalanceCalculator::new(initial_balances);
        calculator.add_operations(vec![swap_operation]);

        assert_eq!(calculator.calculate_available_balance("mintA"), 100.0);
        assert_eq!(calculator.calculate_available_balance("mintB"), 150.0);
        assert_eq!(calculator.calculate_available_balance("mintC"), 0.0);
    }
}
