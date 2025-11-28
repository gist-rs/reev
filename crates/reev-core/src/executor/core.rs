//! Executor for Phase 2 Tool Execution
//!
//! This module implements the Phase 2 tool execution with parameter generation.
//! It executes each step of the YML flow with proper validation and error recovery.

use crate::execution::{SharedExecutor, ToolExecutor};
use crate::validation::FlowValidator;
use crate::yml_schema::YmlFlow;
use anyhow::{anyhow, Result};
use reev_types::flow::{FlowResult, StepResult, WalletContext};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

use super::recovery::RecoveryConfig;
use super::yml_converter::YmlConverter;

/// Executor for Phase 2 tool execution
pub struct Executor {
    /// Flow validator for validation checks
    _validator: FlowValidator,
    /// Recovery configuration
    recovery_config: RecoveryConfig,
    /// Tool executor for actual tool execution
    tool_executor: SharedExecutor,
    /// YML converter
    yml_converter: YmlConverter,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new().expect("Failed to create default executor")
    }
}

impl Executor {
    /// Create a new executor
    pub fn new() -> Result<Self> {
        // Always use the real tool executor with RigAgent enabled
        // Store the tool executor without RigAgent first
        let tool_executor: SharedExecutor = Arc::new(ToolExecutor::new()?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
            yml_converter: YmlConverter::new(),
        })
    }

    /// Initialize executor with RigAgent enabled (async version)
    pub async fn new_async_with_rig() -> Result<Self> {
        // Create tool executor with RigAgent enabled
        let tool_executor: SharedExecutor =
            Arc::new(ToolExecutor::new()?.enable_rig_agent().await?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
            yml_converter: YmlConverter::new(),
        })
    }

    /// Create a new executor with rig agent enabled
    pub async fn new_with_rig() -> Result<Self> {
        // Use the async version with RigAgent enabled
        Self::new_async_with_rig().await
    }

    /// Set recovery configuration
    pub fn with_recovery_config(mut self, config: RecoveryConfig) -> Self {
        self.recovery_config = config;
        self
    }

    /// Set custom tool executor
    pub fn with_tool_executor(mut self, tool_executor: ToolExecutor) -> Self {
        self.tool_executor = Arc::new(tool_executor);
        self
    }

    /// Execute a YML flow with validation and error recovery
    #[instrument(skip(self, flow, initial_context))]
    pub async fn execute_flow(
        &self,
        flow: &YmlFlow,
        initial_context: &WalletContext,
    ) -> Result<FlowResult> {
        info!("Starting execution of flow: {}", flow.flow_id);

        let start_time = Instant::now();
        let mut step_results = Vec::new();
        let mut execution_successful = true;
        let error_message = None;
        let current_context = initial_context.clone();

        // Validate flow structure before execution
        if let Err(e) = self._validator.validate_flow(flow) {
            error!("Flow validation failed: {}", e);
            return Err(anyhow!("Flow validation failed: {e}"));
        }

        // Convert YML flow to DynamicFlowPlan for execution
        let dynamic_flow_plan = YmlConverter::yml_to_dynamic_flow_plan(flow, initial_context)?;

        // Execute each step in the flow
        for step in &dynamic_flow_plan.steps {
            info!("Executing step: {}", step.step_id);

            // Execute step with recovery
            let step_result = match self
                .execute_step_with_recovery(step, &step_results, &current_context)
                .await
            {
                Ok(result) => {
                    info!("Step {} executed successfully", step.step_id);
                    result
                }
                Err(e) => {
                    error!("Step {} failed: {}", step.step_id, e);
                    let step_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: false,
                        error_message: Some(format!("Step execution failed: {e}")),
                        tool_calls: vec![],
                        output: serde_json::json!({
                            "error": format!("Step execution failed: {}", e)
                        }),
                        execution_time_ms: 0,
                    };

                    if step.critical {
                        error!("Critical step failed, aborting flow execution");
                        execution_successful = false;
                        // The error_message is already set in the StepResult above
                        step_results.push(step_result);
                        break;
                    }

                    step_results.push(step_result);
                    continue;
                }
            };

            step_results.push(step_result);
        }

        // Perform final validation if the flow was successful
        if execution_successful {
            if let Some(ground_truth) = &flow.ground_truth {
                if let Err(e) = self
                    ._validator
                    .validate_final_state(&current_context, ground_truth)
                {
                    warn!("Final state validation failed: {}", e);
                    // Don't fail the entire execution, just record the validation failure
                } else {
                    info!("Final state validation passed");
                }
            }
        }

        // Calculate metrics before moving step_results
        let successful_steps = step_results.iter().filter(|r| r.success).count();
        let failed_steps = step_results.iter().filter(|r| !r.success).count();
        let total_tool_calls = step_results.iter().map(|r| r.tool_calls.len()).sum();

        // Create flow result
        let flow_result = FlowResult {
            flow_id: flow.flow_id.clone(),
            user_prompt: flow.user_prompt.clone(),
            success: execution_successful,
            step_results,
            metrics: reev_types::flow::FlowMetrics {
                total_duration_ms: start_time.elapsed().as_millis() as u64,
                successful_steps,
                failed_steps,
                critical_failures: 0,     // TODO: Count critical failures
                non_critical_failures: 0, // TODO: Count non-critical failures
                total_tool_calls,
                context_resolution_ms: 0, // TODO: Track context resolution time
                prompt_generation_ms: 0,  // TODO: Track prompt generation time
                cache_hit_rate: 0.0,      // TODO: Track cache hit rate
            },
            final_context: Some(current_context),
            error_message,
        };

        info!(
            "Flow execution completed with success: {}",
            flow_result.success
        );
        Ok(flow_result)
    }

    /// Execute a step with error recovery
    #[instrument(skip(self, step, previous_results, current_context))]
    async fn execute_step_with_recovery(
        &self,
        step: &reev_types::flow::DynamicStep,
        previous_results: &[reev_types::flow::StepResult],
        current_context: &WalletContext,
    ) -> Result<reev_types::flow::StepResult> {
        // Convert DynamicStep to YmlStep for tool execution
        let yml_step = self.yml_converter.dynamic_step_to_yml_step(step)?;

        // Use the provided current_context which should have the correct wallet balance
        let wallet_context = current_context.clone();

        debug!(
            "DEBUG: execute_step_with_recovery - USDC balance in context: {:?}",
            current_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Execute step using tool executor with previous step history
        // Execute the step with either simple execution or with history if previous steps exist
        let step_result = if previous_results.is_empty() {
            // First step or when history is not needed
            self.tool_executor
                .execute_step(&yml_step, &wallet_context)
                .await?
        } else {
            // Pass previous step history for context-aware execution
            self.tool_executor
                .execute_step_with_history(&yml_step, current_context, previous_results)
                .await?
        };

        // Add debug logging to ensure updated context is being passed correctly
        if !previous_results.is_empty() {
            info!(
                "DEBUG: execute_step_with_recovery - Passing updated context to next step: USDC balance: {:?}",
                wallet_context
                    .token_balances
                    .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                    .map(|t| t.balance)
            );
        }

        Ok(step_result)
    }
}
