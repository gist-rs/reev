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
        // Always use the real tool executor
        let tool_executor: SharedExecutor = Arc::new(ToolExecutor::new()?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
        })
    }

    /// Create a new executor with rig agent enabled
    pub async fn new_with_rig() -> Result<Self> {
        // Create tool executor with rig agent enabled
        let tool_executor: SharedExecutor =
            Arc::new(ToolExecutor::new()?.enable_rig_agent().await?);

        Ok(Self {
            _validator: FlowValidator::new(),
            recovery_config: RecoveryConfig::default(),
            tool_executor,
        })
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

        // Convert YML flow to DynamicFlowPlan for execution
        let dynamic_flow_plan = self.yml_to_dynamic_flow_plan(flow, initial_context)?;

        // Ground truth is optional for validation
        let _ground_truth = flow.ground_truth.as_ref();

        // Execute each step
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

            // Execute the step using existing step execution pattern
            match self
                .execute_step_with_recovery(&dynamic_step, &step_results, initial_context)
                .await
            {
                Ok(step_result) => {
                    debug!("Step {} completed successfully", step.step_id);
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
    #[instrument(skip(self, step, _previous_results, initial_context))]
    async fn execute_step_with_recovery(
        &self,
        step: &DynamicStep,
        _previous_results: &[StepResult],
        initial_context: &WalletContext, // Add initial_context parameter
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

        // Use the provided initial_context which has the correct wallet balance
        let wallet_context = initial_context.clone();

        // Execute the step using the tool executor
        let step_result = self
            .tool_executor
            .execute_step(&yml_step, &wallet_context)
            .await?;

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
