//! Recovery Engine Implementation
//!
//! This module provides the main recovery engine that coordinates
//! different recovery strategies for failed flow steps.

use crate::recovery::{
    helpers::{
        calculate_backoff_delay, determine_recovery_outcome, log_recovery_attempt,
        should_retry_error,
    },
    RecoveryConfig, RecoveryOutcome, RecoveryResult, RecoveryStrategyEngine, StepExecutionContext,
};
use reev_types::flow::{
    AtomicMode, DynamicFlowPlan, DynamicStep, FlowMetrics, FlowResult, StepResult,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

/// Enum for different recovery strategy types
#[derive(Debug)]
pub enum RecoveryStrategyType {
    Retry(crate::recovery::strategies::RetryStrategy),
    AlternativeFlow(crate::recovery::strategies::AlternativeFlowStrategy),
    UserFulfillment(crate::recovery::strategies::UserFulfillmentStrategy),
}

#[async_trait::async_trait]
impl RecoveryStrategyEngine for RecoveryStrategyType {
    async fn attempt_recovery(
        &self,
        context: &StepExecutionContext,
        config: &RecoveryConfig,
        original_error: &str,
    ) -> Result<RecoveryResult, anyhow::Error> {
        match self {
            RecoveryStrategyType::Retry(strategy) => {
                strategy
                    .attempt_recovery(context, config, original_error)
                    .await
            }
            RecoveryStrategyType::AlternativeFlow(strategy) => {
                strategy
                    .attempt_recovery(context, config, original_error)
                    .await
            }
            RecoveryStrategyType::UserFulfillment(strategy) => {
                strategy
                    .attempt_recovery(context, config, original_error)
                    .await
            }
        }
    }

    fn strategy_name(&self) -> &'static str {
        match self {
            RecoveryStrategyType::Retry(_) => "retry",
            RecoveryStrategyType::AlternativeFlow(_) => "alternative_flow",
            RecoveryStrategyType::UserFulfillment(_) => "user_fulfillment",
        }
    }

    fn is_applicable(&self, step: &DynamicStep) -> bool {
        match self {
            RecoveryStrategyType::Retry(strategy) => strategy.is_applicable(step),
            RecoveryStrategyType::AlternativeFlow(strategy) => strategy.is_applicable(step),
            RecoveryStrategyType::UserFulfillment(strategy) => strategy.is_applicable(step),
        }
    }
}

/// Main recovery engine that orchestrates recovery attempts
#[derive(Debug)]
pub struct RecoveryEngine {
    /// Recovery configuration
    config: RecoveryConfig,
    /// Available recovery strategies
    strategies: Vec<RecoveryStrategyType>,
    /// Metrics tracking
    metrics: RecoveryMetrics,
}

/// Internal recovery metrics for tracking performance
#[derive(Debug, Default, Clone)]
pub struct RecoveryMetrics {
    /// Total recovery attempts made
    pub total_attempts: usize,
    /// Successful recoveries
    pub successful_recoveries: usize,
    /// Failed recoveries
    pub failed_recoveries: usize,
    /// Total recovery time in milliseconds
    pub total_recovery_time_ms: u64,
    /// Recoveries by strategy
    pub recoveries_by_strategy: std::collections::HashMap<String, usize>,
}

impl RecoveryEngine {
    /// Create new recovery engine with default strategies
    pub fn new(config: RecoveryConfig) -> Self {
        let strategies: Vec<RecoveryStrategyType> = vec![
            RecoveryStrategyType::Retry(crate::recovery::strategies::RetryStrategy::new()),
            RecoveryStrategyType::AlternativeFlow(
                crate::recovery::strategies::AlternativeFlowStrategy::new(),
            ),
            RecoveryStrategyType::UserFulfillment(
                crate::recovery::strategies::UserFulfillmentStrategy::new(),
            ),
        ];

        Self {
            config,
            strategies,
            metrics: RecoveryMetrics::default(),
        }
    }

    /// Create recovery engine with custom strategies
    pub fn with_strategies(config: RecoveryConfig, strategies: Vec<RecoveryStrategyType>) -> Self {
        Self {
            config,
            strategies,
            metrics: RecoveryMetrics::default(),
        }
    }

