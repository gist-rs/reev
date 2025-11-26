//! Tests for unified flow builder module

use reev_core::refiner::RefinedPrompt;
use reev_core::yml_generator::UnifiedFlowBuilder;

#[tokio::test]
async fn test_build_flow_preserves_operation_type() {
    // Create a unified flow builder
    let builder = UnifiedFlowBuilder::new();

    // Create a mock wallet context
    let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

    // Create a mock refined prompt for a swap operation
    let refined_prompt = RefinedPrompt::new_for_test(
        "swap 0.1 SOL for USDC".to_string(),
        "swap 0.1 SOL for USDC".to_string(),
        false,
    );

    // Build the flow
    let flow = builder
        .build_flow_from_operations(&refined_prompt, &wallet_context)
        .await
        .unwrap();

    // Verify the flow preserves the operation type
    assert_eq!(flow.user_prompt, "swap 0.1 SOL for USDC");
    assert_eq!(flow.refined_prompt, "swap 0.1 SOL for USDC");
    assert_eq!(flow.steps.len(), 1);

    // Verify step has refined prompt
    let step = &flow.steps[0];
    assert_eq!(step.refined_prompt, "swap 0.1 SOL for USDC");

    // Verify no pre-determined tools (RigAgent should determine these)
    assert_eq!(step.expected_tools, None);
    assert_eq!(step.expected_tool_calls, None);
}

#[tokio::test]
async fn test_build_flow_preserves_transfer_operation() {
    // Create a unified flow builder
    let builder = UnifiedFlowBuilder::new();

    // Create a mock wallet context
    let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

    // Create a mock refined prompt for a transfer operation
    let refined_prompt = RefinedPrompt::new_for_test(
        "send 1 SOL to address123".to_string(),
        "transfer 1 SOL to address123".to_string(),
        true,
    );

    // Build the flow
    let flow = builder
        .build_flow_from_operations(&refined_prompt, &wallet_context)
        .await
        .unwrap();

    // Verify the flow preserves the operation type
    assert_eq!(flow.user_prompt, "send 1 SOL to address123");
    assert_eq!(flow.refined_prompt, "transfer 1 SOL to address123");
    assert_eq!(flow.steps.len(), 1);

    // Verify step has refined prompt
    let step = &flow.steps[0];
    assert_eq!(step.refined_prompt, "transfer 1 SOL to address123");

    // Verify no pre-determined tools (RigAgent should determine these)
    assert_eq!(step.expected_tools, None);
    assert_eq!(step.expected_tool_calls, None);
}

#[tokio::test]
async fn test_build_flow_preserves_lend_operation() {
    // Create a unified flow builder
    let builder = UnifiedFlowBuilder::new();

    // Create a mock wallet context
    let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

    // Create a mock refined prompt for a lend operation
    let refined_prompt = RefinedPrompt::new_for_test(
        "lend 100 USDC".to_string(),
        "lend 100 USDC".to_string(),
        false,
    );

    // Build the flow
    let flow = builder
        .build_flow_from_operations(&refined_prompt, &wallet_context)
        .await
        .unwrap();

    // Verify the flow preserves the operation type
    assert_eq!(flow.user_prompt, "lend 100 USDC");
    assert_eq!(flow.refined_prompt, "lend 100 USDC");
    assert_eq!(flow.steps.len(), 1);

    // Verify step has refined prompt
    let step = &flow.steps[0];
    assert_eq!(step.refined_prompt, "lend 100 USDC");

    // Verify no pre-determined tools (RigAgent should determine these)
    assert_eq!(step.expected_tools, None);
    assert_eq!(step.expected_tool_calls, None);
}
