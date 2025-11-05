//! Enhanced 300-Series Flow Test
//!
//! Tests for enhanced flow generation with detailed visualization
//! and comprehensive step information as described in Issue #23.

use reev_orchestrator::gateway::OrchestratorGateway;
use reev_types::flow::{AtomicMode, DynamicFlowPlan};
use reev_types::wallet::WalletContext;
use serde_json::json;
use std::collections::HashMap;
use tokio_test;

/// Create a mock wallet context for testing
fn create_mock_wallet_context() -> WalletContext {
    let mut token_balances = HashMap::new();

    // SOL balance: 10 SOL
    token_balances.insert(
        "So11111111111111111111111111111111111111112".to_string(),
        reev_types::wallet::TokenBalance {
            pubkey: "So11111111111111111111111111111111111111112".to_string(),
            owner: "test_wallet_owner".to_string(),
            balance: 10_000_000_000, // 10 SOL
            decimals: Some(9),
            symbol: Some("SOL".to_string()),
        },
    );

    // USDC balance: 1000 USDC
    token_balances.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        reev_types::wallet::TokenBalance {
            pubkey: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            owner: "test_wallet_owner".to_string(),
            balance: 1_000_000_000, // 1000 USDC
            decimals: Some(6),
            symbol: Some("USDC".to_string()),
        },
    );

    let mut token_prices = HashMap::new();
    token_prices.insert("So11111111111111111111111111111111111111112".to_string(), 150.0);
    token_prices.insert("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), 1.0);

    WalletContext {
        owner: "test_wallet_owner".to_string(),
        sol_balance: 10_000_000_000, // 10 SOL
        token_balances,
        token_prices,
        total_value_usd: 2500.0, // 10 SOL * $150 + 1000 USDC
        network: "mainnet-beta".to_string(),
    }
}

#[tokio::test]
async fn test_enhanced_300_flow_generation() {
    // Setup
    let gateway = OrchestratorGateway::new().await.unwrap();
    let context = create_mock_wallet_context();
    let prompt = "use my 50% sol to multiply usdc 1.5x on jupiter";

    // Execute enhanced flow generation
    let result = gateway.generate_enhanced_300_flow(prompt, &context);

    assert!(result.is_ok(), "Enhanced 300 flow generation should succeed");

    let flow_plan = result.unwrap();

    // Verify flow structure
    assert_eq!(flow_plan.steps.len(), 5, "Enhanced 300 flow should have 5 steps");
    assert!(flow_plan.flow_id.starts_with("enhanced-300-"), "Flow ID should start with 'enhanced-300-'");
    assert_eq!(flow_plan.atomic_mode, AtomicMode::Strict, "Atomic mode should be Strict");

    // Verify step names
    let step_names: Vec<String> = flow_plan.steps.iter()
        .map(|step| step.name.clone())
        .collect();

    assert!(step_names[0].contains("balance_check"), "First step should be balance check");
    assert!(step_names[1].contains("calculation"), "Second step should be calculation");
    assert!(step_names[2].contains("swap"), "Third step should be swap");
    assert!(step_names[3].contains("lend"), "Fourth step should be lend");
    assert!(step_names[4].contains("check"), "Fifth step should be position check");

    // Verify enhanced step details
    let balance_check = &flow_plan.steps[0];
    assert!(balance_check.description.contains("portfolio assessment"),
           "Balance check should mention portfolio assessment");
    assert!(balance_check.prompt.contains("test_wallet_owner"),
           "Balance check should include wallet pubkey");
    assert!(balance_check.prompt.contains("10.000000"),
           "Balance check should include SOL balance");

    let calculation = &flow_plan.steps[1];
    assert!(calculation.description.contains("strategy calculation"),
           "Calculation step should mention strategy calculation");
    assert!(calculation.prompt.contains("1.5x"),
           "Calculation should include target multiplier");
    assert!(calculation.prompt.contains("5.000000"),
           "Calculation should include available SOL amount");

    let swap = &flow_plan.steps[2];
    assert!(swap.description.contains("detailed parameters"),
           "Swap step should mention detailed parameters");
    assert!(swap.prompt.contains("5.000000"),
           "Swap should include SOL amount");
    assert!(swap.prompt.contains("750.00"),
           "Swap should include estimated USDC output");

    let lend = &flow_plan.steps[3];
    assert!(lend.description.contains("detailed parameters"),
           "Lend step should mention detailed parameters");
    assert!(lend.prompt.contains("750.00"),
           "Lend should include USDC amount");
    assert!(lend.prompt.contains("8.5%"),
           "Lend should include target APY");

    // Verify recovery strategies
    assert!(balance_check.recovery.is_some(), "Balance check should have recovery strategy");
    assert!(swap.recovery.is_some(), "Swap step should have recovery strategy");
    assert!(lend.recovery.is_some(), "Lend step should have recovery strategy");
    assert!(lend.is_critical, "Lend step should be critical");

    println!("✅ Enhanced 300 flow generation test passed");
    println!("Flow ID: {}", flow_plan.flow_id);
    println!("Steps: {:?}", step_names);
}

