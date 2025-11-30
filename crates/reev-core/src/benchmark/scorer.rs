//! Benchmark Scorer for Evaluating AI-Generated DeFi Flows
//!
//! This module provides the implementation for scoring execution results
//! against ground truth expectations, following the benchmark criteria
//! outlined in PLAN_CORE_BENCHMARK.md.

use crate::yml_schema::{YmlFlow, YmlGroundTruth};
use anyhow::{anyhow, Result};
use chrono::Utc;
use reev_types::flow::FlowResult;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

use super::types::{
    BenchmarkCategory, BenchmarkReport, BenchmarkScore, ExecutionMetrics, ScoredResult,
    ValidationResult, ValidationResults,
};

/// Scorer for benchmarking flow execution results
pub struct BenchmarkScorer {
    /// Minimum score threshold for passing benchmarks
    min_score_threshold: f64,
    /// Category weights for score calculation
    category_weights: HashMap<String, f64>,
}

impl Default for BenchmarkScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkScorer {
    /// Create a new benchmark scorer with default configuration
    pub fn new() -> Self {
        let mut category_weights = HashMap::new();

        // Set default category weights according to PLAN_CORE_BENCHMARK.md
        category_weights.insert("final_state".to_string(), 0.4); // 40% weight
        category_weights.insert("tool_calls".to_string(), 0.3); // 30% weight
        category_weights.insert("success_criteria".to_string(), 0.2); // 20% weight
        category_weights.insert("performance".to_string(), 0.1); // 10% weight

        Self {
            min_score_threshold: 0.7, // Default minimum score
            category_weights,
        }
    }

    /// Create a scorer with custom minimum score threshold
    pub fn with_min_score_threshold(mut self, threshold: f64) -> Self {
        self.min_score_threshold = threshold;
        self
    }

    /// Score a flow execution result against its ground truth
    #[instrument(skip(self, flow, flow_result))]
    pub async fn score_flow_execution(
        &self,
        flow: &YmlFlow,
        flow_result: &FlowResult,
        execution_start_time: Instant,
    ) -> Result<ScoredResult> {
        info!("Scoring flow execution for flow: {}", flow.flow_id);

        // Calculate execution time
        let execution_time_ms = execution_start_time.elapsed().as_millis() as u64;

        // Get ground truth from flow
        let ground_truth = match &flow.ground_truth {
            Some(gt) => gt,
            None => {
                error!("No ground truth found for flow: {}", flow.flow_id);
                return Err(anyhow!("No ground truth found for flow: {}", flow.flow_id));
            }
        };

        // Initialize validation results
        let mut validation_results = ValidationResults::new();

        // Validate final state assertions (40% of total score)
        debug!("Validating final state assertions");
        let final_state_score = self
            .validate_final_state(&ground_truth.final_state_assertions, flow_result)
            .await?;

        for result in final_state_score {
            validation_results.add_final_state_result(result);
        }

        // Validate tool calls (30% of total score)
        debug!("Validating tool calls");
        let tool_calls_score = self
            .validate_tool_calls(ground_truth.expected_tool_calls.as_deref(), flow_result)
            .await?;

        for result in tool_calls_score {
            validation_results.add_tool_call_result(result);
        }

        // Validate success criteria (20% of total score)
        debug!("Validating success criteria");
        let success_criteria_score = self
            .validate_success_criteria(ground_truth.success_criteria.as_deref(), flow_result)
            .await?;

        for result in success_criteria_score {
            validation_results.add_success_criteria_result(result);
        }

        // Validate performance metrics (10% of total score)
        debug!("Validating performance metrics");
        let performance_score = self
            .validate_performance_metrics(ground_truth, flow_result, execution_time_ms)
            .await?;

        for result in performance_score {
            validation_results.add_performance_result(result);
        }

        // Calculate overall benchmark score
        let mut benchmark_score = BenchmarkScore::new();
        benchmark_score.execution_time_ms = execution_time_ms;

        // Calculate category scores
        for category in BenchmarkCategory::all() {
            let category_name = match category {
                BenchmarkCategory::FinalState => "final_state",
                BenchmarkCategory::ToolCalls => "tool_calls",
                BenchmarkCategory::SuccessCriteria => "success_criteria",
                BenchmarkCategory::Performance => "performance",
            };

            let category_score = validation_results.calculate_category_score(category);
            benchmark_score.add_category(category_name.to_string(), category_score);
        }

        // Calculate weighted average
        benchmark_score.calculate_weighted_average(&self.category_weights);

        // Check if minimum score is met
        let min_score = ground_truth.min_score.unwrap_or(self.min_score_threshold);
        benchmark_score.set_meets_minimum_score(benchmark_score.overall_score >= min_score);

        // Count passed and failed assertions
        let mut passed_assertions = 0;
        let mut failed_assertions = 0;

        for result in &validation_results.final_state_results {
            if result.passed {
                passed_assertions += 1;
            } else {
                failed_assertions += 1;
            }
        }

        benchmark_score.passed_assertions = passed_assertions;
        benchmark_score.failed_assertions = failed_assertions;

        // Create scored result
        let scored_result = ScoredResult {
            flow_result: flow_result.clone(),
            score: benchmark_score,
            validation_results,
            timestamp: Utc::now(),
        };

        info!(
            "Flow execution scored: {:.2} (min required: {:.2})",
            scored_result.score.overall_score, min_score
        );

        Ok(scored_result)
    }

