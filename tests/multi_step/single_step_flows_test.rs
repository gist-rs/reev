//! Test Single Step Flows for Multi-Step Flow Generation
//!
//! This test validates that single-step flows are properly combined
//! for multi-step flows like "sell all SOL and lend to jup".

use reev_core::planner::FlowPlanner;
use reev_core::yml_generator::flow_builders::{
    build_swap_flow, build_lend_flow, build_transfer_flow,
};
use reev_core::yml_generator::operation_types::{
    is_sell_all, LendParams, SwapParams, TransferParams,
};
use reev_core::yml_schema::{YmlFlow, YmlStep};
use reev_orchestrator::gateway::OrchestratorGateway;
use reev_orchestrator::generators::yml_generator::YmlGenerator;
use reev_types::flow::{WalletContext, DynamicFlowPlan, DynamicStep};
use reev_types::tools::ToolName;
use std::collections::HashMap;
use tempfile::TempDir;
use tracing_subscriber;

#[tokio::test]
async fn test_single_step_swap_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create test wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        reev_types::benchmark::TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: 100_000_000, // 100 USDC
            decimals: 6,
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 5_000_000_000, // 5 SOL
        total_value_usd: 750.0, // $150 per SOL
        token_balances,
    };

    // Create a refined prompt for swap
    let refined_prompt = reev_core::refiner::RefinedPrompt {
        original: "sell all SOL for USDC".to_string(),
        refined: "swap 4.95 SOL to USDC".to_string(), // After accounting for gas
        confidence: 0.9,
    };

    // Create swap parameters
    let swap_params = SwapParams {
        from_token: "SOL".to_string(),
        to_token: "USDC".to_string(),
        amount: 4.95, // 5 SOL - 0.05 SOL for gas
    };

    // Build swap flow
    let swap_flow = build_swap_flow(&refined_prompt, &wallet_context, swap_params).await?;

    // Verify single step swap flow structure
    assert_eq!(swap_flow.steps.len(), 1);
    assert_eq!(swap_flow.user_prompt, "sell all SOL for USDC");
    assert_eq!(swap_flow.refined_prompt, "swap 4.95 SOL to USDC");

    let swap_step = &swap_flow.steps[0];
    assert_eq!(swap_step.step_id, "swap");
    assert!(swap_step.refined_prompt.contains("swap"));
    assert!(swap_step.refined_prompt.contains("4.95"));
    assert!(swap_step.refined_prompt.contains("SOL"));
    assert!(swap_step.refined_prompt.contains("USDC"));

    // Verify expected tool
    assert!(swap_step.expected_tools.is_some());
    let tools = swap_step.expected_tools.as_ref().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0], ToolName::JupiterSwap);

    println!("Single-step swap flow created successfully:");
    println!("  Step ID: {}", swap_step.step_id);
    println!("  Refined Prompt: {}", swap_step.refined_prompt);

    Ok(())
}

#[tokio::test]
async fn test_single_step_lend_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create test wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        reev_types::benchmark::TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: 100_000_000, // 100 USDC
            decimals: 6,
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 5_000_000_000, // 5 SOL
        total_value_usd: 750.0, // $150 per SOL
        token_balances,
    };

    // Create a refined prompt for lend
    let refined_prompt = reev_core::refiner::RefinedPrompt {
        original: "lend USDC to jupiter".to_string(),
        refined: "lend 700 USDC to jupiter".to_string(),
        confidence: 0.9,
    };

    // Create lend parameters
    let lend_params = LendParams {
        token: "USDC".to_string(),
        amount: 700.0,
    };

    // Build lend flow
    let lend_flow = build_lend_flow(&refined_prompt, &wallet_context, lend_params).await?;

    // Verify single step lend flow structure
    assert_eq!(lend_flow.steps.len(), 1);
    assert_eq!(lend_flow.user_prompt, "lend USDC to jupiter");
    assert_eq!(lend_flow.refined_prompt, "lend 700 USDC to jupiter");

    let lend_step = &lend_flow.steps[0];
    assert_eq!(lend_step.step_id, "lend");
    assert!(lend_step.refined_prompt.contains("lend"));
    assert!(lend_step.refined_prompt.contains("700"));
    assert!(lend_step.refined_prompt.contains("USDC"));

    // Verify expected tool
    assert!(lend_step.expected_tools.is_some());
    let tools = lend_step.expected_tools.as_ref().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0], ToolName::JupiterLendEarnDeposit);

    println!("Single-step lend flow created successfully:");
    println!("  Step ID: {}", lend_step.step_id);
    println!("  Refined Prompt: {}", lend_step.refined_prompt);

    Ok(())
}

