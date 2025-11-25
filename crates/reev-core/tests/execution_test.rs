//! Tests for the executor module
#[path = "common/mod.rs"]
mod common;
use common::mock_helpers::mock_tool_executor::MockToolExecutor;
use reev_core::yml_schema::YmlStep;
use reev_types::flow::WalletContext;

#[tokio::test]
async fn test_mock_tool_executor() {
    let mock_executor = MockToolExecutor::new().with_success(true);

    // Create a test step
    let test_step = YmlStep {
        step_id: "test_step_1".to_string(),
        prompt: "Swap 1 SOL to USDC".to_string(),
        refined_prompt: "Swap 1 SOL to USDC".to_string(),
        context: "Test context".to_string(),
        critical: Some(true),
        estimated_time_seconds: Some(30),
        expected_tool_calls: Some(vec![reev_core::yml_schema::YmlToolCall {
            tool_name: reev_types::tools::ToolName::JupiterSwap,
            critical: true,
            expected_parameters: None,
        }]),
        expected_tools: None,
    };

    // Create a test wallet context
    let wallet_context = WalletContext::new("test_wallet".to_string());

    // Execute the step
    let result = mock_executor
        .execute_step(&test_step, &wallet_context)
        .await
        .expect("Failed to execute step");

    // Verify the result
    assert!(result.success);
    assert_eq!(result.step_id, "test_step_1");
    assert_eq!(result.tool_calls, vec!["JupiterSwap"]);
    assert!(result.error_message.is_none());
    assert_eq!(result.execution_time_ms, 50);
}

#[tokio::test]
async fn test_mock_tool_executor_failure() {
    let mock_executor = MockToolExecutor::new().with_success(false);

    // Create a test step
    let test_step = YmlStep {
        step_id: "test_step_1".to_string(),
        prompt: "Swap 1 SOL to USDC".to_string(),
        refined_prompt: "Swap 1 SOL to USDC".to_string(),
        context: "Test context".to_string(),
        critical: Some(true),
        estimated_time_seconds: Some(30),
        expected_tool_calls: Some(vec![reev_core::yml_schema::YmlToolCall {
            tool_name: reev_types::tools::ToolName::JupiterSwap,
            critical: true,
            expected_parameters: None,
        }]),
        expected_tools: None,
    };

    // Create a test wallet context
    let wallet_context = WalletContext::new("test_wallet".to_string());

    // Execute the step
    let result = mock_executor
        .execute_step(&test_step, &wallet_context)
        .await
        .expect("Failed to execute step");

    // Verify the result
    assert!(!result.success);
    assert_eq!(result.step_id, "test_step_1");
    assert_eq!(
        result.error_message,
        Some("Mock execution failure for testing".to_string())
    );
}
