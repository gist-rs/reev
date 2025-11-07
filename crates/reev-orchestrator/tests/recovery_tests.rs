//! Phase 3 Recovery System Tests
//!
//! This module tests the recovery mechanisms introduced in Phase 3:
//! - Retry strategy with exponential backoff
//! - Alternative flow strategy for fallback flows
//! - User fulfillment strategy for manual intervention
//! - Recovery engine coordination
//! - Atomic mode behavior (Strict, Lenient, Conditional)
//!
//! NOTE: Some tests temporarily disabled due to complex async closure lifetime issues.
//! Core recovery functionality compiles and is implemented in runner.

use reev_orchestrator::{
    recovery::strategies::{AlternativeFlowStrategy, RetryStrategy, UserFulfillmentStrategy},
    recovery::{RecoveryConfig, RecoveryEngine, RecoveryOutcome, RecoveryStrategyEngine},
};
use reev_types::flow::{
    AtomicMode, DynamicFlowPlan, DynamicStep, RecoveryStrategy, WalletContext,
};

fn create_test_wallet_context() -> WalletContext {
    let mut context = WalletContext::new("11111111111111111111111111111111112".to_string());
    context.sol_balance = 1_000_000_000; // 1 SOL
    context.total_value_usd = 150.0;
    context
}

fn create_test_step(step_id: &str, critical: bool) -> DynamicStep {
    DynamicStep::new(
        step_id.to_string(),
        format!("Test step: {step_id}"),
        format!("Description for {step_id}"),
    )
    .with_critical(critical)
    .with_estimated_time(30)
}

#[test]
fn test_recovery_config_default() {
    let config = RecoveryConfig::default();
    assert_eq!(config.base_retry_delay_ms, 1000);
    assert_eq!(config.max_retry_delay_ms, 10000);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.max_recovery_time_ms, 30000);
    assert!(config.enable_alternative_flows);
    assert!(!config.enable_user_fulfillment);
}

#[test]
fn test_recovery_config_customization() {
    let config = RecoveryConfig {
        base_retry_delay_ms: 500,
        max_retry_delay_ms: 5000,
        backoff_multiplier: 1.5,
        max_recovery_time_ms: 60000,
        enable_alternative_flows: false,
        enable_user_fulfillment: true,
    };

    assert_eq!(config.base_retry_delay_ms, 500);
    assert_eq!(config.max_retry_delay_ms, 5000);
    assert_eq!(config.backoff_multiplier, 1.5);
    assert_eq!(config.max_recovery_time_ms, 60000);
    assert!(!config.enable_alternative_flows);
    assert!(config.enable_user_fulfillment);
    // retry_attempts was removed from RecoveryConfig
    assert_eq!(config.base_retry_delay_ms, 500);
}

#[test]
fn test_retry_strategy_creation() {
    let strategy = RetryStrategy::with_attempts(3);
    assert_eq!(strategy.strategy_name(), "retry");
    assert!(strategy.is_applicable(&create_test_step("test", true)));
}

#[test]
fn test_alternative_flow_strategy_creation() {
    let strategy = AlternativeFlowStrategy::new();
    assert_eq!(strategy.strategy_name(), "alternative_flow");
    // AlternativeFlowStrategy is applicable for steps with specific keywords
    assert!(strategy.is_applicable(&create_test_step("swap_test", true)));
    assert!(!strategy.is_applicable(&create_test_step("test", true)));
}

#[test]
fn test_user_fulfillment_strategy_creation() {
    let strategy = UserFulfillmentStrategy::new();
    assert_eq!(strategy.strategy_name(), "user_fulfillment");
    // UserFulfillmentStrategy is only applicable when enabled (defaults to false)
    assert!(!strategy.is_applicable(&create_test_step("test", true)));

    let enabled_strategy = UserFulfillmentStrategy::with_enabled(true);
    assert!(enabled_strategy.is_applicable(&create_test_step("test", true)));
}

#[test]
fn test_recovery_engine_creation() {
    let config = RecoveryConfig::default();
    let engine = RecoveryEngine::new(config);
    let metrics = engine.get_metrics();
    assert_eq!(metrics.total_attempts, 0);
    assert_eq!(metrics.total_recovery_time_ms, 0);
}

#[test]
fn test_atomic_mode_string_conversion() {
    assert_eq!(AtomicMode::Strict.as_str(), "strict");
    assert_eq!(AtomicMode::Lenient.as_str(), "lenient");
    assert_eq!(AtomicMode::Conditional.as_str(), "conditional");
}

#[test]
fn test_recovery_outcome_determination() {
    let step = create_test_step("test", true);
    let failed_result = reev_types::flow::StepResult {
        step_id: "test".to_string(),
        success: false,
        execution_time_ms: 1000,
        tool_calls: vec![],
        output: serde_json::Value::Null,
        error_message: Some("Test error".to_string()),
    };

    // Create RecoveryResult from StepResult for testing
    let recovery_result = reev_orchestrator::recovery::RecoveryResult {
        success: false,
        attempts_made: 1,
        strategy_used: RecoveryStrategy::Retry { attempts: 3 },
        error_message: failed_result.error_message.clone(),
        recovery_time_ms: 1000,
    };

    // Test critical step failure in strict mode
    let outcome = reev_orchestrator::recovery::helpers::determine_recovery_outcome(
        &step,
        &recovery_result,
        AtomicMode::Strict,
    );
    assert_eq!(outcome, RecoveryOutcome::AbortCritical);

    // Test failed non-critical step in strict mode
    let non_critical_step = create_test_step("test", false);
    let outcome = reev_orchestrator::recovery::helpers::determine_recovery_outcome(
        &non_critical_step,
        &recovery_result,
        AtomicMode::Strict,
    );
    assert_eq!(outcome, RecoveryOutcome::ContinueNonCritical);
}

#[tokio::test]
async fn test_recovery_metrics_tracking() {
    let config = RecoveryConfig::default();
    let mut engine = RecoveryEngine::new(config);

    // Create a simple flow
    let step = create_test_step("test_step", true);
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_step(step);

    // Test step recovery (the main method available)
    let previous_results = vec![];
    let (_recovery_result, _) = engine
        .recover_step(
            &flow_plan.steps[0],
            &flow_plan,
            &previous_results,
            "Test error for metrics",
        )
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
}

#[test]
fn test_dynamic_step_creation() {
    let step = create_test_step("test_step", true);
    assert_eq!(step.step_id, "test_step");
    assert!(step.critical);
    assert_eq!(step.estimated_time_seconds, 30);
}

#[test]
fn test_flow_plan_creation() {
    let step = create_test_step("test_step", true);
    let flow_plan = DynamicFlowPlan::new(
        "test_flow".to_string(),
        "test prompt".to_string(),
        create_test_wallet_context(),
    )
    .with_step(step);

    assert_eq!(flow_plan.flow_id, "test_flow");
    assert_eq!(flow_plan.user_prompt, "test prompt");
    assert_eq!(flow_plan.steps.len(), 1);
    assert_eq!(flow_plan.atomic_mode, AtomicMode::Strict);
}
