//! Test for expected_tools preservation through YML conversion
//!
//! This test verifies that the expected_tools field is properly preserved
//! when converting between YmlStep and DynamicStep.

use anyhow::Result;
use reev_core::executor::YmlConverter;
use reev_core::yml_schema::{YmlStep, YmlToolCall};
use reev_types::flow::DynamicStep;
use reev_types::tools::ToolName;

#[tokio::test]
async fn test_expected_tools_preservation() -> Result<()> {
    // Create a YmlStep with expected_tools
    let yml_step = YmlStep {
        step_id: "test-step-1".to_string(),
        prompt: "send 1 SOL to test".to_string(),
        refined_prompt: "send 1 SOL to test".to_string(),
        context: "Test context".to_string(),
        critical: Some(true),
        estimated_time_seconds: Some(30),
        expected_tool_calls: Some(vec![YmlToolCall::new(ToolName::SolTransfer, true)]),
        expected_tools: Some(vec![ToolName::SolTransfer]),
    };

    // Convert YmlStep to DynamicStep
    let converter = YmlConverter::new();
    let dynamic_step = YmlConverter::yml_to_dynamic_step(&yml_step, "test-flow")?;

    // Verify expected_tools are preserved in DynamicStep
    assert_eq!(
        dynamic_step.expected_tools,
        Some(vec![ToolName::SolTransfer]),
        "expected_tools should be preserved in DynamicStep"
    );

    // Convert DynamicStep back to YmlStep
    let converted_yml_step = converter.dynamic_step_to_yml_step(&dynamic_step)?;

    // Verify expected_tools are preserved in converted YmlStep
    assert_eq!(
        converted_yml_step.expected_tools,
        Some(vec![ToolName::SolTransfer]),
        "expected_tools should be preserved when converting back to YmlStep"
    );

    Ok(())
}

#[tokio::test]
async fn test_expected_tools_with_multiple_tools() -> Result<()> {
    // Create a YmlStep with multiple expected_tools
    let yml_step = YmlStep {
        step_id: "multi-tool-step".to_string(),
        prompt: "swap SOL to USDC".to_string(),
        refined_prompt: "swap SOL to USDC".to_string(),
        context: "Multi-tool test".to_string(),
        critical: Some(true),
        estimated_time_seconds: Some(60),
        expected_tool_calls: None,
        expected_tools: Some(vec![ToolName::JupiterSwap, ToolName::SplTransfer]),
    };

    // Convert YmlStep to DynamicStep
    let converter = YmlConverter::new();
    let dynamic_step = YmlConverter::yml_to_dynamic_step(&yml_step, "test-flow")?;

    // Verify all expected_tools are preserved
    assert_eq!(
        dynamic_step.expected_tools,
        Some(vec![ToolName::JupiterSwap, ToolName::SplTransfer]),
        "All expected_tools should be preserved in DynamicStep"
    );

    // Convert DynamicStep back to YmlStep
    let converted_yml_step = converter.dynamic_step_to_yml_step(&dynamic_step)?;

    // Verify all expected_tools are preserved in converted YmlStep
    assert_eq!(
        converted_yml_step.expected_tools,
        Some(vec![ToolName::JupiterSwap, ToolName::SplTransfer]),
        "All expected_tools should be preserved when converting back to YmlStep"
    );

    Ok(())
}

#[tokio::test]
async fn test_dynamic_step_without_expected_tools() -> Result<()> {
    // Create a DynamicStep without expected_tools
    let dynamic_step = DynamicStep::new(
        "no-tools-step".to_string(),
        "test prompt".to_string(),
        "test description".to_string(),
    );

    // Convert DynamicStep to YmlStep
    let converter = YmlConverter::new();
    let yml_step = converter.dynamic_step_to_yml_step(&dynamic_step)?;

    // Verify empty expected_tools list is created
    assert_eq!(
        yml_step.expected_tools,
        Some(vec![]),
        "Empty expected_tools list should be created when DynamicStep has no expected_tools"
    );

    Ok(())
}
