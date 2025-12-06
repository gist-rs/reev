//! Conversion utilities for converting protocol results to typed results
//!
//! This module provides implementations for converting between different result types
//! in a more structured way using From trait implementations.

use std::collections::HashMap;

use crate::execution::context_builder::{JupiterLendResult, JupiterSwapResult, TypedToolResult};
use crate::protocols::executor::{
    JupiterEarnResult as ProtocolsJupiterEarnResult,
    JupiterLendResult as ProtocolsJupiterLendResult, JupiterResult,
    JupiterSwapResult as ProtocolsJupiterSwapResult, ProtocolOperation, ProtocolResult,
};
use serde_json::{json, Value};

/// Conversion helper to extract string values from parameters with default fallback
fn get_string_param(params: &HashMap<String, Value>, key: &str) -> String {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Implementation for converting JupiterSwapResult to TypedToolResult
impl From<(&ProtocolsJupiterSwapResult, &ProtocolOperation)> for TypedToolResult {
    fn from((swap_result, operation): (&ProtocolsJupiterSwapResult, &ProtocolOperation)) -> Self {
        TypedToolResult::JupiterSwap(JupiterSwapResult {
            input_mint: get_string_param(&operation.parameters, "input_mint"),
            output_mint: get_string_param(&operation.parameters, "output_mint"),
            input_amount: swap_result.input_amount,
            output_amount: swap_result.output_amount,
            slippage_bps: None,
            instruction_count: None,
            operation_type: "swap".to_string(),
            status: if swap_result.success {
                "success".to_string()
            } else {
                "failed".to_string()
            },
            completed: swap_result.success,
            transaction_signature: swap_result.signature.clone(),
            message: if swap_result.success {
                "Swap executed successfully".to_string()
            } else {
                "Swap failed".to_string()
            },
        })
    }
}

/// Implementation for converting JupiterLendResult to TypedToolResult
impl From<(&ProtocolsJupiterLendResult, &ProtocolOperation)> for TypedToolResult {
    fn from((lend_result, operation): (&ProtocolsJupiterLendResult, &ProtocolOperation)) -> Self {
        TypedToolResult::JupiterLend(JupiterLendResult {
            asset_mint: get_string_param(&operation.parameters, "mint"),
            amount: lend_result.amount,
            protocol: None,
            operation_type: "lend".to_string(),
            status: if lend_result.success {
                "success".to_string()
            } else {
                "failed".to_string()
            },
            completed: lend_result.success,
            transaction_signature: lend_result.signature.clone(),
            message: if lend_result.success {
                "Lend operation executed successfully".to_string()
            } else {
                "Lend operation failed".to_string()
            },
        })
    }
}

/// Implementation for converting JupiterEarnResult to TypedToolResult
impl From<(&ProtocolsJupiterEarnResult, &ProtocolOperation)> for TypedToolResult {
    fn from((_earn_result, _operation): (&ProtocolsJupiterEarnResult, &ProtocolOperation)) -> Self {
        // For now, convert earn to a generic operation
        // TODO: Implement proper JupiterEarnResult conversion
        TypedToolResult::GenericOperation {
            tool_name: "jupiter_earn".to_string(),
            result: json!({ "message": "Earn operation executed" }),
            success: true,
            error: None,
            execution_time_ms: None,
            metadata: HashMap::new(),
        }
    }
}

/// Implementation for converting placeholder message to TypedToolResult
impl From<(&str, &ProtocolOperation)> for TypedToolResult {
    fn from((message, operation): (&str, &ProtocolOperation)) -> Self {
        TypedToolResult::GenericOperation {
            tool_name: operation.operation_type.to_string(),
            result: json!({ "message": message }),
            success: true,
            error: None,
            execution_time_ms: None,
            metadata: HashMap::new(),
        }
    }
}

/// Implementation for converting ProtocolResult with Operation to TypedToolResult
impl From<(&ProtocolResult, &ProtocolOperation)> for TypedToolResult {
    fn from((protocol_result, operation): (&ProtocolResult, &ProtocolOperation)) -> Self {
        match protocol_result {
            ProtocolResult::Jupiter(jupiter_result) => match jupiter_result {
                JupiterResult::Swap(swap_result) => TypedToolResult::from((swap_result, operation)),
                JupiterResult::Lend(lend_result) => TypedToolResult::from((lend_result, operation)),
                JupiterResult::Earn(earn_result) => TypedToolResult::from((earn_result, operation)),
            },
            ProtocolResult::Placeholder(message) => {
                TypedToolResult::from((message.as_str(), operation))
            }
        }
    }
}