#[tokio::test]
async fn test_combine_single_step_flows_into_multi_step_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create test wallet context
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        reev_types::benchmark::TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: 100_000_000, // 100 USDC
            decimals: 6,
        },
    );

    let wallet_context = WalletContext {
        owner: "test_wallet".to_string(),
        sol_balance: 5_000_000_000, // 5 SOL
        total_value_usd: 750.0, // $150 per SOL
        token_balances,
    };

    // Create a multi-step refined prompt (simulating LLM processing)
    let refined_prompt = reev_core::refiner::RefinedPrompt {
        original: "sell all SOL and lend to jup".to_string(),
        refined: "swap 4.95 SOL to USDC then lend the resulting USDC".to_string(),
        confidence: 0.9,
    };

    // Create swap parameters
    let swap_params = SwapParams {
        from_token: "SOL".to_string(),
        to_token: "USDC".to_string(),
        amount: 4.95, // 5 SOL - 0.05 SOL for gas
    };

    // Create lend parameters (using expected output from swap)
    let lend_params = LendParams {
        token: "USDC".to_string(),
        amount: 4.95 * 150.0, // Approximate USDC amount at $150 per SOL
    };

    // Build individual step flows
    let swap_flow = build_swap_flow(&refined_prompt, &wallet_context, swap_params).await?;
    let lend_flow = build_lend_flow(&refined_prompt, &wallet_context, lend_params).await?;

    // Verify individual flows
    assert_eq!(swap_flow.steps.len(), 1);
    assert_eq!(lend_flow.steps.len(), 1);

    // Create a multi-step flow by combining flows
    let flow_id = format!("multi-step-{}", uuid::Uuid::new_v4());
    let mut multi_step_flow = YmlFlow::new(
        flow_id,
        "sell all SOL and lend to jup".to_string(),
        swap_flow.subject_wallet_info.clone(),
    )
    .with_refined_prompt(refined_prompt.refined);

    // Combine steps with proper IDs and context
    let swap_step = YmlStep::new(
        format!("step_1_{}", swap_flow.steps[0].step_id),
        swap_flow.steps[0].refined_prompt.clone(),
        "Swap SOL to USDC using Jupiter".to_string(),
    )
    .with_tool_call(reev_types::tools::YmlToolCall {
        tool_name: ToolName::JupiterSwap,
        critical: true,
    })
    .with_refined_prompt(swap_flow.steps[0].refined_prompt.clone())
    .with_critical(true);

    // Add context from previous step for lend
    let lend_context = format!(
        "After swapping 4.95 SOL to USDC, you now have approximately {} USDC",
        lend_params.amount
    );

    let lend_step = YmlStep::new(
        format!("step_2_{}", lend_flow.steps[0].step_id),
        lend_flow.steps[0].refined_prompt.clone(),
        lend_context,
    )
    .with_tool_call(reev_types::tools::YmlToolCall {
        tool_name: ToolName::JupiterLendEarnDeposit,
        critical: true,
    })
    .with_refined_prompt(lend_flow.steps[0].refined_prompt.clone())
    .with_critical(true);

    // Add steps to multi-step flow
    multi_step_flow.steps.push(swap_step);
    multi_step_flow.steps.push(lend_step);

    // Verify multi-step flow structure
    assert_eq!(multi_step_flow.steps.len(), 2);
    assert_eq!(multi_step_flow.user_prompt, "sell all SOL and lend to jup");
    assert_eq!(multi_step_flow.refined_prompt, "swap 4.95 SOL to USDC then lend the resulting USDC");

    // Verify first step (swap)
    let first_step = &multi_step_flow.steps[0];
    assert!(first_step.step_id.starts_with("step_1_"));
    assert!(first_step.refined_prompt.contains("swap"));
    assert!(first_step.refined_prompt.contains("4.95"));

    // Verify second step (lend)
    let second_step = &multi_step_flow.steps[1];
    assert!(second_step.step_id.starts_with("step_2_"));
    assert!(second_step.refined_prompt.contains("lend"));
    assert!(second_step.context.contains("After swapping"));

    // Create a temporary directory for YML file
    let temp_dir = TempDir::new()?;
    let yml_path = temp_dir.path().join("multi_step_flow.yml");

    // Write YML to file
    let yml_content = serde_yaml::to_string(&multi_step_flow)?;
    std::fs::write(&yml_path, yml_content)?;

    // Verify YML file exists
    assert!(yml_path.exists());

    println!("Multi-step flow created successfully by combining single-step flows:");
    println!("  Flow ID: {}", multi_step_flow.flow_id);
    println!("  Steps: {}", multi_step_flow.steps.len());
    println!("  Step 1: {} -> {}", first_step.step_id, first_step.refined_prompt);
    println!("  Step 2: {} -> {}", second_step.step_id, second_step.refined_prompt);
    println!("  YML Path: {}", yml_path.display());

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_handles_multi_step_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a temporary directory
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

    // Create a dynamic flow plan with multiple steps
    let flow_id = format!("test-{}", uuid::Uuid::new_v4());
    let mut flow_plan = DynamicFlowPlan::new(
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
             Previous step: Swapped {} SOL to USDC.",
            swap_amount_sol * 150.0, // USDC amount from previous step
            wallet_context.owner,
            swap_amount_sol
        ),
        "Deposit USDC into Jupiter lending".to_string(),
    )
    .with_tool(ToolName::JupiterLendEarnDeposit)
    .with_estimated_time(45)
    .with_recovery(reev_types::flow::RecoveryStrategy::Retry { attempts: 2 })
    .with_critical(true);

    // Add steps to flow plan
    flow_plan = flow_plan.with_step(swap_step).with_step(lend_step);

    // Verify flow plan
    assert_eq!(flow_plan.steps.len(), 2);
    assert_eq!(flow_plan.user_prompt, "sell all SOL and lend to jup");

    // Create YML generator
    let yml_generator = YmlGenerator::new();

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

    println!("Orchestrator multi-step flow test passed:");
    println!("  Flow ID: {}", flow_plan.flow_id);
    println!("  Steps: {}", flow_plan.steps.len());
    println!("  Step 1: {}", flow_plan.steps[0].step_id);
    println!("  Step 2: {}", flow_plan.steps[1].step_id);

    Ok(())
}
