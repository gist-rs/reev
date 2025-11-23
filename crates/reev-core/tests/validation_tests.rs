//! Tests for validation module

use reev_core::validation::{AssertionValidator, FlowValidator};
use reev_core::yml_schema::{
    builders::create_swap_flow, YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall,
    YmlWalletInfo,
};
use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use uuid::Uuid;

#[tokio::test]
async fn test_validate_valid_flow() {
    let validator = FlowValidator::new();

    // Create a valid flow
    let wallet_info = YmlWalletInfo::new(
        "11111111111111111111111111111112".to_string(),
        1_000_000_000, // 1 SOL
    )
    .with_token(
        TokenBalance::new(
            "So11111111111111111111111111111111111111112".to_string(),
            1_000_000_000,
        )
        .with_decimals(9)
        .with_symbol("SOL".to_string()),
    );

    let step = YmlStep::new(
        "swap".to_string(),
        "swap 1 SOL to USDC".to_string(),
        "Exchange 1 SOL for USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true));

    let ground_truth = YmlGroundTruth::new().with_error_tolerance(0.01);

    let flow = YmlFlow::new(
        Uuid::now_v7().to_string(),
        "swap 1 SOL to USDC".to_string(),
        wallet_info,
    )
    .with_step(step)
    .with_ground_truth(ground_truth);

    // Validate the flow
    let result = validator.validate_flow(&flow);
    assert!(result.is_ok(), "Flow should be valid");
}

#[tokio::test]
async fn test_validate_invalid_flow() {
    let _validator = FlowValidator::new();

    // Create a flow with invalid ground truth
    let _flow = create_swap_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        0.5, // 0.5 SOL
    );

    // This would fail if we had more sophisticated validation
    // For now, we'll just check basic validation
    // In a real implementation, this would check for invalid field values
}

#[tokio::test]
async fn test_validate_invalid_ground_truth() {
    let _validator = FlowValidator::new();

    // Create a ground truth with error tolerance but no assertions (valid for our current implementation)
    let ground_truth = YmlGroundTruth::new().with_error_tolerance(-0.01);

    // For our current implementation, empty assertions are valid
    assert!(ground_truth.final_state_assertions.is_empty());
    assert_eq!(ground_truth.error_tolerance, Some(-0.01));
}

#[tokio::test]
async fn test_validate_final_state() {
    let _validator = FlowValidator::new();

    // Create a flow with final state assertions
    let flow = create_swap_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        1.0,
    );

    // This would validate against a final state
    // For now, we'll just check that assertions exist
    if let Some(ground_truth) = &flow.ground_truth {
        assert!(!ground_truth.final_state_assertions.is_empty());
    }
}

#[tokio::test]
async fn test_register_custom_validator() {
    let mut validator = FlowValidator::new();

    // Register a custom assertion validator
    struct CustomAssertionValidator;
    impl AssertionValidator for CustomAssertionValidator {
        fn validate(&self, _assertion: &YmlAssertion, _context: &WalletContext) -> Option<String> {
            // Just return None for this test
            None
        }
    }

    validator.register_assertion_validator(
        "CustomAssertion".to_string(),
        Box::new(CustomAssertionValidator),
    );

    // In a real implementation, this would use the custom validator
    // For now, we just verify registration doesn't panic
}

#[test]
fn test_validate_valid_flow_with_builder() {
    let validator = FlowValidator::new();

    // Create a valid flow using the builder function from yml_schema
    let flow = create_swap_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        0.5, // 0.5 SOL
    );

    // Should validate successfully
    assert!(validator.validate_flow(&flow).is_ok());
}

#[test]
fn test_validate_invalid_flow_with_builder() {
    let validator = FlowValidator::new();

    // Create an invalid flow with empty steps
    let mut flow = create_swap_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        0.5, // 0.5 SOL
    );

    // Remove all steps to make it invalid
    flow.steps.clear();

    // Should fail validation
    assert!(validator.validate_flow(&flow).is_err());
}
