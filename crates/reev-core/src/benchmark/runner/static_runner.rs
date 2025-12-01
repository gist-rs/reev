//! Static Benchmark Runner for Protocol Interface Integration
//!
//! This module provides a static benchmark runner that executes predefined
//! benchmark flows using the protocol interface. It integrates with the
//! protocol registry to select appropriate protocols for each operation.

use crate::benchmark::runner::types::{Flow, FlowStep};
use crate::benchmark::types::{
    BenchmarkCategory, BenchmarkReport, BenchmarkScore, ExecutionMetrics, ValidationResult,
    ValidationResults,
};
use crate::protocols::{OperationType, ProtocolOperation, ProtocolRegistry};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

/// Static benchmark runner
///
/// This runner executes predefined benchmark flows using the protocol interface.
/// It loads flows from YML files and executes them with monitoring.
pub struct StaticBenchmarkRunner {
    /// Protocol registry for operation execution
    protocol_registry: ProtocolRegistry,
}

impl Default for StaticBenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticBenchmarkRunner {
    /// Create a new static benchmark runner
    pub fn new() -> Self {
        let mut runner = Self {
            protocol_registry: ProtocolRegistry::new(),
        };

        // Register Jupiter protocol (placeholder for now)
        // In a real implementation, this would register all available protocols
        runner.register_default_protocols();
        runner
    }

    /// Register default protocols
    fn register_default_protocols(&mut self) {
        use crate::protocols::jupiter::JupiterProtocol;

        let jupiter = JupiterProtocol::new();
        self.protocol_registry
            .register_with_operations(
                "jupiter",
                jupiter,
                &[
                    OperationType::Swap,
                    OperationType::Lend,
                    OperationType::Earn,
                ],
            )
            .expect("Failed to register Jupiter protocol");
    }

    /// Execute a benchmark flow
    ///
    /// # Arguments
    /// * `flow` - The flow to execute
    ///
    /// # Returns
    /// Result containing the benchmark report or an error
    pub async fn execute_flow(&self, flow: &Flow) -> Result<BenchmarkReport, BenchmarkError> {
        let start_time = Instant::now();
        let execution_id = uuid::Uuid::new_v4().to_string();

        // Initialize metrics
        let mut metrics = ExecutionMetrics::default();

        // Initialize validation results
        let mut validation_results = ValidationResults::new();

        // Initialize benchmark score
        let mut benchmark_score = BenchmarkScore::new();

        // Execute each step in the flow
        let mut step_count = 0;
        let mut tool_call_count = 0;
        let mut successful_tool_calls = 0;

        for step in &flow.steps {
            step_count += 1;

            // Extract operation from the step
            let operation = self.extract_operation_from_step(step)?;

            // Get protocol for this operation
            let protocol = self
                .protocol_registry
                .get_for_operation(&operation.operation_type)
                .ok_or_else(|| {
                    BenchmarkError::NoProtocolForOperation(operation.operation_type.clone())
                })?;

            // Execute the operation
            let result = protocol.execute(&operation).await;

            tool_call_count += 1;

            // Handle execution result
            match result {
                Ok(_) => {
                    successful_tool_calls += 1;

                    // Add a successful tool call validation
                    validation_results.add_tool_call_result(ValidationResult {
                        assertion_type: "tool_call_success".to_string(),
                        passed: true,
                        actual_value: Some(json!(true)),
                        expected_value: Some(json!(true)),
                        error_message: None,
                        weight: 1.0,
                    });
                }
                Err(e) => {
                    // Add a failed tool call validation
                    validation_results.add_tool_call_result(ValidationResult {
                        assertion_type: "tool_call_success".to_string(),
                        passed: false,
                        actual_value: Some(json!(false)),
                        expected_value: Some(json!(true)),
                        error_message: Some(format!("Protocol execution failed: {e}")),
                        weight: 1.0,
                    });
                }
            }
        }

        // Update execution metrics
        metrics.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        metrics.steps_executed = step_count;
        metrics.tool_calls_made = tool_call_count;
        metrics.successful_tool_calls = successful_tool_calls;

        // Calculate category scores
        let mut weights = HashMap::new();
        for category in BenchmarkCategory::all() {
            weights.insert(format!("{category:?}"), category.default_weight());
        }

        // Calculate scores for each category
        for category in BenchmarkCategory::all() {
            let score = validation_results.calculate_category_score(category);
            benchmark_score.add_category(format!("{category:?}"), score);
        }

        // Calculate weighted average score
        benchmark_score.calculate_weighted_average(&weights);

        // Set score metrics
        benchmark_score.execution_time_ms = metrics.total_execution_time_ms;
        benchmark_score.passed_assertions = validation_results
            .final_state_results
            .iter()
            .chain(validation_results.tool_call_results.iter())
            .chain(validation_results.success_criteria_results.iter())
            .chain(validation_results.performance_results.iter())
            .filter(|r| r.passed)
            .count() as u32;

        benchmark_score.failed_assertions = validation_results
            .final_state_results
            .iter()
            .chain(validation_results.tool_call_results.iter())
            .chain(validation_results.success_criteria_results.iter())
            .chain(validation_results.performance_results.iter())
            .filter(|r| !r.passed)
            .count() as u32;

        // Create benchmark report
        let report = BenchmarkReport {
            flow_id: flow.id.clone(),
            execution_id,
            prompt: flow.prompt.clone().unwrap_or_default(),
            overall_score: benchmark_score.overall_score,
            category_scores: benchmark_score.category_scores.clone(),
            validation_results,
            execution_metrics: metrics,
            improvement_suggestions: vec![], // TODO: Implement suggestion generation
            timestamp: chrono::Utc::now(),
        };

        Ok(report)
    }

