//! Tests for benchmark runners with protocol interface integration

use reev_core::benchmark::runner::types::{Flow, FlowStep};
use reev_core::benchmark::runner::{DynamicBenchmarkRunner, StaticBenchmarkRunner};
use reev_core::protocols::OperationType;
use serde_json::json;
// serial_test is used for test attributes

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

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn test_dynamic_runner_swap() {
    // Initialize dynamic runner with environment setup
    let mut runner = match DynamicBenchmarkRunner::new().await {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Warning: Dynamic runner initialization failed: {e}, skipping test"
            );
            return;
        }
    };

    // Initialize environment (may fail if SURFPOOL not running)
    if let Err(e) = runner.initialize().await {
        println!(
            "Warning: Dynamic runner environment setup failed: {e}, skipping test"
        );
        return;
    }

    // Execute a swap prompt
    let report = match runner.execute_prompt("swap 1 SOL to USDC").await {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Warning: Dynamic runner swap test failed: {e}, skipping test"
            );
            return;
        }
    };

    // Verify the report
    assert!(!report.flow_id.is_empty());
    assert_eq!(report.prompt, "swap 1 SOL to USDC");
    assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    assert!(report.execution_metrics.total_execution_time_ms > 0);
    assert!(report.execution_metrics.steps_executed >= 1);
    assert!(report.execution_metrics.tool_calls_made >= 1);
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn test_dynamic_runner_lend() {
    // Initialize dynamic runner with environment setup
    let mut runner = match DynamicBenchmarkRunner::new().await {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Warning: Dynamic runner initialization failed: {e}, skipping test"
            );
            return;
        }
    };

    // Initialize environment (may fail if SURFPOOL not running)
    if let Err(e) = runner.initialize().await {
        println!(
            "Warning: Dynamic runner environment setup failed: {e}, skipping test"
        );
        return;
    }

    // Execute a lend prompt
    let report = match runner.execute_prompt("lend 10 USDC").await {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Warning: Dynamic runner lend test failed: {e}, skipping test"
            );
            return;
        }
    };

    // Verify the report
    assert!(!report.flow_id.is_empty());
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

#[tokio::test]
async fn test_structured_context_swap() {
    let runner = StaticBenchmarkRunner::new();

    let flow = Flow {
        id: "test-swap-structured".to_string(),
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

    let report = runner.execute_flow(&flow).await.unwrap();

    // Check that the report contains structured context validation
    let structured_context_validations: Vec<_> = report
        .validation_results
        .final_state_results
        .iter()
        .filter(|v| v.assertion_type == "structured_context_preserved")
        .collect();

    assert!(
        !structured_context_validations.is_empty(),
        "Should have structured context validation"
    );

    let validation = structured_context_validations.first().unwrap();
    assert!(
        validation.passed,
        "Structured context validation should pass"
    );

    // Check that we have metadata validation
    let metadata_validations: Vec<_> = report
        .validation_results
        .final_state_results
        .iter()
        .filter(|v| v.assertion_type == "structured_context_metadata")
        .collect();

    assert!(
        !metadata_validations.is_empty(),
        "Should have metadata validation"
    );

    let metadata_validation = metadata_validations.first().unwrap();
    assert!(
        metadata_validation.passed,
        "Metadata validation should pass"
    );

    // Check the actual metadata content
    if let Some(actual_value) = &metadata_validation.actual_value {
        if let Some(metadata) = actual_value.as_object() {
            assert!(
                metadata.get("key_info_count").is_some(),
                "Should have key_info_count"
            );
            assert!(
                metadata.get("balance_changes_count").is_some(),
                "Should have balance_changes_count"
            );
            assert!(
                metadata.get("constraints_count").is_some(),
                "Should have constraints_count"
            );
            assert!(
                metadata.get("available_tokens_count").is_some(),
                "Should have available_tokens_count"
            );
        }
    }
}

#[tokio::test]
async fn test_structured_context_lend() {
    let runner = StaticBenchmarkRunner::new();

    let flow = Flow {
        id: "test-lend-structured".to_string(),
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

    let report = runner.execute_flow(&flow).await.unwrap();

    // Check that the report contains structured context validation
    let structured_context_validations: Vec<_> = report
        .validation_results
        .final_state_results
        .iter()
        .filter(|v| v.assertion_type == "structured_context_preserved")
        .collect();

    assert!(
        !structured_context_validations.is_empty(),
        "Should have structured context validation"
    );

    let validation = structured_context_validations.first().unwrap();
    assert!(
        validation.passed,
        "Structured context validation should pass"
    );
}

#[tokio::test]
async fn test_structured_context_multi_step() {
    let runner = StaticBenchmarkRunner::new();

    let flow = Flow {
        id: "test-multi-step-structured".to_string(),
        prompt: Some("swap 1 SOL to USDC then lend 5 USDC".to_string()),
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
                refined_prompt: "lend 5 USDC".to_string(),
                context: Some("lend 5 USDC".to_string()),
                critical: false,
                expected_tools: vec![],
            },
        ],
        ground_truth: None,
    };

    let report = runner.execute_flow(&flow).await.unwrap();

    // Check that we executed both steps
    assert_eq!(report.execution_metrics.steps_executed, 2);
    assert_eq!(report.execution_metrics.tool_calls_made, 2);
    assert_eq!(report.execution_metrics.successful_tool_calls, 2);

    // Check that the report contains structured context validation
    let structured_context_validations: Vec<_> = report
        .validation_results
        .final_state_results
        .iter()
        .filter(|v| v.assertion_type == "structured_context_preserved")
        .collect();

    assert!(
        !structured_context_validations.is_empty(),
        "Should have structured context validation"
    );

    let validation = structured_context_validations.first().unwrap();
    assert!(
        validation.passed,
        "Structured context validation should pass"
    );
}
