//! Benchmark 300 Test: Swap SOL then Multiply USDC
//!
//! Tests the multiplication strategy where agent uses 50% of SOL to achieve 1.5x USDC increase.
//! This is the foundational benchmark that establishes the core 300-series capabilities.

use crate::tests::dynamic_flow::300_series::{create_test_wallet_context, load_benchmark_yaml, TestUtils};
use reev_orchestrator::OrchestratorGateway;
use reev_types::flow::WalletContext;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::test]
async fn test_benchmark_300_multiplication_strategy() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Multiplication Strategy");

    let utils = TestUtils::new();
    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Create test wallet context matching benchmark initial state
    let context = create_test_wallet_context(4, 20); // 4 SOL, 20 USDC

    println!("💰 Initial state: {} SOL, {} USDC",
        context.sol_balance_sol(),
        context.token_balances.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap_or(&0) / 1_000_000);

    // Generate flow plan from prompt
    let prompt = benchmark.get("prompt").unwrap().as_str().unwrap();
    let flow_plan = utils.gateway.generate_flow_plan(prompt, &context, None)?;

    println!("📋 Generated flow plan: {} steps", flow_plan.steps.len());

    // Validate flow plan structure
    assert_eq!(flow_plan.steps.len(), 2, "Should have swap and lend steps");
    assert_eq!(flow_plan.steps[0].step_id, "swap_1");
    assert_eq!(flow_plan.steps[1].step_id, "lend_1");

    // Validate tool calls in flow steps
    let swap_tools = &flow_plan.steps[0].required_tools;
    let lend_tools = &flow_plan.steps[1].required_tools;

    assert!(swap_tools.contains(&"sol_tool".to_string()), "Swap step should use sol_tool");
    assert!(lend_tools.contains(&"jupiter_earn_tool".to_string()), "Lend step should use jupiter_earn_tool");

    // Validate prompt contains percentage and multiplication
    let swap_prompt = &flow_plan.steps[0].prompt_template;
    assert!(swap_prompt.contains("2"), "Should use 50% of 4 SOL = 2 SOL");
    assert!(swap_prompt.contains("SOL"), "Should mention SOL");

    let lend_prompt = &flow_plan.steps[1].prompt_template;
    assert!(lend_prompt.contains("USDC"), "Should lend USDC");

    println!("✅ Flow plan validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_percentage_calculation() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Percentage Calculation");

    let utils = TestUtils::new();
    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Test with different wallet sizes
    let test_cases = vec![
        (4, 2, "50% of 4 SOL = 2 SOL"),
        (6, 3, "50% of 6 SOL = 3 SOL"),
        (10, 5, "50% of 10 SOL = 5 SOL"),
    ];

    for (initial_sol, expected_used, description) in test_cases {
        println!("Testing: {}", description);

        let context = create_test_wallet_context(initial_sol, 20);
        let prompt = benchmark.get("prompt").unwrap().as_str().unwrap();
        let flow_plan = utils.gateway.generate_flow_plan(prompt, &context, None)?;

        // Extract swap amount from prompt (simplified - in real test would parse more carefully)
        let swap_prompt = &flow_plan.steps[0].prompt_template;
        let (is_accurate, actual_percentage) = utils.validate_percentage_calculation(prompt, initial_sol, expected_used);

        assert!(is_accurate, "Percentage calculation should be accurate: {}", description);
        println!("  ✅ Used {}% of SOL (expected 50%)", actual_percentage);
    }

    println!("✅ Percentage calculation validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_multiplication_achievement() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Multiplication Achievement");

    let utils = TestUtils::new();

    // Test multiplication target achievement
    let test_cases = vec![
        (20, 30, 1.5, 0.2, "20 USDC → 30 USDC (1.5x)"),
        (25, 40, 1.5, 0.2, "25 USDC → 40 USDC (1.5x)"),
        (30, 45, 1.5, 0.2, "30 USDC → 45 USDC (1.5x)"),
    ];

    for (initial_usdc, final_usdc, target_multiplier, tolerance, description) in test_cases {
        println!("Testing: {}", description);

        let (achieved, actual_multiplier) = utils.validate_multiplication_achievement(
            initial_usdc, final_usdc, target_multiplier, tolerance
        );

        assert!(achieved, "Multiplication target should be achieved: {}", description);
        println!("  ✅ Achieved {:.2}x multiplication (target {:.1}x)", actual_multiplier, target_multiplier);
    }

    println!("✅ Multiplication achievement validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_tool_call_expectations() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Tool Call Expectations");

    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");
    let utils = TestUtils::new();

    // Extract expected tools from benchmark
    let expected_tools: Vec<String> = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("expected_tool_calls"))
        .and_then(|etc| etc.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|tool| tool.get("tool_name"))
                .filter_map(|name| name.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    println!("Expected tools: {:?}", expected_tools);

    // Expected tools for benchmark 300
    let expected_core_tools = vec![
        "account_balance".to_string(),
        "jupiter_swap".to_string(),
        "jupiter_lend".to_string(),
        "jupiter_positions".to_string(),
    ];

    // Validate that expected tools match core expectations
    for expected_tool in &expected_core_tools {
        assert!(expected_tools.contains(expected_tool),
               "Should expect {} tool", expected_tool);
    }

    // Test tool validation with simulated execution
    let executed_tools = vec![
        "account_balance".to_string(),
        "jupiter_swap".to_string(),
        "jupiter_lend".to_string(),
        "jupiter_positions".to_string(),
    ];

    let tool_calls_valid = utils.validate_tool_calls(&benchmark, &executed_tools);
    assert!(tool_calls_valid, "All expected tools should be called");

    // Test with missing critical tool
    let incomplete_tools = vec![
        "account_balance".to_string(),
        "jupiter_lend".to_string(),
    ];

    let incomplete_valid = utils.validate_tool_calls(&benchmark, &incomplete_tools);
    assert!(!incomplete_valid, "Should fail when critical tools are missing");

    println!("✅ Tool call expectations validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_flow_complexity() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Flow Complexity");

    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");
    let utils = TestUtils::new();

    // Test tool sequence complexity
    let executed_sequence = vec![
        "account_balance".to_string(),
        "jupiter_swap".to_string(),
        "jupiter_lend".to_string(),
        "jupiter_positions".to_string(),
    ];

    let (meets_minimum, actual_steps) = utils.validate_tool_sequence(&benchmark, &executed_sequence);
    assert!(meets_minimum, "Should meet minimum step requirements");
    assert_eq!(actual_steps, 4, "Should have 4 tool calls");

    // Test with insufficient steps
    let short_sequence = vec![
        "jupiter_swap".to_string(),
        "jupiter_lend".to_string(),
    ];

    let (short_meets_minimum, short_steps) = utils.validate_tool_sequence(&benchmark, &short_sequence);
    assert!(!short_meets_minimum, "Should fail with insufficient steps");
    assert_eq!(short_steps, 2, "Should count actual steps correctly");

    println!("✅ Flow complexity validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_end_to_end_flow_generation() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: End-to-End Flow Generation");

    let utils = TestUtils::new();
    let gateway = OrchestratorGateway::new();

    let user_prompt = "use my 50% sol to multiply usdc 1.5x on jup";
    let wallet_pubkey = "test_wallet_benchmark_300";

    // Process full user request
    let (flow_plan, yml_path) = gateway
        .process_user_request(user_prompt, wallet_pubkey)
        .await?;

    println!("📋 Generated flow: {}", flow_plan.flow_id);
    println!("📄 YML file: {}", yml_path);

    // Validate flow plan
    assert_eq!(flow_plan.user_prompt, user_prompt);
    assert_eq!(flow_plan.context.owner, wallet_pubkey);
    assert_eq!(flow_plan.steps.len(), 2); // swap + lend

    // Validate YML file was generated and contains expected structure
    assert!(std::path::Path::new(&yml_path).exists());

    let yml_content = std::fs::read_to_string(&yml_path)?;
    assert!(yml_content.contains("id"));
    assert!(yml_content.contains("description"));
    assert!(yml_content.contains("tags"));
    assert!(yml_content.contains("initial_state"));
    assert!(yml_content.contains("prompt"));
    assert!(yml_content.contains("ground_truth"));

    // Validate YML content contains the prompt and context
    assert!(yml_content.contains(wallet_pubkey));
    assert!(yml_content.contains("SOL"));
    assert!(yml_content.contains("USDC"));

    // Clean up
    std::fs::remove_file(yml_path)?;
    gateway.cleanup().await?;

    println!("✅ End-to-end flow generation validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_critical_step_handling() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Critical Step Handling");

    let utils = TestUtils::new();
    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Extract critical tools from benchmark
    let critical_tools: Vec<String> = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("expected_tool_calls"))
        .and_then(|etc| etc.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter(|tool| {
                    tool.get("critical")
                        .and_then(|c| c.as_bool())
                        .unwrap_or(false)
                })
                .filter_map(|tool| tool.get("tool_name"))
                .filter_map(|name| name.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    println!("Critical tools: {:?}", critical_tools);

    // Expected critical tools for benchmark 300
    let expected_critical = vec![
        "jupiter_swap".to_string(),
        "jupiter_lend".to_string(),
    ];

    for tool in &expected_critical {
        assert!(critical_tools.contains(tool),
               "{} should be marked as critical", tool);
    }

    // Test atomic mode enforcement
    let context = create_test_wallet_context(4, 20);
    let prompt = benchmark.get("prompt").unwrap().as_str().unwrap();

    // Generate flow with Strict atomic mode
    let flow_plan = utils.gateway.generate_flow_plan(prompt, &context, None)?;

    // All steps should be critical by default in multiplication strategy
    for step in &flow_plan.steps {
        assert!(step.critical, "All steps in multiplication strategy should be critical");
    }

    println!("✅ Critical step handling validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_success_criteria() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Success Criteria");

    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Extract success criteria from benchmark
    let success_criteria = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("success_criteria"))
        .and_then(|sc| sc.as_sequence())
        .unwrap_or(&serde_json::Value::Null);

    let expected_criteria = vec![
        "percentage_calculation",
        "multiplication_strategy",
        "tool_coordination",
        "yield_generation",
    ];

    println!("Success criteria: {:?}", success_criteria);

    for criterion_name in &expected_criteria {
        let found = success_criteria
            .as_sequence()
            .unwrap_or(&serde_json::json!([]))
            .iter()
            .any(|criterion| {
                criterion.get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == criterion_name)
                    .unwrap_or(false)
            });

        assert!(found, "Should have {} success criteria", criterion_name);
    }

    // Validate each criterion is marked as required
    for criterion in success_criteria.as_sequence().unwrap_or(&serde_json::json!([])) {
        let required = criterion
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);

        assert!(required, "All success criteria should be required");
    }

    println!("✅ Success criteria validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_expected_data_structure() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: Expected Data Structure");

    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Extract expected data structure from benchmark
    let expected_data = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("expected_data_structure"))
        .and_then(|eds| eds.as_sequence())
        .unwrap_or(&serde_json::Value::Null);

    let expected_paths = vec![
        "$.result.data.wallet_context",
        "$.result.data.swap_execution",
        "$.result.data.lend_execution",
        "$.result.data.final_state",
        "$.result.data.tool_calls",
    ];

    println!("Expected data paths: {:?}", expected_paths);

    for expected_path in &expected_paths {
        let found = expected_data
            .as_sequence()
            .unwrap_or(&serde_json::json!([]))
            .iter()
            .any(|data| {
                data.get("path")
                    .and_then(|p| p.as_str())
                    .map(|p| p == expected_path)
                    .unwrap_or(false)
            });

        assert!(found, "Should expect {} data structure", expected_path);
    }

    // Validate weight distribution
    let total_weight: f64 = expected_data
        .as_sequence()
        .unwrap_or(&serde_json::json!([]))
        .iter()
        .map(|data| {
            data.get("weight")
                .and_then(|w| w.as_f64())
                .unwrap_or(0.0)
        })
        .sum();

    assert!((total_weight - 1.0).abs() < 0.01,
           "Total weight should sum to 1.0, got {}", total_weight);

    println!("✅ Expected data structure validation passed");
    Ok(())
}