    /// Attempt recovery for a failed step
    #[instrument(skip(self, step, flow_plan, previous_results))]
    pub async fn recover_step(
        &mut self,
        step: &DynamicStep,
        flow_plan: &DynamicFlowPlan,
        previous_results: &[StepResult],
        original_error: &str,
    ) -> (RecoveryResult, RecoveryOutcome) {
        let start_time = Instant::now();
        let deadline = Instant::now() + Duration::from_millis(self.config.max_recovery_time_ms);

        let mut context =
            StepExecutionContext::new(step.clone(), flow_plan.clone(), previous_results.to_vec())
                .with_deadline(deadline);

        info!(
            step_id = %step.step_id,
            flow_id = %flow_plan.flow_id,
            critical = %step.critical,
            atomic_mode = %flow_plan.atomic_mode.as_str(),
            error = %original_error,
            "Starting recovery for failed step"
        );

        // Try each recovery strategy in order
        for strategy in &self.strategies {
            let strategy_name = strategy.strategy_name();

            if !strategy.is_applicable(step) {
                debug!(
                    strategy = %strategy_name,
                    step_id = %step.step_id,
                    "Strategy not applicable for step"
                );
                continue;
            }

            loop {
                // Check deadline
                if context.is_deadline_exceeded() {
                    warn!(
                        step_id = %context.step.step_id,
                        "Recovery deadline exceeded, aborting"
                    );
                    let recovery_result = RecoveryResult {
                        success: false,
                        attempts_made: context.current_attempt - 1,
                        strategy_used: step
                            .recovery_strategy
                            .clone()
                            .unwrap_or(reev_types::flow::RecoveryStrategy::Retry { attempts: 3 }),
                        error_message: Some("Recovery deadline exceeded".to_string()),
                        recovery_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    return (recovery_result, RecoveryOutcome::AbortTimeout);
                }

                log_recovery_attempt(&context, strategy_name, original_error, &self.config);

                // Attempt recovery
                match strategy
                    .attempt_recovery(&context, &self.config, original_error)
                    .await
                {
                    Ok(recovery_result) => {
                        let recovery_time = start_time.elapsed().as_millis() as u64;

                        // Update metrics
                        self.update_metrics(strategy_name, &recovery_result, recovery_time);

                        let outcome = determine_recovery_outcome(
                            step,
                            &recovery_result,
                            flow_plan.atomic_mode,
                        );

                        if recovery_result.success {
                            info!(
                                step_id = %step.step_id,
                                strategy = %strategy_name,
                                attempts = %recovery_result.attempts_made,
                                recovery_time_ms = %recovery_time,
                                "Recovery successful"
                            );
                        } else {
                            warn!(
                                step_id = %step.step_id,
                                strategy = %strategy_name,
                                attempts = %recovery_result.attempts_made,
                                error = %recovery_result.error_message.as_deref().unwrap_or("unknown"),
                                "Recovery failed"
                            );
                        }

                        return (recovery_result, outcome);
                    }
                    Err(e) => {
                        error!(
                            step_id = %context.step.step_id,
                            strategy = %strategy_name,
                            attempt = %context.current_attempt,
                            error = %e,
                            "Recovery attempt failed"
                        );

                        // Check if we should retry this strategy
                        if should_retry_error(&e.to_string()) && context.current_attempt < 3 {
                            // Wait before retry
                            let delay_ms = calculate_backoff_delay(
                                context.current_attempt,
                                self.config.base_retry_delay_ms,
                                self.config.max_retry_delay_ms,
                                self.config.backoff_multiplier,
                            );

                            debug!(
                                step_id = %context.step.step_id,
                                delay_ms = %delay_ms,
                                "Waiting before retry"
                            );

                            sleep(Duration::from_millis(delay_ms)).await;
                            context.increment_attempt();
                            continue;
                        } else {
                            // Move to next strategy
                            debug!(
                                step_id = %context.step.step_id,
                                strategy = %strategy_name,
                                "Moving to next recovery strategy"
                            );
                            break;
                        }
                    }
                }
            }
        }

        // All strategies failed
        let recovery_result = RecoveryResult {
            success: false,
            attempts_made: context.current_attempt - 1,
            strategy_used: reev_types::flow::RecoveryStrategy::Retry { attempts: 0 },
            error_message: Some("All recovery strategies exhausted".to_string()),
            recovery_time_ms: start_time.elapsed().as_millis() as u64,
        };

        let outcome = determine_recovery_outcome(step, &recovery_result, flow_plan.atomic_mode);

        error!(
            step_id = %step.step_id,
            total_attempts = %recovery_result.attempts_made,
            "All recovery strategies failed"
        );

        (recovery_result, outcome)
    }

    /// Execute a flow with recovery support
    #[instrument(skip(self, flow_plan, step_executor))]
    pub async fn execute_flow_with_recovery<F, Fut>(
        &mut self,
        flow_plan: DynamicFlowPlan,
        mut step_executor: F,
    ) -> FlowResult
    where
        F: FnMut(&DynamicStep, &Vec<StepResult>) -> Fut,
        Fut: std::future::Future<Output = Result<StepResult, anyhow::Error>> + Send,
    {
        let start_time = Instant::now();
        let mut step_results = Vec::new();
        let mut successful_steps = 0;
        let mut failed_steps = 0;
        let mut critical_failures = 0;
        let mut non_critical_failures = 0;

        info!(
            flow_id = %flow_plan.flow_id,
            total_steps = %flow_plan.steps.len(),
            atomic_mode = %flow_plan.atomic_mode.as_str(),
            "Starting flow execution with recovery"
        );

        for (index, step) in flow_plan.steps.iter().enumerate() {
            debug!(
                flow_id = %flow_plan.flow_id,
                step_id = %step.step_id,
                step_index = %index,
                total_steps = %flow_plan.steps.len(),
                critical = %step.critical,
                "Executing step"
            );

            // Attempt to execute step
            match step_executor(step, &step_results).await {
                Ok(step_result) => {
                    successful_steps += 1;
                    step_results.push(step_result);

                    debug!(
                        flow_id = %flow_plan.flow_id,
                        step_id = %step.step_id,
                        "Step executed successfully"
                    );
                }
                Err(step_error) => {
                    failed_steps += 1;
                    let error_message = step_error.to_string();

                    // Log the initial failure
                    error!(
                        flow_id = %flow_plan.flow_id,
                        step_id = %step.step_id,
                        error = %error_message,
                        "Step execution failed, attempting recovery"
                    );

                    // Attempt recovery
                    let (recovery_result, recovery_outcome) = self
                        .recover_step(step, &flow_plan, &step_results, &error_message)
                        .await;

                    // Create step result based on recovery outcome
                    let final_step_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: recovery_result.success,
                        duration_ms: recovery_result.recovery_time_ms,
                        tool_calls: vec![],
                        output: None,
                        error_message: recovery_result.error_message.clone(),
                        recovery_attempts: recovery_result.attempts_made,
                    };

                    // Update failure counters based on criticality and outcome
                    if !recovery_result.success {
                        if step.critical {
                            critical_failures += 1;
                        } else {
                            non_critical_failures += 1;
                        }
                    } else {
                        // Recovery was successful
                        successful_steps += 1;
                    }

                    step_results.push(final_step_result);

                    // Check if we should abort the flow
                    match recovery_outcome {
                        RecoveryOutcome::AbortCritical | RecoveryOutcome::AbortTimeout => {
                            error!(
                                flow_id = %flow_plan.flow_id,
                                step_id = %step.step_id,
                                outcome = ?recovery_outcome,
                                "Aborting flow due to critical failure or timeout"
                            );
                            break;
                        }
                        RecoveryOutcome::AbortNoMoreAttempts => {
                            warn!(
                                flow_id = %flow_plan.flow_id,
                                step_id = %step.step_id,
                                "No more recovery attempts available"
                            );
                            if step.critical {
                                critical_failures += 1;
                                break;
                            }
                        }
                        RecoveryOutcome::Continue | RecoveryOutcome::ContinueNonCritical => {
                            debug!(
                                flow_id = %flow_plan.flow_id,
                                step_id = %step.step_id,
                                outcome = ?recovery_outcome,
                                "Continuing with next step"
                            );
                            // Continue to next step
                        }
                    }
                }
            }
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        // Create flow metrics
        let metrics = FlowMetrics {
            total_duration_ms,
            successful_steps,
            failed_steps,
            critical_failures,
            non_critical_failures,
            total_tool_calls: step_results.iter().map(|r| r.tool_calls.len()).sum(),
            context_resolution_ms: 0, // Will be set by caller
            prompt_generation_ms: 0,  // Will be set by caller
            cache_hit_rate: 0.0,      // Will be set by caller
        };

        // Determine overall success based on atomic mode and critical failures
        let overall_success = match flow_plan.atomic_mode {
            AtomicMode::Strict => critical_failures == 0 && successful_steps > 0,
            AtomicMode::Lenient => successful_steps > 0, // Any success is acceptable
            AtomicMode::Conditional => {
                // Success if no critical failures
                critical_failures == 0 && successful_steps > 0
            }
        };

        info!(
            flow_id = %flow_plan.flow_id,
            success = %overall_success,
            successful_steps = %successful_steps,
            failed_steps = %failed_steps,
            critical_failures = %critical_failures,
            total_duration_ms = %total_duration_ms,
            recovery_attempts = %self.metrics.total_attempts,
            "Flow execution with recovery completed"
        );

        FlowResult {
            flow_id: flow_plan.flow_id,
            user_prompt: flow_plan.user_prompt,
            success: overall_success,
            step_results,
            metrics,
            final_context: Some(flow_plan.context),
            error_message: if overall_success {
                None
            } else {
                Some(format!(
                    "Flow failed with {critical_failures} critical failures"
                ))
            },
        }
    }

    /// Get recovery metrics
    pub fn get_metrics(&self) -> &RecoveryMetrics {
        &self.metrics
    }

    /// Reset recovery metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = RecoveryMetrics::default();
    }

    /// Update internal metrics
    fn update_metrics(
        &mut self,
        strategy_name: &str,
        result: &RecoveryResult,
        recovery_time_ms: u64,
    ) {
        self.metrics.total_attempts += result.attempts_made;
        self.metrics.total_recovery_time_ms += recovery_time_ms;

        if result.success {
            self.metrics.successful_recoveries += 1;
        } else {
            self.metrics.failed_recoveries += 1;
        }

        *self
            .metrics
            .recoveries_by_strategy
            .entry(strategy_name.to_string())
            .or_insert(0) += 1;
    }
}

impl Default for RecoveryEngine {
    fn default() -> Self {
        Self::new(RecoveryConfig::default())
    }
}

/// Helper function to create step result for recovery testing
#[cfg(test)]
pub fn create_test_step_result(
    step_id: String,
    success: bool,
    error_message: Option<String>,
) -> StepResult {
    StepResult {
        step_id,
        success,
        duration_ms: 1000,
        tool_calls: vec![],
        output: None,
        error_message,
        recovery_attempts: 0,
    }
}
