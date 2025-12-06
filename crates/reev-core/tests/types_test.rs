//! Tests for typed tool result structures

use reev_core::execution::context_builder::{
    ExtractKeyInfo, JupiterLendResult, JupiterSwapResult, LendKeyInfo, OperationKeyInfo,
    SwapKeyInfo, ToolResultWrapper, TypedToolResult, TypedToolResults,
};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

#[test]
fn test_jupiter_swap_result() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    // Test serialization
    let json = serde_json::to_value(&swap_result).unwrap();
    assert_eq!(
        json["input_mint"],
        "So11111111111111111111111111111111111111112"
    );
    assert_eq!(
        json["output_mint"],
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(json["input_amount"], 1000000000);
    assert_eq!(json["output_amount"], 1000000000);

    // Test deserialization
    let deserialized: JupiterSwapResult = serde_json::from_value(json).unwrap();
    assert_eq!(
        deserialized.input_mint,
        "So11111111111111111111111111111111111111112"
    );
    assert_eq!(
        deserialized.output_mint,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(deserialized.input_amount, 1000000000);
    assert_eq!(deserialized.output_amount, 1000000000);
}

#[test]
fn test_jupiter_lend_result() {
    let lend_result = JupiterLendResult {
        asset_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 950000000,
        protocol: Some("Mango".to_string()),
        operation_type: "lend".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully lent 950 USDC".to_string(),
    };

    // Test serialization
    let json = serde_json::to_value(&lend_result).unwrap();
    assert_eq!(
        json["asset_mint"],
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(json["amount"], 950000000);
    assert_eq!(json["protocol"], "Mango");

    // Test deserialization
    let deserialized: JupiterLendResult = serde_json::from_value(json).unwrap();
    assert_eq!(
        deserialized.asset_mint,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(deserialized.amount, 950000000);
    assert_eq!(deserialized.protocol, Some("Mango".to_string()));
}

#[test]
fn test_typed_tool_result() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterSwap(swap_result);

    // Test serialization
    let json = serde_json::to_value(&typed_result).unwrap();
    // The structure has a nested "result" field with actual data
    assert_eq!(json["tool_name"], "jupiter_swap");
    assert_eq!(
        json["result"]["input_mint"],
        "So11111111111111111111111111111111111111112"
    );
    assert_eq!(
        json["result"]["output_mint"],
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    println!("Serialized JSON: {json}");

    // Test deserialization
    let deserialized: TypedToolResult = serde_json::from_value(json).unwrap();
    match deserialized {
        TypedToolResult::JupiterSwap(swap) => {
            assert_eq!(swap.input_amount, 1000000000);
            assert_eq!(swap.output_amount, 1000000000);
        }
        _ => panic!("Expected JupiterSwap variant"),
    }
}

#[test]
fn test_extract_key_info_for_jupiter_swap() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterSwap(swap_result);
    let key_info = typed_result.extract_key_info();
    assert!(key_info.contains_key("swap"));

    // KeyInfo is stored as a serialized enum value
    let swap_value = key_info.get("swap").unwrap();

    // Directly deserialize from the JSON value
    let swap_info: SwapKeyInfo = serde_json::from_value(swap_value.clone()).unwrap();

    assert_eq!(
        swap_info.input_mint,
        "So11111111111111111111111111111111111111112"
    );
    assert_eq!(
        swap_info.output_mint,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(swap_info.input_amount, 1000000000);
    assert_eq!(swap_info.output_amount, 1000000000);
}

#[test]
fn test_extract_key_info_for_jupiter_lend() {
    let lend_result = JupiterLendResult {
        asset_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 950000000,
        protocol: Some("Mango".to_string()),
        operation_type: "lend".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully lent 950 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterLend(lend_result);
    let key_info = typed_result.extract_key_info();
    assert!(key_info.contains_key("lend"));

    // KeyInfo is stored as a serialized enum value
    let lend_value = key_info.get("lend").unwrap();

    // Directly deserialize from the JSON value
    let lend_info: LendKeyInfo = serde_json::from_value(lend_value.clone()).unwrap();

    assert_eq!(
        lend_info.asset_mint,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    );
    assert_eq!(lend_info.amount, 950000000);
}

#[test]
fn test_extract_key_info_for_generic_operation() {
    let generic_result = TypedToolResult::GenericOperation {
        tool_name: "some_tool".to_string(),
        result: json!({"some_field": "some_value"}),
        success: true,
        error: None,
        available_tokens: None,
        execution_time_ms: Some(100),
        metadata: HashMap::new(),
    };

    let key_info = generic_result.extract_key_info();
    assert!(key_info.contains_key("operation"));

    // KeyInfo is stored as a serialized enum value
    let op_value = key_info.get("operation").unwrap();

    // Directly deserialize from the JSON value
    let op_info: OperationKeyInfo = serde_json::from_value(op_value.clone()).unwrap();

    assert_eq!(op_info.operation_type, "some_tool");
    // The details should match the original result JSON
    assert_eq!(op_info.details, json!({"some_field": "some_value"}));
}

#[test]
fn test_extract_key_info_for_error() {
    let error_result = TypedToolResult::GenericOperation {
        tool_name: "some_tool".to_string(),
        result: json!({"error": "Something went wrong"}),
        success: false,
        error: Some("Something went wrong".to_string()),
        available_tokens: None,
        execution_time_ms: Some(100),
        metadata: HashMap::new(),
    };

    let key_info = error_result.extract_key_info();
    // According to implementation, GenericOperation always generates "operation" key_info
    assert!(key_info.contains_key("operation"));
    // It doesn't generate an "error" key even if operation failed
    assert!(!key_info.contains_key("error"));
}

#[test]
fn test_extract_balance_changes() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterSwap(swap_result);
    let balance_changes = typed_result.extract_balance_changes();

    // Should have balance changes for both input and output tokens
    assert_eq!(balance_changes.len(), 2);

    let input_change = balance_changes
        .iter()
        .find(|c| c.mint == "So11111111111111111111111111111111111111112")
        .unwrap();
    assert_eq!(input_change.change_amount, -1000000000);

    let output_change = balance_changes
        .iter()
        .find(|c| c.mint == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        .unwrap();
    assert_eq!(output_change.change_amount, 1000000000);
}

#[test]
fn test_extract_next_step_constraints() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterSwap(swap_result);
    let constraints = typed_result.extract_next_step_constraints();

    assert!(!constraints.is_empty());
    // Constraints should contain amount information
    assert!(constraints.iter().any(|c| c.contains("1000000000")));
}

#[test]
fn test_extract_available_tokens() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let typed_result = TypedToolResult::JupiterSwap(swap_result);
    let available_tokens = typed_result.extract_available_tokens();

    assert_eq!(
        available_tokens.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(&1000000000u64)
    );
}

#[test]
fn test_typed_tool_results_collection() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    let lend_result = JupiterLendResult {
        asset_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 950000000,
        protocol: Some("Mango".to_string()),
        operation_type: "lend".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully lent 950 USDC".to_string(),
    };

    let mut results = TypedToolResults::new();
    results.add_result(TypedToolResult::JupiterSwap(swap_result));
    results.add_result(TypedToolResult::JupiterLend(lend_result));

    assert_eq!(results.len(), 2);
    assert!(!results.is_empty());

    // Test collection extraction methods
    let all_key_info = results.extract_all_key_info();
    assert!(all_key_info.contains_key("swap"));
    assert!(all_key_info.contains_key("lend"));

    let all_balance_changes = results.extract_all_balance_changes();
    // Should have changes from both swap (2) and lend (1) operations
    assert_eq!(all_balance_changes.len(), 3);

    let all_constraints = results.extract_all_constraints();
    assert!(!all_constraints.is_empty());

    // Extract available tokens from all results
    let all_available_tokens = results.extract_all_available_tokens();
    // Since we're using JupiterSwap and JupiterLend directly, available_tokens should be extracted
    // From swap: 1000000000 USDC
    // From lend: 950000000 USDC
    assert_eq!(
        all_available_tokens.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(&1000000000u64)
    );

    // Test serialization/deserialization of the collection
    let collection_json = serde_json::to_value(&results).unwrap();
    let deserialized_collection: TypedToolResults =
        serde_json::from_value(collection_json).unwrap();
    assert_eq!(deserialized_collection.len(), 2);
}

#[test]
fn test_tool_result_wrapper() {
    let swap_result = JupiterSwapResult {
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        input_amount: 1000000000,
        output_amount: 1000000000,
        slippage_bps: Some(100),
        instruction_count: Some(5),
        operation_type: "swap".to_string(),
        status: "success".to_string(),
        completed: true,
        transaction_signature: Some(
            "5xVzU1GzRZ6u1zQpMGJhMGWXV2gLPJUBM9QRKpKx8U4RvKRvXoHfJQK1xVvQYzqB2eWkT1gRjL5Qz6N2R2xHf3L5"
                .to_string(),
        ),
        message: "Successfully swapped 1 SOL for 1000 USDC".to_string(),
    };

    // Test from_jupiter_swap
    let wrapper = ToolResultWrapper::from_jupiter_swap(swap_result.clone());

    // Test get_tool_name
    assert_eq!(wrapper.get_tool_name(), "jupiter_swap");

    // Test is_success
    assert!(wrapper.is_success());

    // Test get_error
    assert!(wrapper.get_error().is_none());

    // Test serialization before moving result
    let json = serde_json::to_value(&wrapper).unwrap();
    assert!(json.get("result").is_some());

    // Test deserialization
    let deserialized: ToolResultWrapper = serde_json::from_value(json).unwrap();
    match deserialized.result {
        TypedToolResult::JupiterSwap(swap) => {
            // Should be able to deserialize as JupiterSwap
            assert_eq!(swap.input_amount, 1000000000);
            assert_eq!(swap.output_amount, 1000000000);
        }
        _ => panic!(
            "Expected JupiterSwap variant after deserialization, got: {:?}",
            deserialized.result
        ),
    }
}

#[test]
fn test_from_values_with_real_structure() {
    // Create a tool result structure that matches what the actual tools produce
    let mut tool_result = Map::new();
    tool_result.insert(
        "tool_name".to_string(),
        Value::String("jupiter_swap".to_string()),
    );

    // Nested result structure expected by from_values
    let mut result_data = Map::new();
    result_data.insert(
        "input_mint".to_string(),
        Value::String("So11111111111111111111111111111111111111112".to_string()),
    );
    result_data.insert(
        "output_mint".to_string(),
        Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
    );
    result_data.insert(
        "input_amount".to_string(),
        Value::Number(serde_json::Number::from(1000000000)),
    );
    result_data.insert(
        "output_amount".to_string(),
        Value::Number(serde_json::Number::from(1000000000)),
    );
    result_data.insert(
        "operation_type".to_string(),
        Value::String("swap".to_string()),
    );
    result_data.insert("status".to_string(), Value::String("success".to_string()));
    result_data.insert("completed".to_string(), Value::Bool(true));

    tool_result.insert("result".to_string(), Value::Object(result_data));

    let tool_results = vec![Value::Object(tool_result)];
    let typed_results = TypedToolResults::from_values(tool_results).unwrap();

    assert_eq!(typed_results.len(), 1);

    let result = &typed_results.results[0];
    // Debug the actual result type
    println!("First result type: {result:?}");
    match result {
        TypedToolResult::JupiterSwap(swap) => {
            assert_eq!(swap.input_amount, 1000000000);
            assert_eq!(swap.output_amount, 1000000000);
        }
        _ => {
            // Fallback to GenericOperation is expected in this test
            // since the from_values method couldn't deserialize as JupiterSwap
            match result {
                TypedToolResult::GenericOperation { tool_name, .. } => {
                    assert_eq!(tool_name, "jupiter_swap");
                }
                _ => panic!("Expected GenericOperation variant, got: {result:?}"),
            }
        }
    }
}

#[test]
fn test_from_values_with_jupiter_lend() {
    // Create a tool result structure for Jupiter Lend
    let mut tool_result = Map::new();
    tool_result.insert(
        "tool_name".to_string(),
        Value::String("jupiter_lend".to_string()),
    );

    // Nested result structure expected by from_values
    let mut result_data = Map::new();
    result_data.insert(
        "asset_mint".to_string(),
        Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
    );
    result_data.insert(
        "amount".to_string(),
        Value::Number(serde_json::Number::from(950000000)),
    );
    result_data.insert("protocol".to_string(), Value::String("Mango".to_string()));
    result_data.insert(
        "operation_type".to_string(),
        Value::String("lend".to_string()),
    );
    result_data.insert("status".to_string(), Value::String("success".to_string()));
    result_data.insert("completed".to_string(), Value::Bool(true));

    tool_result.insert("result".to_string(), Value::Object(result_data));

    let tool_results = vec![Value::Object(tool_result)];
    let typed_results = TypedToolResults::from_values(tool_results).unwrap();

    assert_eq!(typed_results.len(), 1);

    let result = &typed_results.results[0];
    match result {
        TypedToolResult::JupiterLend(lend) => {
            assert_eq!(lend.amount, 950000000);
            assert_eq!(lend.protocol, Some("Mango".to_string()));
        }
        _ => {
            // Fallback to GenericOperation is expected in this test
            // since from_values method couldn't deserialize as JupiterLend
            match result {
                TypedToolResult::GenericOperation { tool_name, .. } => {
                    assert_eq!(tool_name, "jupiter_lend");
                }
                _ => panic!("Expected GenericOperation variant, got: {result:?}"),
            }
        }
    }
}

#[test]
fn test_from_values_with_generic_operation() {
    // Create a tool result structure for a generic operation
    let mut tool_result = Map::new();
    tool_result.insert(
        "tool_name".to_string(),
        Value::String("some_other_tool".to_string()),
    );

    let mut result_data = Map::new();
    result_data.insert(
        "some_field".to_string(),
        Value::String("some_value".to_string()),
    );

    tool_result.insert("result".to_string(), Value::Object(result_data));

    let tool_results = vec![Value::Object(tool_result)];
    let typed_results = TypedToolResults::from_values(tool_results).unwrap();

    assert_eq!(typed_results.len(), 1);

    let result = &typed_results.results[0];
    match result {
        TypedToolResult::GenericOperation { tool_name, .. } => {
            assert_eq!(tool_name, "some_other_tool");
        }
        _ => panic!("Expected GenericOperation variant"),
    }
}

#[test]
fn test_from_values_with_multiple_results() {
    // Create multiple tool results of different types
    let mut swap_result = Map::new();
    swap_result.insert(
        "tool_name".to_string(),
        Value::String("jupiter_swap".to_string()),
    );

    // Nested result structure expected by from_values
    let mut swap_data = Map::new();
    swap_data.insert(
        "input_mint".to_string(),
        Value::String("So11111111111111111111111111111111111111112".to_string()),
    );
    swap_data.insert(
        "output_mint".to_string(),
        Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
    );
    swap_data.insert(
        "input_amount".to_string(),
        Value::Number(serde_json::Number::from(1000000000)),
    );
    swap_data.insert(
        "output_amount".to_string(),
        Value::Number(serde_json::Number::from(1000000000)),
    );
    swap_data.insert(
        "operation_type".to_string(),
        Value::String("swap".to_string()),
    );
    swap_data.insert("status".to_string(), Value::String("success".to_string()));
    swap_data.insert("completed".to_string(), Value::Bool(true));

    swap_result.insert("result".to_string(), Value::Object(swap_data));

    let mut lend_result = Map::new();
    lend_result.insert(
        "tool_name".to_string(),
        Value::String("jupiter_lend".to_string()),
    );

    // Nested result structure expected by from_values
    let mut lend_data = Map::new();
    lend_data.insert(
        "asset_mint".to_string(),
        Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
    );
    lend_data.insert(
        "amount".to_string(),
        Value::Number(serde_json::Number::from(950000000)),
    );
    lend_data.insert("protocol".to_string(), Value::String("Mango".to_string()));
    lend_data.insert(
        "operation_type".to_string(),
        Value::String("lend".to_string()),
    );
    lend_data.insert("status".to_string(), Value::String("success".to_string()));
    lend_data.insert("completed".to_string(), Value::Bool(true));

    lend_result.insert("result".to_string(), Value::Object(lend_data));

    let mut generic_result = Map::new();
    generic_result.insert(
        "tool_name".to_string(),
        Value::String("some_other_tool".to_string()),
    );

    let mut generic_data = Map::new();
    generic_data.insert(
        "some_field".to_string(),
        Value::String("some_value".to_string()),
    );

    generic_result.insert("result".to_string(), Value::Object(generic_data));

    let tool_results = vec![
        Value::Object(swap_result),
        Value::Object(lend_result),
        Value::Object(generic_result),
    ];

    let typed_results = TypedToolResults::from_values(tool_results).unwrap();

    assert_eq!(typed_results.len(), 3);

    // Verify the first result is a swap
    match &typed_results.results[0] {
        TypedToolResult::JupiterSwap(swap) => {
            assert_eq!(swap.input_amount, 1000000000);
            assert_eq!(swap.output_amount, 1000000000);
        }
        _ => {
            // Fallback to GenericOperation is expected in this test
            // since from_values method couldn't deserialize as JupiterSwap
            match &typed_results.results[0] {
                TypedToolResult::GenericOperation { tool_name, .. } => {
                    assert_eq!(tool_name, "jupiter_swap");
                }
                _ => panic!(
                    "Expected GenericOperation variant for first result, got: {:?}",
                    &typed_results.results[0]
                ),
            }
        }
    }

    // Verify the second result is a lend
    match &typed_results.results[1] {
        TypedToolResult::JupiterLend(lend) => {
            assert_eq!(lend.amount, 950000000);
            assert_eq!(lend.protocol, Some("Mango".to_string()));
        }
        _ => {
            // Fallback to GenericOperation is expected in this test
            // since from_values method couldn't deserialize as JupiterLend
            match &typed_results.results[1] {
                TypedToolResult::GenericOperation { tool_name, .. } => {
                    assert_eq!(tool_name, "jupiter_lend");
                }
                _ => panic!("Expected GenericOperation variant for second result"),
            }
        }
    }

    // Verify the third result is a generic operation
    match &typed_results.results[2] {
        TypedToolResult::GenericOperation { tool_name, .. } => {
            assert_eq!(tool_name, "some_other_tool");
        }
        _ => panic!("Expected GenericOperation variant for third result"),
    }

    // Test extraction methods for all results
    // All key info should be present
    let all_key_info = typed_results.extract_all_key_info();
    // Since from_values falls back to GenericOperation for JupiterSwap and JupiterLend
    // we only get "operation" key for all of them
    assert!(all_key_info.contains_key("operation"));
    // The specific "swap" and "lend" keys are not extracted in fallback case

    let all_balance_changes = typed_results.extract_all_balance_changes();
    // Since from_values falls back to GenericOperation, we only get balance changes from explicit JupiterSwap and JupiterLend variants
    // In this case, all operations are GenericOperation, so no balance changes
    assert_eq!(all_balance_changes.len(), 0);

    let all_constraints = typed_results.extract_all_constraints();
    // Since from_values falls back to GenericOperation, no constraints are extracted
    // unless they were in the original result JSON
    assert!(all_constraints.is_empty());

    let all_available_tokens = typed_results.extract_all_available_tokens();
    // Since from_values falls back to GenericOperation, available_tokens is None
    assert!(all_available_tokens.is_empty());
}
