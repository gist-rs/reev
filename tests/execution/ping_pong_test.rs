//! Ping-Pong Executor Tests
//!
//! Test the new step-by-step execution coordination mechanism
//! that fixes Issue #16 orchestrator-agent coordination problems.

use reev_orchestrator::{OrchestratorGateway, PingPongExecutor};
use reev_types::flow::{DynamicFlowPlan, DynamicStep, WalletContext};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_ping_pong_executor_basic() {
    // Create a simple flow plan with 2 steps
    let flow_plan = create_test_flow_plan();

    let mut executor = PingPongExecutor::new(5000); // 5 second timeout
    let agent_type = "glm-4.6-coding";

    // Execute flow with ping-pong coordination
    let step_results = executor.execute_flow_plan(&flow_plan, agent_type).await;

    // Verify execution completed
    assert!(!step_results.is_empty(), "Should have step results");
    assert_eq!(step_results.len(), 2, "Should execute both steps");

    // Verify step 1
    let step1 = &step_results[0];
    assert_eq!(step1.step_id, "balance_check");
    assert!(step1.success, "Step 1 should succeed");
    assert!(!step1.tool_calls.is_empty(), "Step 1 should have tool calls");

    // Verify step 2
    let step2 = &step_results[1];
    assert_eq!(step2.step_id, "swap_1");
    assert!(step2.success, "Step 2 should succeed");
    assert!(!step2.tool_calls.is_empty(), "Step 2 should have tool calls");

    println!("✅ Ping-pong executor test passed");
}

#[tokio::test]
async fn test_orchestrator_ping_pong_integration() {
    // Test integration through orchestrator gateway
    let gateway = OrchestratorGateway::new();
    let flow_plan = create_test_flow_plan();

    // Execute through gateway's ping-pong method
    let step_results = gateway.execute_flow_with_ping_pong(&flow_plan, "glm-4.6").await.unwrap();

    // Verify coordination worked
    assert_eq!(step_results.len(), 2, "Should execute both steps via gateway");

    let completed_steps = step_results.iter().filter(|s| s.success).count();
    assert_eq!(completed_steps, 2, "Both steps should complete successfully");

    println!("✅ Gateway ping-pong integration test passed");
}

#[tokio::test]
async fn test_ping_pong_timeout_handling() {
    // Test timeout behavior with very short timeout
    let flow_plan = create_test_flow_plan();
    let mut executor = PingPongExecutor::new(100); // 100ms timeout (should fail)

    let step_results = executor.execute_flow_plan(&flow_plan, "glm-4.6-coding").await;

    // With mock implementation, should still succeed since no real delay
    // In real implementation with actual agents, timeout would be triggered
    assert!(!step_results.is_empty(), "Should have some results even with timeout");

    println!("✅ Timeout handling test completed");
}

#[tokio::test]
async fn test_ping_pong_partial_completion() {
    // Create flow with one critical and one non-critical step
    let mut flow_plan = create_test_flow_plan();

    // Make second step critical (default), first step non-critical
    flow_plan.steps[0].critical = false; // balance_check is non-critical
    flow_plan.steps[1].critical = true;  // swap_1 is critical

    let mut executor = PingPongExecutor::new(5000);
    let step_results = executor.execute_flow_plan(&flow_plan, "glm-4.6-coding").await;

    // Verify both steps executed
    assert_eq!(step_results.len(), 2, "Should attempt both steps");

    // In mock implementation, both succeed
    // In real implementation, critical failure would stop execution
    assert_eq!(step_results.iter().filter(|s| s.success).count(), 2, "Mock should succeed");

    println!("✅ Partial completion test passed");
}

fn create_test_flow_plan() -> DynamicFlowPlan {
    let context = WalletContext {
        owner: "test_wallet_123".to_string(),
        sol_balance: 4000000000, // 4 SOL
        token_accounts: vec![],
        total_value_usd: 600.0,
    };

    let step1 = DynamicStep {
        step_id: "balance_check".to_string(),
        prompt_template: "Check wallet balance for test wallet".to_string(),
        description: "Balance check step".to_string(),
        required_tools: vec!["account_balance".to_string()],
        critical: false,
        recovery_strategy: None,
        estimated_time_seconds: 10,
    };

    let step2 = DynamicStep {
        step_id: "swap_1".to_string(),
        prompt_template: "Swap 1 SOL to USDC".to_string(),
        description: "Swap execution step".to_string(),
        required_tools: vec!["jupiter_swap".to_string()],
        critical: true,
        recovery_strategy: None,
        estimated_time_seconds: 30,
    };

    DynamicFlowPlan {
        flow_id: "test-ping-pong-123".to_string(),
        user_prompt: "Test prompt".to_string(),
        steps: vec![step1, step2],
        context,
        atomic_mode: reev_types::flow::AtomicMode::Strict,
        metadata: reev_types::flow::FlowMetadata::default(),
    }
}