#[tokio::test]
async fn test_benchmark_300_otel_tracking_expectations() -> anyhow::Result<()> {
    println!("🎯 Testing Benchmark 300: OpenTelemetry Tracking Expectations");

    let benchmark = load_benchmark_yaml("300-jup-swap-then-lend-deposit-dyn");

    // Extract expected OTEL tracking from benchmark
    let expected_otel = benchmark
        .get("ground_truth")
        .and_then(|gt| gt.get("expected_otel_tracking"))
        .and_then(|eot| eot.as_sequence())
        .unwrap_or(&serde_json::Value::Null);

    let expected_tracking_types = vec![
        "tool_call_logging",
        "execution_tracing",
        "mermaid_generation",
        "performance_metrics",
    ];

    println!("Expected OTEL tracking types: {:?}", expected_tracking_types);

    for tracking_type in &expected_tracking_types {
        let found = expected_otel
            .as_sequence()
            .unwrap_or(&serde_json::json!([]))
            .iter()
            .any(|tracking| {
                tracking.get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == tracking_type)
                    .unwrap_or(false)
            });

        assert!(found, "Should expect {} OTEL tracking", tracking_type);
    }

    // Validate required tools for tool call logging
    let tool_logging = expected_otel
        .as_sequence()
        .unwrap_or(&serde_json::json!([]))
        .iter()
        .find(|tracking| {
            tracking.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "tool_call_logging")
                .unwrap_or(false)
        });

    if let Some(logging) = tool_logging {
        let required_tools = logging
            .get("required_tools")
            .and_then(|rt| rt.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|tool| tool.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let expected_tools = vec![
            "account_balance",
            "jupiter_swap",
            "jupiter_lend",
            "jupiter_positions",
        ];

        for tool in &expected_tools {
            assert!(required_tools.contains(&tool),
                   "Should require {} tool in OTEL logging", tool);
        }
    }

    println!("✅ OpenTelemetry tracking expectations validation passed");
    Ok(())
}
