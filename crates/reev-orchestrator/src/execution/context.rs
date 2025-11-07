//! Execution Context for Step-by-Step Flow Tracking
//!
//! This module provides context management for tracking progress
//! through multi-step flows with partial completion support.

use reev_types::flow::StepResult;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Execution context for tracking flow progress
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Start time of the overall execution
    start_time: Instant,
    /// Results from completed steps
    step_results: HashMap<String, StepResult>,
    /// Current step index
    current_step_index: usize,
    /// Total number of steps in the flow
    total_steps: usize,
    /// Accumulated execution context data
    accumulated_data: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            step_results: HashMap::new(),
            current_step_index: 0,
            total_steps: 0,
            accumulated_data: HashMap::new(),
        }
    }

    /// Create new execution context with known total steps
    pub fn with_total_steps(total_steps: usize) -> Self {
        Self {
            start_time: Instant::now(),
            step_results: HashMap::new(),
            current_step_index: 0,
            total_steps,
            accumulated_data: HashMap::new(),
        }
    }

    /// Add a step result to the context
    pub fn add_step_result(&mut self, step_id: &str, result: &StepResult) {
        self.step_results
            .insert(step_id.to_string(), result.clone());
        self.current_step_index += 1;

        // Extract useful data from step result for future steps
        if !result.output.is_null() {
            if let Ok(parsed) =
                serde_json::from_str::<serde_json::Value>(&result.output.to_string())
            {
                // Store transaction signatures, balances, etc.
                if let Some(transactions) = parsed.get("transactions").and_then(|t| t.as_array()) {
                    for tx in transactions {
                        if let Some(signature) = tx.get("signature").and_then(|s| s.as_str()) {
                            let tx_key = format!("tx_{step_id}");
                            self.accumulated_data
                                .insert(tx_key, serde_json::json!(signature));
                        }
                        if let Some(amount) = tx.get("amount") {
                            let amount_key = format!("amount_{step_id}");
                            self.accumulated_data.insert(amount_key, amount.clone());
                        }
                    }
                }
            }
        }

        debug!(
            "[ExecutionContext] Added step result for {}: {} ({} ms)",
            step_id,
            if result.success { "SUCCESS" } else { "FAILED" },
            result.execution_time_ms
        );
    }

    /// Get result for a specific step
    pub fn get_step_result(&self, step_id: &str) -> Option<&StepResult> {
        self.step_results.get(step_id)
    }

    /// Get all completed step results
    pub fn get_all_results(&self) -> &HashMap<String, StepResult> {
        &self.step_results
    }

    /// Get the number of completed steps
    pub fn completed_steps(&self) -> usize {
        self.step_results.len()
    }

    /// Get the total number of steps
    pub fn total_steps(&self) -> usize {
        self.total_steps
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.completed_steps() as f64 / self.total_steps as f64) * 100.0
        }
    }

    /// Get current step index
    pub fn current_step_index(&self) -> usize {
        self.current_step_index
    }

    /// Get elapsed time since execution started
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get accumulated data from previous steps
    pub fn get_accumulated_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.accumulated_data.get(key)
    }

    /// Store arbitrary data for future steps
    pub fn store_data(&mut self, key: String, value: serde_json::Value) {
        let key_str = key.clone();
        self.accumulated_data.insert(key, value);
        debug!("[ExecutionContext] Stored data for key: {}", key_str);
    }

    /// Check if a previous step was successful
    pub fn was_step_successful(&self, step_id: &str) -> bool {
        self.step_results
            .get(step_id)
            .map(|r| r.success)
            .unwrap_or(false)
    }

    /// Get execution summary for logging
    pub fn get_summary(&self) -> String {
        let successful_steps = self.step_results.values().filter(|r| r.success).count();
        let failed_steps = self.step_results.len() - successful_steps;

        format!(
            "Progress: {}/{} steps ({:.1}% completion) | {} successful, {} failed | Elapsed: {}ms",
            self.completed_steps(),
            self.total_steps,
            self.completion_percentage(),
            successful_steps,
            failed_steps,
            self.elapsed_time().as_millis()
        )
    }

    /// Calculate flow score based on step completion
    pub fn calculate_flow_score(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }

        // Base score from completion percentage
        let completion_score = self.completed_steps() as f64 / self.total_steps as f64;

        // No bonus for now, just use completion score
        let critical_bonus = 0.0;

        // Overall score (0.0 to 1.0)
        (completion_score + critical_bonus).min(1.0)
    }

    /// Reset context for new execution
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.step_results.clear();
        self.current_step_index = 0;
        self.accumulated_data.clear();
        info!("[ExecutionContext] Reset for new execution");
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for StepResult to check critical status
pub trait StepResultExt {
    /// Check if this step result represents a critical failure
    fn is_critical(&self) -> bool;
}

impl StepResultExt for StepResult {
    fn is_critical(&self) -> bool {
        // For now, assume all failures are critical unless explicitly marked otherwise
        // This can be enhanced to store critical flag in StepResult itself
        !self.success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_execution_context_tracking() {
        let mut ctx = ExecutionContext::with_total_steps(3);

        // Add first successful step
        let step1 = reev_types::flow::StepResult {
            step_id: "step1".to_string(),
            success: true,
            execution_time_ms: 1000,
            tool_calls: vec![reev_constants::JUPITER_SWAP.to_string()],
            output: serde_json::json!({"transactions": [{"signature": "abc123"}]}),
            error_message: None,
        };
        ctx.add_step_result("step1", &step1);

        assert_eq!(ctx.completed_steps(), 1);
        assert_eq!(ctx.completion_percentage(), 33.33333333333333);
        assert!(ctx.was_step_successful("step1"));

        // Add second failed step
        let step2 = reev_types::flow::StepResult {
            step_id: "step2".to_string(),
            success: false,
            execution_time_ms: 500,
            tool_calls: vec![],
            output: serde_json::Value::Null,
            error_message: Some("Failed".to_string()),
        };
        ctx.add_step_result("step2", &step2);

        assert_eq!(ctx.completed_steps(), 2);
        assert_eq!(ctx.completion_percentage(), 66.66666666666666);
        assert!(!ctx.was_step_successful("step2"));
        assert_eq!(ctx.calculate_flow_score(), 0.6666666666666666); // 2/3 completed
    }

    #[test]
    fn test_accumulated_data() {
        let mut ctx = ExecutionContext::new();

        // Store some data
        ctx.store_data("swap_amount".to_string(), serde_json::json!(1000.0));
        ctx.store_data("signature".to_string(), serde_json::json!("abc123"));

        assert_eq!(
            ctx.get_accumulated_data("swap_amount"),
            Some(&serde_json::json!(1000.0))
        );
        assert_eq!(
            ctx.get_accumulated_data("signature"),
            Some(&serde_json::json!("abc123"))
        );
        assert_eq!(ctx.get_accumulated_data("missing"), None);
    }
}
