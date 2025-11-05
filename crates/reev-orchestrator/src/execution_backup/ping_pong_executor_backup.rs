//! Ping-Pong Executor for Orchestrator-Agent Coordination
//!
//! This module implements the critical missing coordination mechanism between
//! the orchestrator and agents. It executes flow plans step-by-step with
//! verification, enabling proper multi-step flow completion and partial scoring.

use anyhow::Result;
use reev_types::flow::{DynamicFlowPlan, DynamicStep, StepResult};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};

/// Ping-pong executor that coordinates step-by-step execution
#[derive(Debug)]
pub struct PingPongExecutor {
    /// Track execution progress
    execution_context: crate::execution::ExecutionContext,
    /// Maximum time per step (milliseconds)
    step_timeout_ms: u64,
}

impl PingPongExecutor {
    /// Create new ping-pong executor
    pub fn new(step_timeout_ms: u64) -> Self {
        Self {
            execution_context: crate::execution::ExecutionContext::new(),
            step_timeout_ms,
        }
    }

    /// Execute flow plan with step-by-step ping-pong coordination
    #[instrument(skip_all, fields(
        flow_id = %flow_plan.flow_id,
        total_steps = %flow_plan.steps.len()
    ))]
    pub async fn execute_flow_plan(
        &mut self,
        flow_plan: &DynamicFlowPlan,
        agent_type: &str,
    ) -> Result<Vec<StepResult>> {
        info!(
            "[PingPongExecutor] Starting step-by-step execution: {} steps",
            flow_plan.steps.len()
        );

        let mut step_results = Vec::new();
        let mut completed_steps = 0;

        for (step_index, step) in flow_plan.steps.iter().enumerate() {
            info!(
                "[PingPongExecutor] Executing step {}/{}: {}",
                step_index + 1,
                flow_plan.steps.len(),
                step.step_id
            );

            // Execute current step with agent
            match self
                .execute_single_step(step, agent_type, &step_results)
                .await
            {
                Ok(step_result) => {
                    completed_steps += 1;
                    info!(
                        "[PingPongExecutor] ✅ Step {} completed successfully",
                        step.step_id
                    );

                    // Store result for next steps
                    step_results.push(step_result);
                    self.execution_context
                        .add_step_result(&step.step_id, step_results.last().unwrap());

                    // Check if flow should continue
                    if !self.should_continue_flow(step_results.last().unwrap(), step) {
                        warn!(
                            "[PingPongExecutor] Flow terminated after step {}",
                            step.step_id
                        );
                        break;
                    }
                }
                Err(step_error) => {
                    error!(
                        "[PingPongExecutor] ❌ Step {} failed: {}",
                        step.step_id, step_error
                    );

                    // Create failed step result
                    let failed_result = StepResult {
                        step_id: step.step_id.clone(),
                        success: false,
                        error_message: Some(step_error.to_string()),
                        duration_ms: self.step_timeout_ms,
                        tool_calls: vec![],
                        output: None,
                        recovery_attempts: 0,
                    };

                    step_results.push(failed_result);

                    // Check if this is a critical failure
                    if step.critical {
                        error!(
                            "[PingPongExecutor] Critical step {} failed, aborting flow",
                            step.step_id
                        );
                        break;
                    } else {
                        warn!(
                            "[PingPongExecutor] Non-critical step {} failed, continuing flow",
                            step.step_id
                        );
                        // Continue to next step for non-critical failures
                    }
                }
            }
        }

        let completion_rate = completed_steps as f64 / flow_plan.steps.len() as f64;
        info!(
            "[PingPongExecutor] Flow completed: {}/{} steps ({:.1}% completion)",
            completed_steps,
            flow_plan.steps.len(),
            completion_rate * 100.0
        );

        Ok(step_results)
    }

    /// Execute a single step with agent coordination
    async fn execute_single_step(
        &self,
        step: &DynamicStep,
        agent_type: &str,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        let start_time = std::time::Instant::now();

        // Create context for this step
        let step_context = self.create_step_context(step, previous_results).await?;

        // Execute with timeout
        let step_result = tokio::time::timeout(
            std::time::Duration::from_millis(self.step_timeout_ms),
            self.execute_agent_step(step, agent_type, &step_context),
        )
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "Step {} timed out after {}ms",
                step.step_id,
                self.step_timeout_ms
            )
        })?;

        let duration_ms = start_time.elapsed().as_millis();

        // Add execution timing
        Ok(StepResult {
            duration_ms,
            ..step_result
        })
    }

    /// Create context for current step execution
    async fn create_step_context(
        &self,
        step: &DynamicStep,
        previous_results: &[StepResult],
    ) -> Result<String> {
        let mut context = format!(
            "Executing step: {}\nDescription: {}\n\n",
            step.step_id, step.description
        );

        if !previous_results.is_empty() {
            context.push_str("Previous step results:\n");
            for (i, result) in previous_results.iter().enumerate() {
                context.push_str(&format!(
                    "  Step {}: {} - {}\n",
                    i + 1,
                    result.step_id,
                    if result.success { "SUCCESS" } else { "FAILED" }
                ));
                if let Some(data) = &result.output {
                    context.push_str(&format!("    Data: {}\n", data));
                }
            }
            context.push('\n');
        }

        context.push_str(&format!(
            "Current task: {}\nPlease execute this step and report results.",
            step.prompt_template
        ));

        Ok(context)
    }

    /// Execute agent for a single step
    async fn execute_agent_step(
        &self,
        step: &DynamicStep,
        agent_type: &str,
        context: &str,
    ) -> Result<StepResult> {
        // For now, create a simple mock execution
        // Real agent execution will be integrated after coordination is working
        let _prompt = format!(
            "{}\n\nContext: {}\n\nExecute this specific step and return results.",
            step.prompt_template, context
        );
        info!(
            "[PingPongExecutor] Executing step {} with {} agent",
            step.step_id, agent_type
        );

        // For now, return a mock successful result to test coordination
        // Real agent execution will be integrated after coordination is working
        Ok(StepResult {
            step_id: step.step_id.clone(),
            success: true,
            duration_ms: 3000,
            tool_calls: vec![format!("mock_tool_{}", step.step_id)],
            output: Some(format!("Mock execution for step: {}", step.step_id)),
            error_message: None,
            recovery_attempts: 0,
        })
    }

    /// Determine if flow should continue after a step
    fn should_continue_flow(&self, step_result: &StepResult, step: &DynamicStep) -> bool {
        // Always continue for successful steps
        if step_result.success {
            return true;
        }

        // Continue for non-critical failures
        if !step.critical {
            return true;
        }

        // Stop for critical failures
        false
    }
}

impl Default for PingPongExecutor {
    fn default() -> Self {
        Self::new(30000) // 30 second default timeout per step
    }
}
