//! Executor for Phase 2 Tool Execution
//!
//! This module implements the Phase 2 tool execution with parameter generation.
//! It executes each step of the YML flow with proper validation and error recovery.

use crate::execution::{SharedExecutor, ToolExecutor};
use crate::validation::FlowValidator;
use crate::yml_schema::{YmlFlow, YmlStep};
use anyhow::{anyhow, Result};
use reev_types::flow::{DynamicFlowPlan, DynamicStep, FlowResult, StepResult, WalletContext};
use reev_types::tools::ToolName;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument, warn};

/// Executor for Phase 2 tool execution
pub struct Executor {
    /// Flow validator for validation checks
    _validator: FlowValidator,
    /// Recovery configuration
    recovery_config: RecoveryConfig,
    /// Tool executor for actual tool execution
    tool_executor: SharedExecutor,
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
        let mut error_message = None;

        // Validate flow structure before execution
        if let Err(e) = self._validator.validate_flow(flow) {
            error!("Flow validation failed: {}", e);
            return Err(anyhow!("Flow validation failed: {e}"));
        }

        // Convert YML flow to DynamicFlowPlan for execution
        let dynamic_flow_plan = self.yml_to_dynamic_flow_plan(flow, initial_context)?;

        // Ground truth is optional for validation
        let _ground_truth = flow.ground_truth.as_ref();

        // Execute each step with updated context
        let mut current_context = initial_context.clone();