    /// Extract a protocol operation from a flow step
    ///
    /// # Arguments
    /// * `step` - The flow step to extract from
    ///
    /// # Returns
    /// Result containing the protocol operation or an error
    fn extract_operation_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<ProtocolOperation, BenchmarkError> {
        // Extract operation type from step
        let operation_type = self.extract_operation_type_from_step(step)?;

        // Extract parameters from step
        let parameters = self.extract_parameters_from_step(step)?;

        Ok(ProtocolOperation {
            operation_type,
            parameters,
        })
    }

    /// Extract operation type from a flow step
    fn extract_operation_type_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<OperationType, BenchmarkError> {
        // This is a simplified extraction - in a real implementation,
        // this would parse the step's refined_prompt or context
        let context = step
            .context.as_deref()
            .unwrap_or(&step.refined_prompt);

        if context.contains("swap") {
            Ok(OperationType::Swap)
        } else if context.contains("lend") {
            Ok(OperationType::Lend)
        } else if context.contains("earn") {
            Ok(OperationType::Earn)
        } else if context.contains("transfer") {
            Ok(OperationType::Transfer)
        } else if context.contains("stake") {
            Ok(OperationType::Stake)
        } else {
            Err(BenchmarkError::InvalidStepFormat(format!(
                "Cannot determine operation type from step: {}",
                step.step_id
            )))
        }
    }

    /// Extract parameters from a flow step
    fn extract_parameters_from_step(
        &self,
        step: &FlowStep,
    ) -> Result<HashMap<String, Value>, BenchmarkError> {
        let mut parameters = HashMap::new();

        // Extract parameters from step's refined_prompt
        // This is a simplified extraction - in a real implementation,
        // this would parse the step's context and refined_prompt more thoroughly

        if let Some(context) = &step.context {
            // For swap operations
            if context.contains("swap") {
                parameters.insert(
                    "input_mint".to_string(),
                    json!("So11111111111111111111111111111111111111112"),
                );
                parameters.insert(
                    "output_mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(1000000)); // 1 SOL in lamports
                }
            }

            // For lend operations
            if context.contains("lend") {
                parameters.insert(
                    "mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(10000000)); // 10 USDC
                }
            }

            // For earn operations
            if context.contains("earn") {
                parameters.insert(
                    "mint".to_string(),
                    json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                );

                // Extract amount from context
                if let Some(amount) = extract_amount_from_text(context) {
                    parameters.insert("amount".to_string(), json!(amount));
                } else {
                    // Default amount if not found
                    parameters.insert("amount".to_string(), json!(5000000)); // 5 USDC
                }
            }
        }

        Ok(parameters)
    }
}

