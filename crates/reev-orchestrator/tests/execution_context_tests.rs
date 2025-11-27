//! Tests for execution context module

use reev_constants::JUPITER_SWAP;
use reev_orchestrator::execution::context::ExecutionContext;

#[test]
fn test_execution_context_tracking() {
    let mut ctx = ExecutionContext::with_total_steps(3);

    // Add first successful step
    let step1 = reev_types::flow::StepResult {
        step_id: "step1".to_string(),
        success: true,
        execution_time_ms: 1000,
        tool_calls: vec![JUPITER_SWAP.to_string()],
        output: serde_json::json!({"transactions": [{"signature": "abc123"}]}),
        error_message: None,
    };
    ctx.add_step_result("step1", &step1);

    assert_eq!(ctx.completed_steps(), 1);
    assert_eq!(ctx.completion_percentage(), 33.33333333333333);
    assert!(ctx.was_step_successful("step1"));

    // Add second failed step
    let step2 = reev_types::flow::StepResult {
        step_id: "step2".to_string(),
        success: false,
        execution_time_ms: 500,
        tool_calls: vec![],
        output: serde_json::Value::Null,
        error_message: Some("Failed".to_string()),
    };
    ctx.add_step_result("step2", &step2);

    assert_eq!(ctx.completed_steps(), 2);
    assert_eq!(ctx.completion_percentage(), 66.66666666666666);
    assert!(!ctx.was_step_successful("step2"));
    assert_eq!(ctx.calculate_flow_score(), 0.6666666666666666); // 2/3 completed
}

#[test]
fn test_accumulated_data() {
    let mut ctx = ExecutionContext::new();

    // Store some data
    ctx.store_data("swap_amount".to_string(), serde_json::json!(1000.0));
    ctx.store_data("signature".to_string(), serde_json::json!("abc123"));

    assert_eq!(
        ctx.get_accumulated_data("swap_amount"),
        Some(&serde_json::json!(1000.0))
    );
    assert_eq!(
        ctx.get_accumulated_data("signature"),
        Some(&serde_json::json!("abc123"))
    );
    assert_eq!(ctx.get_accumulated_data("missing"), None);
}