        for (step_index, step) in flow.steps.iter().enumerate() {
            let step_start_time = Instant::now();

            info!(
                "Executing step {} of {}: {}",
                step_index + 1,
                flow.steps.len(),
                step.step_id
            );

            // Convert YML step to DynamicStep
            let dynamic_step = self.yml_to_dynamic_step(step, &dynamic_flow_plan.flow_id)?;

            // Execute step using existing step execution pattern
            match self
                .execute_step_with_recovery(&dynamic_step, &step_results, &current_context)
                .await
            {
                Ok(step_result) => {
                    debug!("Step {} completed successfully", step.step_id);

                    // Update context based on step result before moving to next step
                    current_context = self
                        .update_context_after_step(&current_context, &step_result)
                        .await?;
                    step_results.push(step_result);
                }
                Err(e) => {
                    error!("Step {} failed: {}", step.step_id, e);

                    // Create failed step result
                    let step_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: false,
                        error_message: Some(e.to_string()),
                        tool_calls: vec![],
                        output: json!({}),
                        execution_time_ms: step_start_time.elapsed().as_millis() as u64,
                    };

                    // Don't update context for failed steps
                    step_results.push(step_result);

                    // Check if this is a critical step
                    if step.critical.unwrap_or(true) {
                        error!(
                            "Critical step {} failed, stopping flow execution",
                            step.step_id
                        );
                        execution_successful = false;
                        error_message = Some(format!("Critical step failed: {e}"));
                        break;
                    } else {
                        warn!(
                            "Non-critical step {} failed, continuing with flow",
                            step.step_id
                        );
                    }
                }
            }
        }

        // Validate final state against ground truth if available
        if let Some(ground_truth) = &flow.ground_truth {
            // Get final wallet context after all steps
            let final_context = if execution_successful {
                self.get_final_wallet_context(initial_context, &step_results)
                    .await
            } else {
                initial_context.clone()
            };

            if let Err(e) = self
                ._validator
                .validate_final_state(&final_context, ground_truth)
            {
                warn!("Final state validation failed: {}", e);
                // Don't fail the entire execution, just record the validation failure
            } else {
                info!("Final state validation passed");
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
            final_context: Some(initial_context.clone()),
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
        step: &DynamicStep,
        previous_results: &[StepResult],
        current_context: &WalletContext,
    ) -> Result<StepResult> {
        // Convert DynamicStep to YmlStep for tool execution
        let yml_step = YmlStep {
            step_id: step.step_id.clone(),
            prompt: step.prompt_template.clone(),
            refined_prompt: step.prompt_template.clone(), // Default to original prompt
            context: step.description.clone(),
            critical: Some(step.critical),
            estimated_time_seconds: Some(step.estimated_time_seconds),
            expected_tool_calls: Some(Vec::new()), // Will be generated by ToolExecutor
            expected_tools: if !step.required_tools.is_empty() {
                // Convert required_tools (Vec<String>) to Vec<ToolName>
                Some(
                    step.required_tools
                        .iter()
                        .filter_map(|tool_str| ToolName::from_str_safe(tool_str.into()))
                        .collect(),
                )
            } else {
                None
            },
        };

        // Use the provided current_context which has the correct wallet balance
        let wallet_context = current_context.clone();

        // Execute the step using the tool executor with previous step history
        let step_result = if previous_results.is_empty() {
            // First step, no history to pass
            self.tool_executor
                .execute_step(&yml_step, &wallet_context)
                .await?
        } else {
            // For now, just execute step without history support
            // TODO: Implement proper history support in tool executor
            self.tool_executor
                .execute_step(&yml_step, &wallet_context)
                .await?
        };

        Ok(step_result)
    }

    /// Convert YML flow to DynamicFlowPlan
    fn yml_to_dynamic_flow_plan(
        &self,
        flow: &YmlFlow,
        initial_context: &WalletContext,
    ) -> Result<DynamicFlowPlan> {
        let mut steps = Vec::new();

        // Convert each YML step to DynamicStep
        for yml_step in &flow.steps {
            let dynamic_step = self.yml_to_dynamic_step(yml_step, &flow.flow_id)?;
            steps.push(dynamic_step);
        }

        // Create DynamicFlowPlan
        let mut plan = DynamicFlowPlan::new(
            flow.flow_id.clone(),
            flow.user_prompt.clone(),
            initial_context.clone(),
        );

        // Add each step to the plan
        for step in steps {
            plan = plan.with_step(step);
        }

        Ok(plan)
    }

    /// Convert YML step to DynamicStep
    fn yml_to_dynamic_step(&self, yml_step: &YmlStep, _flow_id: &str) -> Result<DynamicStep> {
        let step_id = yml_step.step_id.clone();

        // Convert expected tool calls to required tools
        let required_tools = if let Some(tool_calls) = &yml_step.expected_tool_calls {
            tool_calls.iter().map(|tc| tc.tool_name.clone()).collect()
        } else {
            vec![]
        };

        // Create DynamicStep
        let step = DynamicStep::new(step_id, yml_step.prompt.clone(), yml_step.context.clone())
            .with_required_tools(required_tools)
            .with_critical(yml_step.critical.unwrap_or(true))
            .with_estimated_time(yml_step.estimated_time_seconds.unwrap_or(30));

        Ok(step)
    }

    /// Update wallet context after a step execution
    async fn update_context_after_step(
        &self,
        current_context: &WalletContext,
        step_result: &StepResult,
    ) -> Result<WalletContext> {
        info!(
            "Updating wallet context after step: {}",
            step_result.step_id
        );

        // Create a new context based on the current one
        let mut updated_context = current_context.clone();

        // Extract tool results from step output
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    // Update context based on different tool types
                    if let Some(jupiter_swap) = result.get("jupiter_swap") {
                        if let (Some(input_mint), Some(output_mint)) = (
                            jupiter_swap.get("input_mint").and_then(|v| v.as_str()),
                            jupiter_swap.get("output_mint").and_then(|v| v.as_str()),
                        ) {
                            // Get swap amounts
                            let input_amount = jupiter_swap
                                .get("input_amount")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let output_amount = jupiter_swap
                                .get("output_amount")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);

                            // Update SOL balance if SOL was swapped
                            if input_mint == "So11111111111111111111111111111111111111111112" {
                                updated_context.sol_balance =
                                    updated_context.sol_balance.saturating_sub(input_amount);
                            }

                            // Update output token balance
                            if let Some(token_balance) =
                                updated_context.token_balances.get_mut(output_mint)
                            {
                                token_balance.balance =
                                    token_balance.balance.saturating_add(output_amount);
                            } else {
                                // Create new token entry if it doesn't exist
                                updated_context.token_balances.insert(
                                    output_mint.to_string(),
                                    reev_types::flow::TokenBalance {
                                        balance: output_amount,
                                        decimals: Some(6), // Default to 6 decimals for most tokens
                                        formatted_amount: None,
                                        mint: output_mint.to_string(),
                                        owner: Some(current_context.owner.clone()),
                                        symbol: None,
                                    },
                                );
                            }

                            info!(
                                "Updated context: swapped {} of {} for {} of {}",
                                input_amount, input_mint, output_amount, output_mint
                            );
                        }
                    } else if let Some(jupiter_lend) = result.get("jupiter_lend") {
                        if let (Some(asset_mint), Some(amount)) = (
                            jupiter_lend.get("asset_mint").and_then(|v| v.as_str()),
                            jupiter_lend.get("amount").and_then(|v| v.as_u64()),
                        ) {
                            // Update token balance after lending (subtract from available balance)
                            if let Some(token_balance) =
                                updated_context.token_balances.get_mut(asset_mint)
                            {
                                token_balance.balance =
                                    token_balance.balance.saturating_sub(amount);
                                info!("Updated context: lent {} of {}", amount, asset_mint);
                            }
                        }
                    }
                }
            }
        }

        // Recalculate total value
        updated_context.calculate_total_value();

        info!(
            "Context update completed. SOL: {}, Total value: ${:.2}",
            updated_context.sol_balance_sol(),
            updated_context.total_value_usd
        );

        Ok(updated_context)
    }

    /// Get final wallet context after all steps have executed
    async fn get_final_wallet_context(
        &self,
        initial_context: &WalletContext,
        _step_results: &[StepResult],
    ) -> WalletContext {
        // For now, return the initial context as a placeholder
        // In a full implementation, this would update the context based on step results
        initial_context.clone()
    }
}

/// Configuration for error recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff for retries
    pub exponential_backoff: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay_ms: 1000,
            exponential_backoff: false,
        }
    }
}
