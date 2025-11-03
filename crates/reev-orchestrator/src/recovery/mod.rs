//! Recovery Module for Phase 3
//!
//! This module implements recovery mechanisms for dynamic flow execution,
//! including retry strategies, alternative flows, and user fulfillment.

use crate::Result;
use reev_types::flow::{
    AtomicMode, DynamicFlowPlan, DynamicStep, RecoveryStrategy, StepResult,
};
use std::time::Duration;
use tracing::{debug, instrument, warn};

pub mod engine;
pub mod strategies;

pub use engine::RecoveryEngine;
pub use strategies::{AlternativeFlowStrategy, RetryStrategy, UserFulfillmentStrategy};

/// Recovery result for step execution
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// Whether recovery was successful
    pub success: bool,
    /// Number of attempts made
    pub attempts_made: usize,
    /// Recovery strategy used
    pub strategy_used: RecoveryStrategy,
    /// Recovery error message if failed
    pub error_message: Option<String>,
    /// Recovery time in milliseconds
    pub recovery_time_ms: u64,
}

/// Recovery configuration for fine-tuning behavior
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Base delay between retries in milliseconds
    pub base_retry_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_retry_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum total recovery time per step in milliseconds
    pub max_recovery_time_ms: u64,
    /// Whether to enable alternative flow recovery
    pub enable_alternative_flows: bool,
    /// Whether to enable user fulfillment recovery
    pub enable_user_fulfillment: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            base_retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            backoff_multiplier: 2.0,
            max_recovery_time_ms: 30000, // 30 seconds max
            enable_alternative_flows: true,
            enable_user_fulfillment: false, // Disabled by default for automated systems
        }
    }
}

/// Step execution context for recovery
#[derive(Debug)]
pub struct StepExecutionContext {
    /// Current step being executed
    pub step: DynamicStep,
    /// Flow plan context
    pub flow_plan: DynamicFlowPlan,
    /// Previous step results
    pub previous_results: Vec<StepResult>,
    /// Current attempt number
    pub current_attempt: usize,
    /// Step execution deadline
    pub deadline: Option<std::time::Instant>,
}

impl StepExecutionContext {
    /// Create new step execution context
    pub fn new(
        step: DynamicStep,
        flow_plan: DynamicFlowPlan,
        previous_results: Vec<StepResult>,
    ) -> Self {
        Self {
            step,
            flow_plan,
            previous_results,
            current_attempt: 1,
            deadline: None,
        }
    }

    /// Set execution deadline
    pub fn with_deadline(mut self, deadline: std::time::Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Check if deadline is exceeded
    pub fn is_deadline_exceeded(&self) -> bool {
        if let Some(deadline) = self.deadline {
            std::time::Instant::now() > deadline
        } else {
            false
        }
    }

    /// Increment attempt counter
    pub fn increment_attempt(&mut self) {
        self.current_attempt += 1;
    }

    /// Get remaining time until deadline
    pub fn remaining_time(&self) -> Option<Duration> {
        if let Some(deadline) = self.deadline {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            Some(remaining)
        } else {
            None
        }
    }
}

/// Recovery outcome for flow execution decision making
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryOutcome {
    /// Recovery successful, continue with next step
    Continue,
    /// Recovery failed but step is non-critical, continue with next step
    ContinueNonCritical,
    /// Recovery failed and step is critical, abort entire flow
    AbortCritical,
    /// Recovery failed and no more attempts available
    AbortNoMoreAttempts,
    /// Recovery exceeded time limit
    AbortTimeout,
}

/// Recovery engine interface for different recovery strategies
use async_trait::async_trait;

#[async_trait]
pub trait RecoveryStrategyEngine: Send + Sync {
    /// Attempt recovery for a failed step
    async fn attempt_recovery(
        &self,
        context: &StepExecutionContext,
        config: &RecoveryConfig,
        original_error: &str,
    ) -> Result<RecoveryResult>;

    /// Get strategy name for logging
    fn strategy_name(&self) -> &'static str;

    /// Check if strategy is applicable for the given step
    fn is_applicable(&self, step: &DynamicStep) -> bool;
}

/// Helper functions for recovery operations
pub mod helpers {
    use super::*;

    /// Calculate exponential backoff delay
    pub fn calculate_backoff_delay(
        attempt: usize,
        base_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
    ) -> u64 {
        let delay = (base_delay_ms as f64 * multiplier.powi(attempt as i32 - 1)).round() as u64;
        delay.min(max_delay_ms)
    }

    /// Check if step should be retried based on error type
    pub fn should_retry_error(error_message: &str) -> bool {
        let error_lower = error_message.to_lowercase();

        // Don't retry on permanent errors
        let permanent_errors = [
            "insufficient funds",
            "invalid signature",
            "account not found",
            "invalid instruction",
            "custom program error",
            "permission denied",
            "authentication failed",
        ];

        for permanent_error in &permanent_errors {
            if error_lower.contains(permanent_error) {
                return false;
            }
        }

        // Retry on transient errors
        let transient_errors = [
            "timeout",
            "network error",
            "connection refused",
            "rate limit",
            "temporary failure",
            "service unavailable",
            "slot skipped",
            "blockhash not found",
        ];

        for transient_error in &transient_errors {
            if error_lower.contains(transient_error) {
                return true;
            }
        }

        // Default to retrying if error type is unknown
        true
    }

    /// Determine recovery outcome based on step criticality and result
    pub fn determine_recovery_outcome(
        step: &DynamicStep,
        recovery_result: &RecoveryResult,
        atomic_mode: AtomicMode,
    ) -> RecoveryOutcome {
        if recovery_result.success {
            return RecoveryOutcome::Continue;
        }

        match atomic_mode {
            AtomicMode::Strict => {
                // In strict mode, any failure aborts the flow
                if step.critical {
                    RecoveryOutcome::AbortCritical
                } else {
                    RecoveryOutcome::ContinueNonCritical
                }
            }
            AtomicMode::Lenient => {
                // In lenient mode, continue regardless of criticality
                RecoveryOutcome::ContinueNonCritical
            }
            AtomicMode::Conditional => {
                // In conditional mode, check step criticality
                if step.critical {
                    RecoveryOutcome::AbortCritical
                } else {
                    RecoveryOutcome::ContinueNonCritical
                }
            }
        }
    }

    /// Log recovery attempt details
    #[instrument(skip(context))]
    pub fn log_recovery_attempt(
        context: &StepExecutionContext,
        strategy_name: &str,
        original_error: &str,
        config: &RecoveryConfig,
    ) {
        debug!(
            step_id = %context.step.step_id,
            flow_id = %context.flow_plan.flow_id,
            attempt = context.current_attempt,
            strategy = strategy_name,
            error = %original_error,
            critical = %context.step.critical,
            "Attempting recovery for failed step"
        );

        if let Some(remaining) = context.remaining_time() {
            debug!(
                step_id = %context.step.step_id,
                remaining_ms = remaining.as_millis(),
                max_recovery_ms = config.max_recovery_time_ms,
                "Recovery time window"
            );
        }
    }
}
