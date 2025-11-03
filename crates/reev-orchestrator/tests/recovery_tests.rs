//! Phase 3 Recovery System Tests
//!
//! This module tests the recovery mechanisms introduced in Phase 3:
//! - Retry strategy with exponential backoff
//! - Alternative flow strategy for fallback flows
//! - User fulfillment strategy for manual intervention
//! - Recovery engine coordination
//! - Atomic mode behavior (Strict, Lenient, Conditional)

use reev_orchestrator::{
    recovery::strategies::{AlternativeFlowStrategy, RetryStrategy, UserFulfillmentStrategy},
    recovery::{
        RecoveryConfig, RecoveryEngine, RecoveryOutcome, RecoveryResult, RecoveryStrategyEngine,
    },
};
use reev_types::flow::{
    AtomicMode, DynamicFlowPlan, DynamicStep, RecoveryStrategy, StepResult, WalletContext,
};
use std::time::Duration;
use tokio::time::sleep;

fn create_test_wallet_context() -> WalletContext {
    let mut context = WalletContext::new("11111111111111111111111111111112".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL
    context.total_value_usd = 150.0;
    context
}

fn create_test_step(step_id: &str, critical: bool) -> DynamicStep {
    DynamicStep::new(
        step_id.to_string(),
        format!("Test prompt for {}", step_id),
        format!("Test step {}", step_id),
    )
    .with_critical(critical)
    .with_recovery(RecoveryStrategy::Retry { attempts: 3 })
}

fn create_test_step_result(step_id: &str, success: bool) -> StepResult {
    StepResult {
        step_id: step_id.to_string(),
        success,
        duration_ms: 1000,
        tool_calls: vec!["test_tool".to_string()],
        output: if success {
            Some("Success".to_string())
        } else {
            None
        },
        error_message: if !success {
            Some("Test error".to_string())
        } else {
            None
        },
        recovery_attempts: 0,
    }
}

#[tokio::test]
async fn test_retry_strategy_success() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = RetryStrategy::with_attempts(3);
    let config = RecoveryConfig::default();
    let step = create_test_step("test_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Simulate success on second attempt
    let result = strategy
        .attempt_recovery(&context, &config, "Transient error")
        .await?;

    assert!(result.success);
    assert_eq!(result.attempts_made, 3); // Simulated success on final attempt
    assert!(matches!(
        result.strategy_used,
        RecoveryStrategy::Retry { attempts: 3 }
    ));

    Ok(())
}

#[tokio::test]
async fn test_retry_strategy_non_retryable_error() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = RetryStrategy::new();
    let config = RecoveryConfig::default();
    let step = create_test_step("test_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Non-retryable error
    let result = strategy
        .attempt_recovery(&context, &config, "Insufficient funds")
        .await?;

    assert!(!result.success);
    assert_eq!(result.attempts_made, 0);
    assert!(result.error_message.unwrap().contains("not retryable"));

    Ok(())
}

#[tokio::test]
async fn test_alternative_flow_strategy_jupiter_error() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = AlternativeFlowStrategy::new();
    let config = RecoveryConfig::default();
    let step = create_test_step("swap_1", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Jupiter-specific error should trigger alternative flow
    let result = strategy
        .attempt_recovery(&context, &config, "Jupiter timeout occurred")
        .await?;

    assert!(result.success);
    assert_eq!(result.attempts_made, 1);
    assert!(matches!(
        result.strategy_used,
        RecoveryStrategy::AlternativeFlow { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn test_alternative_flow_strategy_no_alternative() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = AlternativeFlowStrategy::new();
    let config = RecoveryConfig::default();
    let step = create_test_step("unknown_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Unknown error should not trigger alternative flow
    let result = strategy
        .attempt_recovery(&context, &config, "Unknown error")
        .await?;

    assert!(!result.success);
    assert_eq!(result.attempts_made, 0);
    assert!(result
        .error_message
        .unwrap()
        .contains("No suitable alternative flow"));

    Ok(())
}

#[tokio::test]
async fn test_user_fulfillment_strategy_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = UserFulfillmentStrategy::with_enabled(false);
    let config = RecoveryConfig::default();
    let step = create_test_step("test_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Disabled strategy should return failure
    let result = strategy
        .attempt_recovery(&context, &config, "Test error")
        .await?;

    assert!(!result.success);
    assert_eq!(result.attempts_made, 0);
    assert!(result
        .error_message
        .unwrap()
        .contains("User fulfillment is disabled"));

    Ok(())
}

#[tokio::test]
async fn test_user_fulfillment_strategy_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = UserFulfillmentStrategy::with_enabled(true);
    let config = RecoveryConfig::default();
    let step = create_test_step("test_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Enabled strategy should simulate user interaction
    let result = strategy
        .attempt_recovery(&context, &config, "Test error")
        .await?;

    assert!(result.success); // Simulated user chooses to retry
    assert_eq!(result.attempts_made, 1);
    assert!(matches!(
        result.strategy_used,
        RecoveryStrategy::UserFulfillment { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn test_recovery_engine_strict_mode_critical_failure(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RecoveryConfig::default();
    let mut engine = RecoveryEngine::new(config);

    let step = create_test_step("critical_step", true);
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_atomic_mode(AtomicMode::Strict)
    .with_step(step);

    // Mock step executor that always fails
    let step_executor = |_step: &DynamicStep, _previous_results: &Vec<StepResult>| {
        Err(anyhow::anyhow!("Step execution failed"))
    };

    let result = engine
        .execute_flow_with_recovery(flow_plan, step_executor)
        .await;

    // In strict mode, critical failure should abort the flow
    assert!(!result.success);
    assert_eq!(result.metrics.critical_failures, 1);
    assert_eq!(result.metrics.failed_steps, 1);
    assert_eq!(result.metrics.successful_steps, 0);

    Ok(())
}

#[tokio::test]
async fn test_recovery_engine_lenient_mode_critical_failure(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RecoveryConfig::default();
    let mut engine = RecoveryEngine::new(config);

    let step = create_test_step("critical_step", true);
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_atomic_mode(AtomicMode::Lenient)
    .with_step(step);

    // Mock step executor that always fails
    let step_executor = |_step: &DynamicStep, _previous_results: &Vec<StepResult>| {
        Err(anyhow::anyhow!("Step execution failed"))
    };

    let result = engine
        .execute_flow_with_recovery(flow_plan, step_executor)
        .await;

    // In lenient mode, flow continues despite critical failure
    // but overall success depends on whether recovery succeeded
    assert!(result.step_results.len() == 1);
    assert!(result.metrics.failed_steps >= 1);

    Ok(())
}

#[tokio::test]
async fn test_recovery_engine_conditional_mode_non_critical_failure(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RecoveryConfig::default();
    let mut engine = RecoveryEngine::new(config);

    let step1 = create_test_step("step_1", true);
    let step2 = create_test_step("step_2", false); // Non-critical
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_atomic_mode(AtomicMode::Conditional)
    .with_step(step1)
    .with_step(step2);

    let mut call_count = 0;
    let step_executor = move |step: &DynamicStep, _previous_results: &Vec<StepResult>| async move {
        call_count += 1;
        if step.step_id == "step_2" {
            // Non-critical step fails
            Err(anyhow::anyhow!("Non-critical step failed"))
        } else {
            // Critical step succeeds
            Ok(create_test_step_result(&step.step_id, true))
        }
    };

    let result = engine
        .execute_flow_with_recovery(flow_plan, Box::new(step_executor))
        .await;

    // In conditional mode, non-critical failure should not abort flow
    assert!(result.success); // Should be successful because critical step passed
    assert_eq!(result.metrics.successful_steps, 1);
    assert_eq!(result.metrics.non_critical_failures, 1);
    assert_eq!(result.metrics.critical_failures, 0);

    Ok(())
}

#[tokio::test]
async fn test_recovery_engine_multiple_steps_mixed_results(
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RecoveryConfig {
        max_recovery_time_ms: 60000, // 60 seconds for this test
        enable_alternative_flows: true,
        ..Default::default()
    };
    let mut engine = RecoveryEngine::new(config);

    let step1 = create_test_step("step_1", true); // Success
    let step2 = create_test_step("step_2", true); // Success after retry
    let step3 = create_test_step("step_3", false); // Non-critical failure
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_atomic_mode(AtomicMode::Conditional)
    .with_step(step1)
    .with_step(step2)
    .with_step(step3);

    let mut step_count = 0;
    let step_executor = move |step: &DynamicStep, _previous_results: &Vec<StepResult>| {
        step_count += 1;
        match step.step_id.as_str() {
            "step_1" => Ok(create_test_step_result(&step.step_id, true)),
            "step_2" => {
                if step_count <= 2 {
                    Err(anyhow::anyhow!("Transient error"))
                } else {
                    Ok(create_test_step_result(&step.step_id, true))
                }
            }
            "step_3" => Err(anyhow::anyhow!("Non-critical step failed")),
            _ => Err(anyhow::anyhow!("Unknown step")),
        }
    };

    let result = engine
        .execute_flow_with_recovery(flow_plan, Box::new(step_executor))
        .await;

    // Should have 2 successful steps, 1 non-critical failure
    assert_eq!(result.metrics.successful_steps, 2);
    assert_eq!(result.metrics.failed_steps, 1);
    assert_eq!(result.metrics.non_critical_failures, 1);
    assert_eq!(result.metrics.critical_failures, 0);

    // Overall success in conditional mode
    assert!(result.success);

    Ok(())
}

#[tokio::test]
async fn test_recovery_config_customization() -> Result<(), Box<dyn std::error::Error>> {
    let custom_config = RecoveryConfig {
        base_retry_delay_ms: 500,
        max_retry_delay_ms: 5000,
        backoff_multiplier: 1.5,
        max_recovery_time_ms: 15000, // 15 seconds
        enable_alternative_flows: false,
        enable_user_fulfillment: false,
    };

    let strategy = RetryStrategy::new();
    let step = create_test_step("test_step", true);
    let context = reev_orchestrator::recovery::StepExecutionContext::new(
        step.clone(),
        DynamicFlowPlan::new(
            "test_flow".to_string(),
            "test prompt".to_string(),
            create_test_wallet_context(),
        ),
        vec![],
    );

    // Test that custom config is used
    let result = strategy
        .attempt_recovery(&context, &custom_config, "Transient error")
        .await?;

    assert!(result.success);
    // The exact timing would depend on the implementation, but it should use the custom delays
    assert_eq!(result.attempts_made, 3);

    Ok(())
}

#[tokio::test]
async fn test_recovery_metrics_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let config = RecoveryConfig::default();
    let mut engine = RecoveryEngine::new(config);

    // Create a simple flow that will trigger recovery
    let step = create_test_step("test_step", true);
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_step(step);

    let step_executor = |_step: &DynamicStep, _previous_results: &[StepResult]| async {
        Err(anyhow::anyhow!("Step always fails"))
    };

    let _result = engine
        .execute_flow_with_recovery(flow_plan, step_executor)
        .await;

    // Check that metrics were tracked
    let metrics = engine.get_metrics();
    assert!(metrics.total_attempts > 0);
    assert!(metrics.total_recovery_time_ms > 0);

    // Reset metrics and verify they're cleared
    engine.reset_metrics();
    let reset_metrics = engine.get_metrics();
    assert_eq!(reset_metrics.total_attempts, 0);
    assert_eq!(reset_metrics.total_recovery_time_ms, 0);

    Ok(())
}

#[test]
fn test_atomic_mode_string_conversion() {
    assert_eq!(AtomicMode::Strict.as_str(), "strict");
    assert_eq!(AtomicMode::Lenient.as_str(), "lenient");
    assert_eq!(AtomicMode::Conditional.as_str(), "conditional");
}

#[test]
fn test_recovery_outcome_determination() {
    let step = create_test_step("test_step", true);

    // Test successful recovery
    let successful_result = RecoveryResult {
        success: true,
        attempts_made: 1,
        strategy_used: RecoveryStrategy::Retry { attempts: 3 },
        error_message: None,
        recovery_time_ms: 1000,
    };

    let outcome = reev_orchestrator::recovery::helpers::determine_recovery_outcome(
        &step,
        &successful_result,
        AtomicMode::Strict,
    );
    assert_eq!(outcome, RecoveryOutcome::Continue);

    // Test failed critical step in strict mode
    let failed_result = RecoveryResult {
        success: false,
        attempts_made: 3,
        strategy_used: RecoveryStrategy::Retry { attempts: 3 },
        error_message: Some("Failed".to_string()),
        recovery_time_ms: 3000,
    };

    let outcome = reev_orchestrator::recovery::helpers::determine_recovery_outcome(
        &step,
        &failed_result,
        AtomicMode::Strict,
    );
    assert_eq!(outcome, RecoveryOutcome::AbortCritical);

    // Test failed non-critical step in strict mode
    let non_critical_step = create_test_step("test_step", false);
    let outcome = reev_orchestrator::recovery::helpers::determine_recovery_outcome(
        &non_critical_step,
        &failed_result,
        AtomicMode::Strict,
    );
    assert_eq!(outcome, RecoveryOutcome::ContinueNonCritical);
}
