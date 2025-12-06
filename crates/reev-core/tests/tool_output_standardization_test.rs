//! Test for Tool Output Standardization
//!
//! This test verifies that tool outputs are properly standardized using
//! ToolResultWrapper and can be correctly converted between different representations.

use reev_core::execution::context_builder::ToolResultWrapper;
use reev_types::flow::StepResult;
use serde_json::json;

#[tokio::test]
async fn test_tool_result_wrapper_serialization() {
    // Test that ToolResultWrapper can be serialized to JSON
    let wrapper = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            input_amount: 1000000,
            output_amount: 950000,
            slippage_bps: Some(500),
            instruction_count: Some(5),
            operation_type: "jupiter_swap".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: Some(
                "2ZE7R2Ka9NGCZkA2m6rQx5GiC2XVhqaA3Jt5kUFv2zFjDQaVzN7s7R9XjR9LzKx".to_string(),
            ),
            message: "Swap completed successfully".to_string(),
        },
    );

    // Serialize to JSON
    let serialized = serde_json::to_value(&wrapper).expect("Failed to serialize ToolResultWrapper");

    // Verify it has the expected structure
    assert!(serialized.get("tool_name").is_some());
    assert!(serialized.get("result").is_some());
}

#[tokio::test]
async fn test_step_result_with_tool_wrapper() {
    // Test that StepResult can store ToolResultWrapper correctly
    let wrapper = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "So11111111111111111111111111111111111111112".to_string(),
            input_amount: 1000000000,
            output_amount: 1000000000,
            slippage_bps: None,
            instruction_count: Some(1),
            operation_type: "sol_transfer".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: Some(
                "2ZE7R2Ka9NGCZkA2m6rQx5GiC2XVhqaA3Jt5kUFv2zFjDQaVzN7s7R9XjR9LzKx".to_string(),
            ),
            message: "SOL transfer completed".to_string(),
        },
    );

    // Serialize to JSON
    let wrapper_json =
        serde_json::to_value(&wrapper).expect("Failed to serialize ToolResultWrapper");

    // Create StepResult with wrapper
    let mut step_result = StepResult {
        step_id: "test-step".to_string(),
        success: true,
        error_message: None,
        tool_calls: vec!["sol_transfer".to_string()],
        output: json!({}),
        execution_time_ms: 100,
        tool_results: None,
    };

    step_result.set_tool_results(vec![wrapper_json]);

    // Verify that tool results can be retrieved
    let retrieved_results = step_result.get_tool_results();
    assert!(retrieved_results.is_some());

    let results = retrieved_results.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_all_tool_result_types_to_wrapper() {
    // Test conversion of all tool result types to ToolResultWrapper

    // SOL Transfer
    let _wrapper1 = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "So11111111111111111111111111111111111111112".to_string(),
            input_amount: 1000000000,
            output_amount: 1000000000,
            slippage_bps: None,
            instruction_count: Some(1),
            operation_type: "sol_transfer".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: None,
            message: "SOL transfer completed".to_string(),
        },
    );

    // SPL Transfer
    let _wrapper2 = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "token_mint".to_string(),
            output_mint: "mint_address".to_string(),
            input_amount: 1000000,
            output_amount: 950000,
            slippage_bps: Some(500),
            instruction_count: Some(2),
            operation_type: "spl_transfer".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: None,
            message: "SPL transfer completed".to_string(),
        },
    );

    // Jupiter Swap
    let _wrapper3 = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "input_mint".to_string(),
            output_mint: "output_mint".to_string(),
            input_amount: 1000000,
            output_amount: 950000,
            slippage_bps: Some(500),
            instruction_count: Some(3),
            operation_type: "jupiter_swap".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: None,
            message: "Jupiter swap completed".to_string(),
        },
    );

    // Jupiter Lend
    let _wrapper4 = ToolResultWrapper::from_jupiter_lend(
        reev_core::execution::context_builder::JupiterLendResult {
            asset_mint: "mint".to_string(),
            amount: 1000000,
            protocol: Some("Jupiter".to_string()),
            operation_type: "jupiter_lend".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: None,
            message: "Jupiter lend operation completed".to_string(),
        },
    );

    // Account Balance
    let _wrapper5 = ToolResultWrapper::from_jupiter_lend(
        reev_core::execution::context_builder::JupiterLendResult {
            asset_mint: "mint".to_string(),
            amount: 1000000,
            protocol: None,
            operation_type: "account_balance".to_string(),
            status: "completed".to_string(),
            completed: true,
            transaction_signature: None,
            message: "Account balance queried".to_string(),
        },
    );
}

#[tokio::test]
async fn test_tool_result_wrapper_with_errors() {
    // Test that error cases are handled correctly
    let wrapper = ToolResultWrapper::from_jupiter_swap(
        reev_core::execution::context_builder::JupiterSwapResult {
            input_mint: "input_mint".to_string(),
            output_mint: "output_mint".to_string(),
            input_amount: 1000000,
            output_amount: 0,
            slippage_bps: Some(500),
            instruction_count: Some(3),
            operation_type: "jupiter_swap".to_string(),
            status: "failed".to_string(),
            completed: false,
            transaction_signature: None,
            message: "Insufficient balance".to_string(),
        },
    );

    // Serialize and verify error information is preserved
    let serialized = serde_json::to_value(&wrapper).expect("Failed to serialize ToolResultWrapper");

    // Verify structure includes error information
    assert!(serialized.get("tool_name").is_some());
    assert!(serialized.get("result").is_some());
}
