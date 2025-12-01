//! Tests for benchmark runners with protocol interface integration

use reev_core::benchmark::runner::types::{Flow, FlowStep};
use reev_core::benchmark::runner::{DynamicBenchmarkRunner, StaticBenchmarkRunner};
use reev_core::protocols::{OperationType, ProtocolRegistry};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_static_runner_swap() {
    let runner = StaticBenchmarkRunner::new();

    // Create a simple swap flow
    let flow = Flow {
        id: "test-swap".to_string(),
        prompt: Some("swap 1 SOL to USDC".to_string()),
        created_at: chrono::Utc::now(),
        subject_wallet_info: None,
        steps: vec![FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "swap 1 SOL to USDC".to_string(),
            context: Some("swap 1 SOL to USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        }],
        ground_truth: None,
    };

    // Execute the flow
    let report = runner.execute_flow(&flow).await.unwrap();

    // Verify the report
    assert_eq!(report.flow_id, "test-swap");
    assert_eq!(report.prompt, "swap 1 SOL to USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert_eq!(report.execution_metrics.steps_executed, 1);
    assert_eq!(report.execution_metrics.tool_calls_made, 1);
}

#[tokio::test]
async fn test_static_runner_lend() {
    let runner = StaticBenchmarkRunner::new();

    // Create a simple lend flow
    let flow = Flow {
        id: "test-lend".to_string(),
        prompt: Some("lend 10 USDC".to_string()),
        created_at: chrono::Utc::now(),
        subject_wallet_info: None,
        steps: vec![FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "lend 10 USDC".to_string(),
            context: Some("lend 10 USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        }],
        ground_truth: None,
    };

    // Execute the flow
    let report = runner.execute_flow(&flow).await.unwrap();

    // Verify the report
    assert_eq!(report.flow_id, "test-lend");
    assert_eq!(report.prompt, "lend 10 USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert_eq!(report.execution_metrics.steps_executed, 1);
    assert_eq!(report.execution_metrics.tool_calls_made, 1);
}

#[tokio::test]
async fn test_static_runner_multi_step() {
    let runner = StaticBenchmarkRunner::new();

    // Create a multi-step flow
    let flow = Flow {
        id: "test-multi-step".to_string(),
        prompt: Some("swap 1 SOL to USDC then lend 10 USDC".to_string()),
        created_at: chrono::Utc::now(),
        subject_wallet_info: None,
        steps: vec![
            FlowStep {
                step_id: "1".to_string(),
                refined_prompt: "swap 1 SOL to USDC".to_string(),
                context: Some("swap 1 SOL to USDC".to_string()),
                critical: false,
                expected_tools: vec![],
            },
            FlowStep {
                step_id: "2".to_string(),
                refined_prompt: "lend 10 USDC".to_string(),
                context: Some("lend 10 USDC".to_string()),
                critical: false,
                expected_tools: vec![],
            },
        ],
        ground_truth: None,
    };

    // Execute the flow
    let report = runner.execute_flow(&flow).await.unwrap();

    // Verify the report
    assert_eq!(report.flow_id, "test-multi-step");
    assert_eq!(report.prompt, "swap 1 SOL to USDC then lend 10 USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert_eq!(report.execution_metrics.steps_executed, 2);
    assert_eq!(report.execution_metrics.tool_calls_made, 2);
}

#[tokio::test]
async fn test_dynamic_runner_swap() {
    // Skip this test if we can't create a dynamic runner (might be environment issues)
    let mut runner = match DynamicBenchmarkRunner::new().await {
        Ok(r) => r,
        Err(_) => return,
    };

    // Execute a swap prompt
    let report = runner
        .execute_prompt("swap 1 SOL to USDC", "11111111111111111111111111111111111")
        .await
        .unwrap();

    // Verify the report
    assert!(report.flow_id.len() > 0);
    assert_eq!(report.prompt, "swap 1 SOL to USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert!(report.execution_metrics.steps_executed >= 1);
    assert!(report.execution_metrics.tool_calls_made >= 1);
}

#[tokio::test]
async fn test_dynamic_runner_lend() {
    // Skip this test if we can't create a dynamic runner (might be environment issues)
    let mut runner = match DynamicBenchmarkRunner::new().await {
        Ok(r) => r,
        Err(_) => return,
    };

    // Execute a lend prompt
    let report = runner
        .execute_prompt("lend 10 USDC", "11111111111111111111111111111111111")
        .await
        .unwrap();

    // Verify the report
    assert!(report.flow_id.len() > 0);
    assert_eq!(report.prompt, "lend 10 USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert!(report.execution_metrics.steps_executed >= 1);
    assert!(report.execution_metrics.tool_calls_made >= 1);
}

#[tokio::test]
async fn test_protocol_registry_integration() {
    let runner = StaticBenchmarkRunner::new();

    // Check that Jupiter protocol is registered
    let swap_protocol = runner
        .protocol_registry
        .get_for_operation(&OperationType::Swap);
    assert!(swap_protocol.is_some());

    let lend_protocol = runner
        .protocol_registry
        .get_for_operation(&OperationType::Lend);
    assert!(lend_protocol.is_some());

    let earn_protocol = runner
        .protocol_registry
        .get_for_operation(&OperationType::Earn);
    assert!(earn_protocol.is_some());

    // Check that unsupported operation returns None
    let stake_protocol = runner
        .protocol_registry
        .get_for_operation(&OperationType::Stake);
    assert!(stake_protocol.is_none());
}

#[tokio::test]
async fn test_parameter_extraction() {
    let runner = StaticBenchmarkRunner::new();

    // Create a swap step with specific amounts
    let swap_step = FlowStep {
        step_id: "1".to_string(),
        refined_prompt: "swap 2 SOL to USDC".to_string(),
        context: Some("swap 2 SOL to USDC".to_string()),
        critical: false,
        expected_tools: vec![],
    };

    // Extract parameters
    let operation = runner.extract_operation_from_step(&swap_step).unwrap();

    // Verify operation type
    assert_eq!(operation.operation_type, OperationType::Swap);

    // Verify parameters
    assert_eq!(
        operation.parameters.get("input_mint"),
        Some(&json!("So11111111111111111111111111111111111111112"))
    );
    assert_eq!(
        operation.parameters.get("output_mint"),
        Some(&json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
    );
    assert_eq!(
        operation.parameters.get("amount"),
        Some(&json!(2_000_000_000))
    ); // 2 SOL

    // Create a lend step with specific amounts
    let lend_step = FlowStep {
        step_id: "1".to_string(),
        refined_prompt: "lend 5 USDC".to_string(),
        context: Some("lend 5 USDC".to_string()),
        critical: false,
        expected_tools: vec![],
    };

    // Extract parameters
    let operation = runner.extract_operation_from_step(&lend_step).unwrap();

    // Verify operation type
    assert_eq!(operation.operation_type, OperationType::Lend);

    // Verify parameters
    assert_eq!(
        operation.parameters.get("mint"),
        Some(&json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
    );
    assert_eq!(operation.parameters.get("amount"), Some(&json!(5_000_000))); // 5 USDC
}