/// Error types for benchmark runner
#[derive(thiserror::Error, Debug)]
pub enum BenchmarkError {
    /// No protocol available for operation
    #[error("No protocol available for operation: {0}")]
    NoProtocolForOperation(OperationType),

    /// Invalid step format
    #[error("Invalid step format: {0}")]
    InvalidStepFormat(String),

    /// Protocol execution error
    #[error("Protocol execution error: {0}")]
    ProtocolExecutionError(String),

    /// Flow execution error
    #[error("Flow execution error: {0}")]
    FlowExecutionError(String),
}

/// Helper function to extract amount from text
///
/// # Arguments
/// * `text` - Text to extract amount from
///
/// # Returns
/// Option containing the amount if found
fn extract_amount_from_text(text: &str) -> Option<u64> {
    use regex::Regex;

    // Look for patterns like "1 SOL", "10 USDC", etc.
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*([A-Za-z]+)").ok()?;
    let caps = re.captures(text)?;

    let amount_str = caps.get(1)?.as_str();
    let token = caps.get(2)?.as_str();

    // Parse amount
    let amount = amount_str.parse::<f64>().ok()?;

    // Convert to lamports based on token
    let lamports = match token.to_lowercase().as_str() {
        "sol" => (amount * 1_000_000_000.0) as u64,
        "usdc" => (amount * 1_000_000.0) as u64,
        _ => return None,
    };

    Some(lamports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::runner::types::{Flow, FlowStep};

    #[test]
    fn test_extract_amount_from_text() {
        assert_eq!(
            extract_amount_from_text("swap 1 SOL to USDC"),
            Some(1_000_000_000)
        );
        assert_eq!(extract_amount_from_text("lend 10 USDC"), Some(10_000_000));
        assert_eq!(extract_amount_from_text("earn 0.5 SOL"), Some(500_000_000));
        assert_eq!(extract_amount_from_text("invalid amount"), None);
    }

    #[test]
    fn test_extract_operation_type_from_step() {
        let runner = StaticBenchmarkRunner::new();

        let swap_step = FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "swap 1 SOL to USDC".to_string(),
            context: Some("swap 1 SOL to USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        };

        assert_eq!(
            runner.extract_operation_type_from_step(&swap_step).unwrap(),
            OperationType::Swap
        );

        let lend_step = FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "lend 10 USDC".to_string(),
            context: Some("lend 10 USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        };

        assert_eq!(
            runner.extract_operation_type_from_step(&lend_step).unwrap(),
            OperationType::Lend
        );
    }

    #[test]
    fn test_extract_parameters_from_step() {
        let runner = StaticBenchmarkRunner::new();

        let swap_step = FlowStep {
            step_id: "1".to_string(),
            refined_prompt: "swap 1 SOL to USDC".to_string(),
            context: Some("swap 1 SOL to USDC".to_string()),
            critical: false,
            expected_tools: vec![],
        };

        let parameters = runner.extract_parameters_from_step(&swap_step).unwrap();
        assert_eq!(
            parameters.get("input_mint"),
            Some(&json!("So11111111111111111111111111111111111111112"))
        );
        assert_eq!(
            parameters.get("output_mint"),
            Some(&json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"))
        );
        assert_eq!(parameters.get("amount"), Some(&json!(1_000_000_000)));
    }

    #[tokio::test]
    async fn test_execute_flow() {
        let runner = StaticBenchmarkRunner::new();

        let flow = Flow {
            id: "test-flow".to_string(),
            prompt: Some("swap 1 SOL to USDC".to_string()),
            created_at: chrono::Utc::now(),
            subject_wallet_info: None,
            steps: vec![FlowStep {
                step_id: "1".to_string(),
                refined_prompt: "swap 1 SOL to USDC".to_string(),
                context: Some("swap 1 SOL to USDC".to_string()),
                critical: false,
                expected_tools: vec![],
            }],
            ground_truth: None,
        };

        let report = runner.execute_flow(&flow).await.unwrap();
        assert_eq!(report.flow_id, "test-flow");
        assert_eq!(report.prompt, "swap 1 SOL to USDC");
        assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
    }
}
