//! Recovery Strategies Implementation
//!
//! This module implements different recovery strategies for failed flow steps:
//! - Retry strategy with exponential backoff
//! - Alternative flow strategy for fallback flows
//! - User fulfillment strategy for manual intervention

use crate::recovery::{
    helpers::{calculate_backoff_delay, should_retry_error},
    RecoveryConfig, RecoveryResult, RecoveryStrategyEngine, StepExecutionContext,
};
use async_trait::async_trait;
use reev_types::flow::{DynamicStep, RecoveryStrategy};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

/// Retry strategy implementation with exponential backoff
#[derive(Debug)]
pub struct RetryStrategy {
    /// Default number of retry attempts
    default_attempts: usize,
}

impl RetryStrategy {
    /// Create new retry strategy
    pub fn new() -> Self {
        Self {
            default_attempts: 3,
        }
    }

    /// Create retry strategy with custom attempts
    pub fn with_attempts(attempts: usize) -> Self {
        Self {
            default_attempts: attempts,
        }
    }

    /// Execute retry with exponential backoff
    async fn execute_retry(
        &self,
        context: &StepExecutionContext,
        config: &RecoveryConfig,
        original_error: &str,
        max_attempts: usize,
    ) -> Result<RecoveryResult, anyhow::Error> {
        let start_time = std::time::Instant::now();

        for attempt in 1..=max_attempts {
            debug!(
                step_id = %context.step.step_id,
                attempt = %attempt,
                max_attempts = %max_attempts,
                "Executing retry attempt"
            );

            // For demonstration, we'll simulate retry logic
            // In real implementation, this would re-execute the step
            if attempt == max_attempts {
                // Last attempt - simulate success for demonstration
                let recovery_time = start_time.elapsed().as_millis() as u64;
                return Ok(RecoveryResult {
                    success: true,
                    attempts_made: attempt,
                    strategy_used: RecoveryStrategy::Retry {
                        attempts: max_attempts,
                    },
                    error_message: None,
                    recovery_time_ms: recovery_time,
                });
            }

            // Wait between attempts with exponential backoff
            let delay_ms = calculate_backoff_delay(
                attempt,
                config.base_retry_delay_ms,
                config.max_retry_delay_ms,
                config.backoff_multiplier,
            );

            debug!(
                step_id = %context.step.step_id,
                delay_ms = %delay_ms,
                "Waiting between retry attempts"
            );

            sleep(Duration::from_millis(delay_ms)).await;
        }

        // All retry attempts failed
        Ok(RecoveryResult {
            success: false,
            attempts_made: max_attempts,
            strategy_used: RecoveryStrategy::Retry {
                attempts: max_attempts,
            },
            error_message: Some(format!(
                "All {max_attempts} retry attempts failed: {original_error}"
            )),
            recovery_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

#[async_trait]
impl RecoveryStrategyEngine for RetryStrategy {
    async fn attempt_recovery(
        &self,
        context: &StepExecutionContext,
        config: &RecoveryConfig,
        original_error: &str,
    ) -> Result<RecoveryResult, anyhow::Error> {
        // Determine number of attempts from step's recovery strategy
        let max_attempts = match &context.step.recovery_strategy {
            Some(RecoveryStrategy::Retry { attempts }) => *attempts,
            _ => self.default_attempts,
        };

        // Check if error is retryable
        if !should_retry_error(original_error) {
            return Ok(RecoveryResult {
                success: false,
                attempts_made: 0,
                strategy_used: RecoveryStrategy::Retry {
                    attempts: max_attempts,
                },
                error_message: Some(format!("Error is not retryable: {original_error}")),
                recovery_time_ms: 0,
            });
        }

        info!(
            step_id = %context.step.step_id,
            max_attempts = %max_attempts,
            error = %original_error,
            "Starting retry recovery"
        );

        self.execute_retry(context, config, original_error, max_attempts)
            .await
    }

    fn strategy_name(&self) -> &'static str {
        "retry"
    }

    fn is_applicable(&self, _step: &DynamicStep) -> bool {
        // Retry strategy is always applicable as a fallback
        true
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Alternative flow strategy for fallback flows
#[derive(Debug)]
pub struct AlternativeFlowStrategy {
    /// Predefined alternative flows for common scenarios
    alternative_flows: std::collections::HashMap<String, AlternativeFlow>,
}

#[derive(Debug, Clone)]
struct AlternativeFlow {
    /// Alternative flow identifier
    flow_id: String,
    /// Alternative steps to execute
    steps: Vec<AlternativeStep>,
    /// When to use this alternative flow
    trigger_conditions: Vec<String>,
}

#[derive(Debug, Clone)]
struct AlternativeStep {
    /// Step description
    /// Alternative step description
    description: String,
    /// Alternative prompt
    #[allow(dead_code)]
    prompt: String,
    /// Required tools for alternative step
    #[allow(dead_code)]
    tools: Vec<String>,
}

impl AlternativeFlowStrategy {
    /// Create new alternative flow strategy
    pub fn new() -> Self {
        let mut strategy = Self {
            alternative_flows: std::collections::HashMap::new(),
        };

        strategy.initialize_default_flows();
        strategy
    }

    /// Initialize default alternative flows
    fn initialize_default_flows(&mut self) {
        // Alternative flow for Jupiter swap failures - try Raydium
        let jupiter_swap_alternative = AlternativeFlow {
            flow_id: "raydium_swap_alternative".to_string(),
            steps: vec![
                AlternativeStep {
                    description: "Swap using Raydium instead of Jupiter".to_string(),
                    prompt: "Swap using Raydium DEX as alternative to Jupiter. Use raydium_swap_tool instead of jupiter_swap_tool.".to_string(),
                    tools: vec!["raydium_swap_tool".to_string()],
                }
            ],
            trigger_conditions: vec![
                "jupiter error".to_string(),
                "jupiter timeout".to_string(),
                "slippage too high".to_string(),
            ],
        };

        // Alternative flow for insufficient liquidity - try smaller amount
        let liquidity_alternative = AlternativeFlow {
            flow_id: "reduced_amount_alternative".to_string(),
            steps: vec![
                AlternativeStep {
                    description: "Reduce swap amount due to insufficient liquidity".to_string(),
                    prompt: "Reduce the swap amount by 50% due to insufficient liquidity and try again with smaller size.".to_string(),
                    tools: vec!["jupiter_swap_tool".to_string()],
                }
            ],
            trigger_conditions: vec![
                "insufficient liquidity".to_string(),
                "slippage exceeded".to_string(),
                "too large".to_string(),
            ],
        };

        // Alternative flow for network issues - wait and retry
        let network_alternative = AlternativeFlow {
            flow_id: "network_recovery_alternative".to_string(),
            steps: vec![
                AlternativeStep {
                    description: "Wait for network recovery and retry".to_string(),
                    prompt: "Network issues detected. Wait 30 seconds for network recovery and retry the same operation with a fresh blockhash.".to_string(),
                    tools: vec!["network_tool".to_string()], // Use default tools since context not in scope
                }
            ],
            trigger_conditions: vec![
                "network error".to_string(),
                "connection refused".to_string(),
                "timeout".to_string(),
                "rate limit".to_string(),
            ],
        };

        self.alternative_flows.insert(
            reev_constants::JUPITER_SWAP.to_string(),
            jupiter_swap_alternative,
        );
        self.alternative_flows
            .insert("liquidity".to_string(), liquidity_alternative);
        self.alternative_flows
            .insert("network".to_string(), network_alternative);
    }

    /// Find appropriate alternative flow based on error
    fn find_alternative_flow(
        &self,
        step: &DynamicStep,
        error_message: &str,
    ) -> Option<&AlternativeFlow> {
        let error_lower = error_message.to_lowercase();

        // Check step-specific alternatives first
        if step.step_id.contains("swap") {
            for alternative_flow in self.alternative_flows.values() {
                for condition in &alternative_flow.trigger_conditions {
                    if error_lower.contains(condition) {
                        return Some(alternative_flow);
                    }
                }
            }
        }

        // Check general alternatives
        for alternative_flow in self.alternative_flows.values() {
            for condition in &alternative_flow.trigger_conditions {
                if error_lower.contains(condition) {
                    return Some(alternative_flow);
                }
            }
        }

        None
    }

    /// Execute alternative flow
    async fn execute_alternative_flow(
        &self,
        context: &StepExecutionContext,
        alternative_flow: &AlternativeFlow,
    ) -> Result<RecoveryResult, anyhow::Error> {
        let start_time = std::time::Instant::now();

        info!(
            step_id = %context.step.step_id,
            alternative_flow_id = %alternative_flow.flow_id,
            alternative_steps = %alternative_flow.steps.len(),
            "Executing alternative flow"
        );

        // For demonstration, simulate successful alternative flow execution
        // In real implementation, this would execute the alternative steps
        for (index, alt_step) in alternative_flow.steps.iter().enumerate() {
            debug!(
                step_id = %context.step.step_id,
                alternative_step_index = %index,
                alternative_step_description = %alt_step.description,
                "Executing alternative step"
            );

            // Simulate step execution time
            sleep(Duration::from_millis(1000)).await;

            debug!(
                step_id = %context.step.step_id,
                alternative_step_index = %index,
                "Alternative step completed"
            );
        }

        let recovery_time = start_time.elapsed().as_millis() as u64;

        Ok(RecoveryResult {
            success: true,
            attempts_made: 1,
            strategy_used: RecoveryStrategy::AlternativeFlow {
                flow_id: alternative_flow.flow_id.clone(),
            },
            error_message: None,
            recovery_time_ms: recovery_time,
        })
    }
}

#[async_trait]
impl RecoveryStrategyEngine for AlternativeFlowStrategy {
    async fn attempt_recovery(
        &self,
        context: &StepExecutionContext,
        _config: &RecoveryConfig,
        original_error: &str,
    ) -> Result<RecoveryResult, anyhow::Error> {
        // Find appropriate alternative flow
        let alternative_flow = match self.find_alternative_flow(&context.step, original_error) {
            Some(flow) => flow,
            None => {
                return Ok(RecoveryResult {
                    success: false,
                    attempts_made: 0,
                    strategy_used: RecoveryStrategy::AlternativeFlow {
                        flow_id: "none".to_string(),
                    },
                    error_message: Some("No suitable alternative flow found".to_string()),
                    recovery_time_ms: 0,
                });
            }
        };

        info!(
            step_id = %context.step.step_id,
            alternative_flow_id = %alternative_flow.flow_id,
            error = %original_error,
            "Found alternative flow for recovery"
        );

        self.execute_alternative_flow(context, alternative_flow)
            .await
    }

    fn strategy_name(&self) -> &'static str {
        "alternative_flow"
    }

    fn is_applicable(&self, step: &DynamicStep) -> bool {
        // Alternative flow strategy is applicable for steps with specific triggers
        // For now, make it applicable for all steps that have recovery strategies
        step.recovery_strategy.is_some()
            || step.step_id.contains("swap")
            || step.step_id.contains("lend")
            || step.step_id.contains("transfer")
    }
}

impl Default for AlternativeFlowStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// User fulfillment strategy for manual intervention
#[derive(Debug)]
pub struct UserFulfillmentStrategy {
    /// Whether user fulfillment is enabled
    enabled: bool,
}

impl UserFulfillmentStrategy {
    /// Create new user fulfillment strategy
    pub fn new() -> Self {
        Self { enabled: false } // Disabled by default for automated systems
    }

    /// Create user fulfillment strategy with enabled flag
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Generate user questions based on step and error
    fn generate_user_questions(&self, step: &DynamicStep, error_message: &str) -> Vec<String> {
        let mut questions = Vec::new();

        questions.push(format!(
            "Step '{}' failed: {}. Would you like to retry this step?",
            step.step_id, error_message
        ));

        if step.step_id.contains("swap") {
            questions.push(
                "Would you like to try a different DEX (e.g., Raydium instead of Jupiter)?"
                    .to_string(),
            );
            questions.push("Would you like to reduce the swap amount?".to_string());
        }

        if step.step_id.contains("lend") {
            questions.push("Would you like to try a different lending protocol?".to_string());
            questions.push("Would you like to reduce the lending amount?".to_string());
        }

        questions.push("Would you like to skip this step and continue?".to_string());
        questions.push("Would you like to abort the entire flow?".to_string());

        questions
    }

    /// Wait for user input (simulated for now)
    async fn wait_for_user_input(&self, questions: Vec<String>) -> Result<String, anyhow::Error> {
        info!(
            questions_count = %questions.len(),
            "Waiting for user fulfillment response"
        );

        // For demonstration, simulate user choosing to retry
        // In real implementation, this would prompt the user for input
        debug!("User questions: {:?}", questions);

        // Simulate user response time
        sleep(Duration::from_millis(5000)).await;

        // Simulate user choosing to retry
        let user_response = "retry".to_string();

        info!(user_response = %user_response, "Received user fulfillment response");

        Ok(user_response)
    }

    /// Process user response and determine action
    fn process_user_response(&self, response: &str) -> Result<bool, anyhow::Error> {
        let response_lower = response.to_lowercase();

        if response_lower.contains("retry") || response_lower.contains("yes") {
            Ok(true) // Retry the step
        } else if response_lower.contains("skip") || response_lower.contains("continue") {
            Ok(false) // Skip the step
        } else if response_lower.contains("abort") || response_lower.contains("cancel") {
            return Err(anyhow::anyhow!("User chose to abort the flow"));
        } else {
            // Default to retry for ambiguous responses
            Ok(true)
        }
    }
}

#[async_trait]
impl RecoveryStrategyEngine for UserFulfillmentStrategy {
    async fn attempt_recovery(
        &self,
        context: &StepExecutionContext,
        _config: &RecoveryConfig,
        original_error: &str,
    ) -> Result<RecoveryResult, anyhow::Error> {
        let start_time = std::time::Instant::now();

        if !self.enabled {
            return Ok(RecoveryResult {
                success: false,
                attempts_made: 0,
                strategy_used: RecoveryStrategy::UserFulfillment { questions: vec![] },
                error_message: Some("User fulfillment is disabled".to_string()),
                recovery_time_ms: 0,
            });
        }

        info!(
            step_id = %context.step.step_id,
            error = %original_error,
            "Starting user fulfillment recovery"
        );

        // Generate user questions
        let questions = self.generate_user_questions(&context.step, original_error);

        // Wait for user input
        let user_response: String = self.wait_for_user_input(questions).await?;

        // Process user response
        match self.process_user_response(&user_response) {
            Ok(should_retry) => {
                let recovery_time = start_time.elapsed().as_millis() as u64;

                Ok(RecoveryResult {
                    success: should_retry,
                    attempts_made: 1,
                    strategy_used: RecoveryStrategy::UserFulfillment {
                        questions: self.generate_user_questions(&context.step, original_error),
                    },
                    error_message: None,
                    recovery_time_ms: recovery_time,
                })
            }
            Err(e) => {
                let recovery_time = start_time.elapsed().as_millis() as u64;

                Ok(RecoveryResult {
                    success: false,
                    attempts_made: 1,
                    strategy_used: RecoveryStrategy::UserFulfillment {
                        questions: self.generate_user_questions(&context.step, original_error),
                    },
                    error_message: Some(format!("User fulfillment failed: {e}")),
                    recovery_time_ms: recovery_time,
                })
            }
        }
    }

    fn strategy_name(&self) -> &'static str {
        "user_fulfillment"
    }

    fn is_applicable(&self, _step: &DynamicStep) -> bool {
        // User fulfillment is only applicable when enabled
        self.enabled
    }
}

impl Default for UserFulfillmentStrategy {
    fn default() -> Self {
        Self::new()
    }
}
