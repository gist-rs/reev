//! Test for Multi-Step Flow Generation
//!
//! This test validates that the system can handle multi-step prompts like
//! "sell all SOL and lend to jup" by dynamically combining single-step flows.

use reev_core::planner::FlowPlanner;
use reev_orchestrator::gateway::OrchestratorGateway;
use reev_orchestrator::generators::yml_generator::YmlGenerator;
use reev_types::flow::{WalletContext, DynamicStep};
use reev_types::tools::ToolName;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing_subscriber;

#[tokio::test]
async fn test_multi_step_flow_generation() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;

    // Create test wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        reev_types::benchmark::TokenBalance {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 5_000_000_000, // 5 SOL
            decimals: 9,
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 5_000_000_000, // 5 SOL
        total_value_usd: 750.0, // $150 per SOL
        token_balances,
    };

    // Create YML generator
    let yml_generator = YmlGenerator::new();

    // Create a multi-step flow plan for "sell all SOL and lend to jup"
    let flow_id = format!("test-{}", uuid::Uuid::new_v4());
    let mut flow_plan = reev_types::flow::DynamicFlowPlan::new(
        flow_id.clone(),
        "sell all SOL and lend to jup".to_string(),
        wallet_context.clone(),
    );

    // Step 1: Swap ALL SOL to USDC (reserve 0.05 SOL for gas)
    let swap_amount_sol = 4.95; // 5 SOL - 0.05 SOL for gas
    let swap_step = DynamicStep::new(
        "step_1_swap".to_string(),
        format!(
            "Execute Jupiter swap: {} SOL → USDC for wallet {}. \
             Expected output: {:.2} USDC. \
             SOL price: ${:.2}.",
            swap_amount_sol,
            wallet_context.owner,
            swap_amount_sol * 150.0, // Expected USDC at $150 per SOL
            150.0
        ),
        "Jupiter DEX swap execution with detailed parameters".to_string(),
    )
    .with_tool(ToolName::JupiterSwap)
    .with_estimated_time(30)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(true);

    // Step 2: Lend the swapped USDC to Jupiter
    let lend_step = DynamicStep::new(
        "step_2_lend".to_string(),
        format!(
            "Deposit {} USDC into Jupiter lending for wallet {}. \
             Target APY: 8.5%, Market range: 5-12%. \
             Expected daily yield: ${:.4}.",
            swap_amount_sol * 150.0, // USDC amount from previous step
            wallet_context.owner,
            (swap_amount_sol * 150.0) * (8.5 / 100.0) / 365.0 // Daily yield
        ),
        "Deposit USDC into Jupiter lending".to_string(),
    )
    .with_tool(ToolName::JupiterLendEarnDeposit)
    .with_estimated_time(45)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(true);

    // Add steps to flow plan
    flow_plan = flow_plan.with_step(swap_step).with_step(lend_step);

    // Generate YML content
    let yml_content = yml_generator.generate_yml_content(&flow_plan)?;

    // Verify YML structure
    let yaml: serde_yaml::Value = serde_yaml::from_str(&yml_content)?;

    // Check that it's a multi-step flow
    let initial_state = yaml.get("initial_state").unwrap().as_sequence().unwrap();
    assert!(!initial_state.is_empty(), "Initial state should not be empty");

    // Check that the prompt is preserved
    let prompt = yaml.get("prompt").unwrap().as_str().unwrap();
    assert!(prompt.contains("swap"), "Prompt should mention swap");

    // Check ground truth
    let ground_truth = yaml.get("ground_truth").unwrap().as_mapping().unwrap();
    let final_state_assertions = ground_truth
        .get("final_state_assertions")
        .unwrap()
        .as_sequence()
        .unwrap();

    // Should have assertions for both swap and lend operations
    let mut has_swap_assertion = false;
    let mut has_lend_assertion = false;

    for assertion in final_state_assertions {
        let assertion_type = assertion
            .as_mapping()
            .unwrap()
            .get("type")
            .unwrap()
            .as_str()
            .unwrap();

        if assertion_type == "TokenAccountBalance" {
            has_swap_assertion = true;
        } else if assertion_type == "SolBalanceChange" {
            has_lend_assertion = true;
        }
    }

    assert!(has_swap_assertion, "Should have swap assertion");
    assert!(has_lend_assertion, "Should have lend assertion");

    // Print the generated YML for inspection
    println!("Generated YML:\n{}", yml_content);

    Ok(())
}

#[tokio::test]
async fn test_dynamic_flow_plan_with_steps() -> Result<(), Box<dyn std::error::Error>> {
    // Create a wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        reev_types::benchmark::TokenBalance {
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 10_000_000_000, // 10 SOL
            decimals: 9,
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 10_000_000_000, // 10 SOL
        total_value_usd: 1500.0, // $150 per SOL
        token_balances,
    };

    // Create a flow plan with multiple steps
    let flow_id = format!("test-{}", uuid::Uuid::new_v4());
    let flow_plan = reev_types::flow::DynamicFlowPlan::new(
        flow_id.clone(),
        "sell all SOL and lend to jup".to_string(),
        wallet_context.clone(),
    );

    // Verify flow plan structure
    assert_eq!(flow_plan.user_prompt, "sell all SOL and lend to jup");
    assert_eq!(flow_plan.flow_id, flow_id);
    assert_eq!(flow_plan.context.owner, "test_wallet");
    assert_eq!(flow_plan.context.sol_balance, 10_000_000_000);

    // Verify empty steps initially
    assert!(flow_plan.steps.is_empty());

    Ok(())
}
