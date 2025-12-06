//! Tests for the conversion logic between ProtocolResult and TypedToolResult

use reev_core::benchmark::runner::static_runner::StaticBenchmarkRunner;
use reev_core::execution::context_builder::TypedToolResult;
use reev_core::protocols::{
    JupiterLendResult, JupiterResult, JupiterSwapResult, OperationType, ProtocolOperation,
    ProtocolResult,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_jupiter_swap_conversion() {
    // Create a mock swap result
    let swap_result = JupiterSwapResult {
        success: true,
        signature: Some("abc123".to_string()),
        input_amount: 1000000,
        output_amount: 20000000,
        price_impact: Some(0.01),
    };

    // Create operation parameters
    let mut parameters = HashMap::new();
    parameters.insert(
        "input_mint".to_string(),
        json!("So11111111111111111111111111111111111111112"),
    );
    parameters.insert(
        "output_mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    // Convert protocol result to typed result
    let protocol_result = ProtocolResult::Jupiter(JupiterResult::Swap(swap_result));
    let runner = StaticBenchmarkRunner::new();
    let typed_result = runner
        .convert_protocol_result_to_typed_result(&protocol_result, &operation)
        .unwrap();

    // Verify conversion
    match typed_result {
        TypedToolResult::JupiterSwap(swap) => {
            assert_eq!(
                swap.input_mint,
                "So11111111111111111111111111111111111111112"
            );
            assert_eq!(
                swap.output_mint,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            );
            assert_eq!(swap.input_amount, 1000000);
            assert_eq!(swap.output_amount, 20000000);
            assert_eq!(swap.operation_type, "swap");
            assert_eq!(swap.status, "success");
            assert!(swap.completed);
            assert_eq!(swap.transaction_signature, Some("abc123".to_string()));
            assert_eq!(swap.message, "Swap executed successfully");
        }
        _ => panic!("Expected JupiterSwap result"),
    }
}

#[test]
fn test_jupiter_lend_conversion() {
    // Create a mock lend result
    let lend_result = JupiterLendResult {
        success: true,
        signature: Some("def456".to_string()),
        amount: 5000000,
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    };

    // Create operation parameters
    let mut parameters = HashMap::new();
    parameters.insert(
        "mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );

    let operation = ProtocolOperation {
        operation_type: OperationType::Lend,
        parameters,
    };

    // Convert protocol result to typed result
    let protocol_result = ProtocolResult::Jupiter(JupiterResult::Lend(lend_result));
    let runner = StaticBenchmarkRunner::new();
    let typed_result = runner
        .convert_protocol_result_to_typed_result(&protocol_result, &operation)
        .unwrap();

    // Verify conversion
    match typed_result {
        TypedToolResult::JupiterLend(lend) => {
            assert_eq!(
                lend.asset_mint,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            );
            assert_eq!(lend.amount, 5000000);
            assert_eq!(lend.operation_type, "lend");
            assert_eq!(lend.status, "success");
            assert!(lend.completed);
            assert_eq!(lend.transaction_signature, Some("def456".to_string()));
            assert_eq!(lend.message, "Lend operation executed successfully");
        }
        _ => panic!("Expected JupiterLend result"),
    }
}

#[test]
fn test_failed_swap_conversion() {
    // Create a failed swap result
    let swap_result = JupiterSwapResult {
        success: false,
        signature: None,
        input_amount: 0,
        output_amount: 0,
        price_impact: None,
    };

    // Create operation parameters
    let mut parameters = HashMap::new();
    parameters.insert(
        "input_mint".to_string(),
        json!("So11111111111111111111111111111111111111112"),
    );
    parameters.insert(
        "output_mint".to_string(),
        json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    );

    let operation = ProtocolOperation {
        operation_type: OperationType::Swap,
        parameters,
    };

    // Convert protocol result to typed result
    let protocol_result = ProtocolResult::Jupiter(JupiterResult::Swap(swap_result));
    let runner = StaticBenchmarkRunner::new();
    let typed_result = runner
        .convert_protocol_result_to_typed_result(&protocol_result, &operation)
        .unwrap();

    // Verify conversion
    match typed_result {
        TypedToolResult::JupiterSwap(swap) => {
            assert_eq!(swap.status, "failed");
            assert!(!swap.completed);
            assert_eq!(swap.message, "Swap failed");
        }
        _ => panic!("Expected JupiterSwap result"),
    }
}

#[test]
fn test_placeholder_conversion() {
    // Create a placeholder result
    let protocol_result = ProtocolResult::Placeholder("Test message".to_string());

    let operation = ProtocolOperation {
        operation_type: OperationType::Custom("test_operation".to_string()),
        parameters: HashMap::new(),
    };

    // Convert protocol result to typed result
    let runner = StaticBenchmarkRunner::new();
    let typed_result = runner
        .convert_protocol_result_to_typed_result(&protocol_result, &operation)
        .unwrap();

    // Verify conversion
    match typed_result {
        TypedToolResult::GenericOperation {
            tool_name,
            result,
            success,
            ..
        } => {
            assert_eq!(tool_name, "Custom(test_operation)");
            assert_eq!(result["message"], "Test message");
            assert!(success);
        }
        _ => panic!("Expected GenericOperation result"),
    }
}
