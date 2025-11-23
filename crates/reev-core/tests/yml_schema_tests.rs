//! Tests for yml_schema module

use reev_core::yml_schema::{
    builders, YmlAssertion, YmlContext, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall,
    YmlWalletInfo,
};
use reev_types::benchmark::TokenBalance;
use reev_types::tools::ToolName;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_yml_flow_validation() {
    let wallet_info = YmlWalletInfo::new("test_pubkey".to_string(), 1_000_000_000) // 1 SOL
        .with_token(
            TokenBalance::new(
                "So11111111111111111111111111111111111111112".to_string(),
                1_000_000_000,
            )
            .with_decimals(9)
            .with_symbol("SOL".to_string()),
        )
        .with_total_value(150.0);

    let step = YmlStep::new(
        "swap".to_string(),
        "swap 1 SOL to USDC".to_string(),
        "Exchange 1 SOL for USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
    .with_critical(true)
    .with_estimated_time(30);

    let flow = YmlFlow::new(
        Uuid::now_v7().to_string(),
        "swap 1 SOL to USDC".to_string(),
        wallet_info,
    )
    .with_step(step);

    // Test flow validation
    let result = flow.validate();
    assert!(result.is_ok(), "Flow should be valid");
}

#[tokio::test]
async fn test_yml_flow_builder() {
    // Test using the builder functions
    let flow = builders::create_swap_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        1.0,
    );

    assert_eq!(flow.user_prompt, "swap 1 SOL to USDC");
    assert_eq!(flow.subject_wallet_info.pubkey, "test_pubkey");
    assert_eq!(flow.steps.len(), 1);
    assert_eq!(flow.steps[0].step_id, "swap");
    // Note: create_swap_flow builder doesn't add ground truth
    assert!(flow.ground_truth.is_none());
}

#[tokio::test]
async fn test_yml_lend_flow_builder() {
    // Test using the lend flow builder
    let flow = builders::create_lend_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "USDC".to_string(),
        100.0,
    );

    assert_eq!(flow.user_prompt, "lend 100 USDC to jupiter");
    assert_eq!(flow.subject_wallet_info.pubkey, "test_pubkey");
    assert_eq!(flow.steps.len(), 1);
    assert_eq!(flow.steps[0].step_id, "lend");
}

#[tokio::test]
async fn test_yml_swap_then_lend_flow_builder() {
    // Test using the swap then lend flow builder
    let flow = builders::create_swap_then_lend_flow(
        "test_pubkey".to_string(),
        1_000_000_000, // 1 SOL
        "SOL".to_string(),
        "USDC".to_string(),
        1.0,
    );

    assert_eq!(flow.user_prompt, "swap 1 SOL to USDC then lend");
    assert_eq!(flow.subject_wallet_info.pubkey, "test_pubkey");
    assert_eq!(flow.steps.len(), 2);
    assert_eq!(flow.steps[0].step_id, "swap");
    assert_eq!(flow.steps[1].step_id, "lend");
    assert!(flow.ground_truth.is_some());

    // Check ground truth has assertions
    let ground_truth = flow.ground_truth.unwrap();
    assert!(!ground_truth.final_state_assertions.is_empty());
}

#[tokio::test]
async fn test_yml_context() {
    // Test YML context functionality
    let context = YmlContext::new()
        .with_variable("SOL_PRICE".to_string(), json!(150.0))
        .with_variable("USDC_PRICE".to_string(), json!(1.0));

    assert_eq!(context.get_variable("SOL_PRICE"), Some(&json!(150.0)));
    assert_eq!(context.get_variable("USDC_PRICE"), Some(&json!(1.0)));
    assert_eq!(context.get_variable("UNKNOWN"), None);
}

#[tokio::test]
async fn test_yml_wallet_info() {
    // Test wallet info functionality
    let wallet_info = YmlWalletInfo::new("test_pubkey".to_string(), 1_000_000_000) // 1 SOL
        .with_token(
            TokenBalance::new(
                "So11111111111111111111111111111111111111112".to_string(),
                1_000_000_000,
            )
            .with_decimals(9)
            .with_symbol("SOL".to_string()),
        )
        .with_total_value(150.0);

    assert_eq!(wallet_info.pubkey, "test_pubkey");
    assert_eq!(wallet_info.lamports, 1_000_000_000);
    assert_eq!(wallet_info.sol_balance_sol(), 1.0);
    assert_eq!(wallet_info.tokens.len(), 1);
    assert_eq!(wallet_info.total_value_usd, Some(150.0));
}

#[tokio::test]
async fn test_yml_step() {
    // Test step functionality
    let step = YmlStep::new(
        "swap".to_string(),
        "swap 1 SOL to USDC".to_string(),
        "Exchange 1 SOL for USDC".to_string(),
    )
    .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
    .with_critical(false)
    .with_estimated_time(60);

    assert_eq!(step.step_id, "swap");
    assert_eq!(step.prompt, "swap 1 SOL to USDC");
    assert_eq!(step.context, "Exchange 1 SOL for USDC");
    assert_eq!(step.critical, Some(false));
    assert_eq!(step.estimated_time_seconds, Some(60));
    assert_eq!(step.expected_tool_calls.clone().unwrap().len(), 1);
    assert_eq!(
        step.expected_tool_calls.unwrap()[0].tool_name,
        ToolName::JupiterSwap
    );
}

#[tokio::test]
async fn test_yml_tool_call() {
    // Test tool call functionality
    let tool_call = YmlToolCall::new(ToolName::JupiterSwap, true)
        .with_parameter(
            "input_mint".to_string(),
            json!("So11111111111111111111111111111111111111112"),
        )
        .with_parameter(
            "output_mint".to_string(),
            json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        )
        .with_parameter("amount".to_string(), json!(1_000_000_000));

    assert_eq!(tool_call.tool_name, ToolName::JupiterSwap);
    assert!(tool_call.critical);
    assert_eq!(tool_call.expected_parameters.unwrap().len(), 3);
}

#[tokio::test]
async fn test_yml_ground_truth() {
    // Test ground truth functionality
    let ground_truth = YmlGroundTruth::new()
        .with_assertion(
            YmlAssertion::new("SolBalanceChange".to_string())
                .with_pubkey("test_pubkey".to_string())
                .with_expected_change_gte(-1_010_000_000.0),
        )
        .with_assertion(
            YmlAssertion::new("TokenBalanceChange".to_string())
                .with_pubkey("test_pubkey".to_string())
                .with_parameter(
                    "mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                )
                .with_expected_change_gte(0.0),
        )
        .with_error_tolerance(0.02);

    assert_eq!(ground_truth.final_state_assertions.len(), 2);
    assert_eq!(ground_truth.error_tolerance, Some(0.02));

    let first_assertion = &ground_truth.final_state_assertions[0];
    assert_eq!(first_assertion.assertion_type, "SolBalanceChange");
    assert_eq!(first_assertion.pubkey, Some("test_pubkey".to_string()));
    assert_eq!(first_assertion.expected_change_gte, Some(-1_010_000_000.0));
}