    /// Generate a benchmark report from a scored result
    pub fn generate_report(
        &self,
        flow: &YmlFlow,
        scored_result: &ScoredResult,
        execution_metrics: ExecutionMetrics,
    ) -> BenchmarkReport {
        // Generate improvement suggestions based on validation results
        let mut improvement_suggestions = Vec::new();

        // Check final state validation failures
        for result in &scored_result.validation_results.final_state_results {
            if !result.passed {
                if let Some(error) = &result.error_message {
                    improvement_suggestions
                        .push(format!("Final state assertion failed: {error}"));
                }
            }
        }

        // Check tool call validation failures
        for result in &scored_result.validation_results.tool_call_results {
            if !result.passed {
                if let Some(error) = &result.error_message {
                    improvement_suggestions.push(format!("Tool call validation failed: {error}"));
                }
            }
        }

        // Check success criteria failures
        for result in &scored_result.validation_results.success_criteria_results {
            if !result.passed {
                if let Some(error) = &result.error_message {
                    improvement_suggestions.push(format!("Success criteria failed: {error}"));
                }
            }
        }

        // Check performance validation failures
        for result in &scored_result.validation_results.performance_results {
            if !result.passed {
                if let Some(error) = &result.error_message {
                    improvement_suggestions.push(format!("Performance metric failed: {error}"));
                }
            }
        }

        // Add general suggestions if score is low
        if scored_result.score.overall_score < 0.8 {
            improvement_suggestions.push(
                "Consider optimizing tool call sequence to reduce execution time".to_string(),
            );
        }

        if scored_result.score.overall_score < 0.6 {
            improvement_suggestions.push("Review parameter accuracy in tool calls".to_string());
        }

        BenchmarkReport {
            flow_id: flow.flow_id.clone(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            prompt: flow.user_prompt.clone(),
            overall_score: scored_result.score.overall_score,
            category_scores: scored_result.score.category_scores.clone(),
            validation_results: scored_result.validation_results.clone(),
            execution_metrics,
            improvement_suggestions,
            timestamp: Utc::now(),
        }
    }

    /// Validate final state assertions against execution result
    async fn validate_final_state(
        &self,
        assertions: &[crate::yml_schema::YmlAssertion],
        flow_result: &FlowResult,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for assertion in assertions {
            let mut passed = false;
            let mut error_message = None;

            // For now, we'll implement a simplified validation
            // In a full implementation, this would check actual wallet state
            // after execution against the expected state in assertions

            match assertion.assertion_type.as_str() {
                "SolBalanceChange" => {
                    // For now, we'll assume this passes if the flow succeeded
                    // In a full implementation, we'd check actual balance changes
                    passed = flow_result.success;
                    if !passed {
                        error_message = Some("Sol balance change assertion failed".to_string());
                    }
                }
                "TokenAccountBalance" => {
                    // For now, we'll assume this passes if the flow succeeded
                    // In a full implementation, we'd check actual token balances
                    passed = flow_result.success;
                    if !passed {
                        error_message = Some("Token account balance assertion failed".to_string());
                    }
                }
                _ => {
                    error_message = Some(format!(
                        "Unknown assertion type: {}",
                        assertion.assertion_type
                    ));
                }
            }

            results.push(ValidationResult {
                assertion_type: assertion.assertion_type.clone(),
                passed,
                actual_value: None,   // Would be filled in a full implementation
                expected_value: None, // Would be filled in a full implementation
                error_message,
                weight: 1.0, // Default weight
            });
        }

        Ok(results)
    }

    /// Validate tool calls against expectations
    async fn validate_tool_calls(
        &self,
        expected_tool_calls: Option<&[crate::yml_schema::YmlToolCall]>,
        flow_result: &FlowResult,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        if let Some(expected_calls) = expected_tool_calls {
            for expected_call in expected_calls {
                // For now, we'll do a simplified validation
                // In a full implementation, we'd check actual tool calls made
                // against the expected ones

                let passed = flow_result.success;
                let error_message = if passed {
                    None
                } else {
                    Some(format!(
                        "Expected tool call {} was not successful",
                        expected_call.tool_name
                    ))
                };

                results.push(ValidationResult {
                    assertion_type: "tool_call".to_string(),
                    passed,
                    actual_value: None, // Would be filled in a full implementation
                    expected_value: Some(serde_json::json!(expected_call.tool_name)),
                    error_message,
                    weight: 1.0, // Default weight
                });
            }
        }

        Ok(results)
    }

    /// Validate success criteria against execution result
    async fn validate_success_criteria(
        &self,
        success_criteria: Option<&[crate::yml_schema::YmlSuccessCriterion]>,
        flow_result: &FlowResult,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        if let Some(criteria) = success_criteria {
            for criterion in criteria {
                // For now, we'll do a simplified validation
                // In a full implementation, we'd check specific criteria

                let passed = flow_result.success;
                let error_message = if passed {
                    None
                } else {
                    Some(format!(
                        "Success criterion {} was not met",
                        criterion.criterion_type
                    ))
                };

                results.push(ValidationResult {
                    assertion_type: criterion.criterion_type.clone(),
                    passed,
                    actual_value: None, // Would be filled in a full implementation
                    expected_value: Some(serde_json::json!(criterion.required)),
                    error_message,
                    weight: criterion.weight.unwrap_or(1.0),
                });
            }
        }

        Ok(results)
    }

    /// Validate performance metrics against expectations
    async fn validate_performance_metrics(
        &self,
        _ground_truth: &YmlGroundTruth,
        flow_result: &FlowResult,
        execution_time_ms: u64,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Validate execution time (target: < 10 seconds according to PLAN_CORE_BENCHMARK.md)
        let target_execution_time = 10_000; // 10 seconds in ms
        let execution_time_passed = execution_time_ms <= target_execution_time;
        let execution_time_error = if execution_time_passed {
            None
        } else {
            Some(format!(
                "Execution time {execution_time_ms}ms exceeded target of {target_execution_time}ms"
            ))
        };

        results.push(ValidationResult {
            assertion_type: "execution_time".to_string(),
            passed: execution_time_passed,
            actual_value: Some(serde_json::json!(execution_time_ms)),
            expected_value: Some(serde_json::json!(target_execution_time)),
            error_message: execution_time_error,
            weight: 0.5, // Weight of 0.5 for execution time
        });

        // Validate tool call accuracy (target: > 95% according to PLAN_CORE_BENCHMARK.md)
        let min_tool_call_accuracy = 0.95;
        let tool_call_accuracy = if flow_result.success { 1.0 } else { 0.0 };
        let tool_call_passed = tool_call_accuracy >= min_tool_call_accuracy;
        let tool_call_error = if tool_call_passed {
            None
        } else {
            Some(format!(
                "Tool call accuracy {:.2}% below target of {:.2}%",
                tool_call_accuracy * 100.0,
                min_tool_call_accuracy * 100.0
            ))
        };

        results.push(ValidationResult {
            assertion_type: "tool_call_accuracy".to_string(),
            passed: tool_call_passed,
            actual_value: Some(serde_json::json!(tool_call_accuracy)),
            expected_value: Some(serde_json::json!(min_tool_call_accuracy)),
            error_message: tool_call_error,
            weight: 0.5, // Weight of 0.5 for tool call accuracy
        });

        Ok(results)
    }
}

/// Calculate benchmark score from ground truth and execution result
pub fn calculate_benchmark_score(
    ground_truth: &YmlGroundTruth,
    execution_result: &FlowResult,
    performance_metrics: &ExecutionMetrics,
) -> Result<BenchmarkScore> {
    // This function is kept for backward compatibility
    // New code should use BenchmarkScorer::score_flow_execution

    let scorer = BenchmarkScorer::new();
    let execution_start_time = Instant::now()
        - std::time::Duration::from_millis(performance_metrics.total_execution_time_ms);

    let flow = YmlFlow::new(
        uuid::Uuid::new_v4().to_string(),
        "dummy".to_string(),
        crate::yml_schema::YmlWalletInfo::new("dummy".to_string(), 0),
    )
    .with_ground_truth(ground_truth.clone());

    let rt = tokio::runtime::Runtime::new()?;

    let scored_result = rt.block_on(async {
        scorer
            .score_flow_execution(&flow, execution_result, execution_start_time)
            .await
    })?;

    Ok(scored_result.score)
}