#[tokio::test]
async fn test_enhanced_flow_plan_integration() {
    // Setup
    let gateway = OrchestratorGateway::new().await.unwrap();
    let context = create_mock_wallet_context();

    // Test different prompt variations that should trigger enhanced flows
    let test_cases = vec![
        ("use my 50% sol to multiply usdc 1.5x on jupiter", true),
        ("swap 1 SOL for USDC with yield", false),
        ("multiply my SOL position 2x using Jupiter", true),
        ("lend my USDC on Jupiter", false),
    ];

    for (prompt, should_be_enhanced) in test_cases {
        let result = gateway.generate_enhanced_flow_plan(prompt, &context, None);

        assert!(result.is_ok(), "Flow generation should succeed for prompt: {}", prompt);

        let flow_plan = result.unwrap();

        if should_be_enhanced {
            assert!(flow_plan.flow_id.starts_with("enhanced-300-"),
                   "Enhanced flow should have enhanced-300 prefix for: {}", prompt);
            assert_eq!(flow_plan.steps.len(), 5,
                      "Enhanced flow should have 5 steps for: {}", prompt);
        } else {
            assert!(!flow_plan.flow_id.starts_with("enhanced-300-"),
                   "Standard flow should not have enhanced-300 prefix for: {}", prompt);
        }

        println!("✅ Integration test passed for: {} -> {}", prompt, flow_plan.flow_id);
    }
}

#[tokio::test]
async fn test_enhanced_step_detailed_prompts() {
    // Setup
    let gateway = OrchestratorGateway::new().await.unwrap();
    let context = create_mock_wallet_context();
    let prompt = "use my 75% sol to multiply usdc 2x on jupiter";

    // Generate enhanced flow
    let flow_plan = gateway.generate_enhanced_300_flow(prompt, &context).unwrap();

    // Verify detailed prompts contain critical information
    for (index, step) in flow_plan.steps.iter().enumerate() {
        match index {
            0 => { // Balance check
                assert!(step.prompt.contains("test_wallet_owner"), "Step 0 should contain wallet pubkey");
                assert!(step.prompt.contains("10.000000"), "Step 0 should contain SOL balance");
                assert!(step.prompt.contains("2500.00"), "Step 0 should contain total value");
                assert!(step.prompt.contains("5.000000"), "Step 0 should contain available SOL (50%)");
            },
            1 => { // Calculation
                assert!(step.prompt.contains("2.0"), "Step 1 should contain target multiplier");
                assert!(step.prompt.contains("7.500000"), "Step 1 should contain available SOL (75%)");
                assert!(step.prompt.contains("1125.00"), "Step 1 should contain estimated USDC");
                assert!(step.prompt.contains("2250.00"), "Step 1 should contain target USDC");
            },
            2 => { // Swap
                assert!(step.prompt.contains("7.500000"), "Step 2 should contain swap amount");
                assert!(step.prompt.contains("1125.00"), "Step 2 should contain expected output");
                assert!(step.prompt.contains("1091.25"), "Step 2 should contain minimum received");
                assert!(step.prompt.contains("7500000000"), "Step 2 should contain lamports");
            },
            3 => { // Lend
                assert!(step.prompt.contains("1125.00"), "Step 3 should contain lend amount");
                assert!(step.prompt.contains("0.2623"), "Step 3 should contain daily yield");
                assert!(step.prompt.contains("1125000000"), "Step 3 should contain micro-USDC");
            },
            4 => { // Position check
                assert!(step.description.contains("final positions"),
                       "Step 4 should mention final positions");
            },
            _ => {}
        }
    }

    println!("✅ Enhanced step detailed prompts test passed");
}
