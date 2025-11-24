//! Mock Tool Executor for Tests
//!
//! This module provides a mock implementation of tool execution
//! for testing purposes, avoiding the need for actual tool calls.

use anyhow::Result;
use reev_core::yml_schema::YmlStep;
use reev_types::flow::{StepResult, WalletContext};
use serde_json::json;
use tracing::{debug, info};

/// Mock tool executor for testing
pub struct MockToolExecutor {
    /// Whether to simulate success or failure
    simulate_success: bool,
}

impl Default for MockToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockToolExecutor {
    /// Create a new mock tool executor
    pub fn new() -> Self {
        Self {
            simulate_success: true,
        }
    }

    /// Set whether to simulate success or failure
    pub fn with_success(mut self, success: bool) -> Self {
        self.simulate_success = success;
        self
    }

    /// Execute a step with mock tool results
    pub async fn execute_step(
        &self,
        step: &YmlStep,
        _wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        info!("Executing mock step: {}", step.prompt);

        // Simulate tool execution
        let tool_calls = if let Some(expected_calls) = &step.expected_tool_calls {
            expected_calls
                .iter()
                .map(|call| format!("{:?}", call.tool_name))
                .collect()
        } else {
            vec![]
        };

        // Generate mock tool results
        let tool_results = if let Some(expected_calls) = &step.expected_tool_calls {
            expected_calls
                .iter()
                .map(|call| {
                    let tool_name = format!("{:?}", call.tool_name);
                    json!({
                        "tool_name": tool_name,
                        "result": "mock_execution_success",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })
                })
                .collect()
        } else {
            vec![]
        };

        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: self.simulate_success,
            error_message: if self.simulate_success {
                None
            } else {
                Some("Mock execution failure for testing".to_string())
            },
            tool_calls,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 50, // Simulated execution time
        };

        debug!("Mock step execution completed: {:?}", step_result);
        Ok(step_result)
    }
}
