//! Benchmark Types for Score Calculation and Reporting

use chrono::Utc;
use reev_types::flow::FlowResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall benchmark score for a flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScore {
    /// Overall weighted score (0.0-1.0)
    pub overall_score: f64,
    /// Individual category scores
    pub category_scores: HashMap<String, f64>,
    /// Number of assertions that passed
    pub passed_assertions: u32,
    /// Number of assertions that failed
    pub failed_assertions: u32,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of recovery attempts
    pub recovery_attempts: u32,
    /// Whether the minimum score threshold was met
    pub meets_minimum_score: bool,
}

impl Default for BenchmarkScore {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkScore {
    /// Create a new benchmark score
    pub fn new() -> Self {
        Self {
            overall_score: 0.0,
            category_scores: HashMap::new(),
            passed_assertions: 0,
            failed_assertions: 0,
            execution_time_ms: 0,
            recovery_attempts: 0,
            meets_minimum_score: false,
        }
    }

    /// Add a category score
    pub fn add_category(&mut self, category: String, score: f64) {
        self.category_scores.insert(category, score);
    }

    /// Calculate the weighted average score
    pub fn calculate_weighted_average(&mut self, weights: &HashMap<String, f64>) {
        if self.category_scores.is_empty() {
            return;
        }

        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for (category, score) in &self.category_scores {
            let weight = weights.get(category).copied().unwrap_or(1.0);
            total_weighted_score += score * weight;
            total_weight += weight;
        }

        self.overall_score = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        };
    }

    /// Set whether the minimum score threshold was met
    pub fn set_meets_minimum_score(&mut self, meets: bool) {
        self.meets_minimum_score = meets;
    }
}

/// Benchmark categories for evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    /// Final state validation (40% weight)
    FinalState,
    /// Tool call accuracy (30% weight)
    ToolCalls,
    /// Success criteria (20% weight)
    SuccessCriteria,
    /// Performance metrics (10% weight)
    Performance,
}

impl BenchmarkCategory {
    /// Get the default weight for this category
    pub fn default_weight(self) -> f64 {
        match self {
            Self::FinalState => 0.4,
            Self::ToolCalls => 0.3,
            Self::SuccessCriteria => 0.2,
            Self::Performance => 0.1,
        }
    }

    /// Get all benchmark categories
    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::FinalState,
            Self::ToolCalls,
            Self::SuccessCriteria,
            Self::Performance,
        ]
        .into_iter()
    }
}

/// Result of validating a single assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Type of assertion
    pub assertion_type: String,
    /// Whether the assertion passed
    pub passed: bool,
    /// Actual value (if applicable)
    pub actual_value: Option<serde_json::Value>,
    /// Expected value (if applicable)
    pub expected_value: Option<serde_json::Value>,
    /// Error message if validation failed
    pub error_message: Option<String>,
    /// Weight of this assertion in its category
    pub weight: f64,
}

/// Results of validating all assertions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Results of final state assertions
    pub final_state_results: Vec<ValidationResult>,
    /// Results of tool call validations
    pub tool_call_results: Vec<ValidationResult>,
    /// Results of success criteria validations
    pub success_criteria_results: Vec<ValidationResult>,
    /// Results of performance metric validations
    pub performance_results: Vec<ValidationResult>,
}

impl Default for ValidationResults {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResults {
    /// Create new validation results
    pub fn new() -> Self {
        Self {
            final_state_results: Vec::new(),
            tool_call_results: Vec::new(),
            success_criteria_results: Vec::new(),
            performance_results: Vec::new(),
        }
    }

    /// Add a final state validation result
    pub fn add_final_state_result(&mut self, result: ValidationResult) {
        self.final_state_results.push(result);
    }

    /// Add a tool call validation result
    pub fn add_tool_call_result(&mut self, result: ValidationResult) {
        self.tool_call_results.push(result);
    }

    /// Add a success criteria validation result
    pub fn add_success_criteria_result(&mut self, result: ValidationResult) {
        self.success_criteria_results.push(result);
    }

    /// Add a performance validation result
    pub fn add_performance_result(&mut self, result: ValidationResult) {
        self.performance_results.push(result);
    }

    /// Calculate the score for a specific category
    pub fn calculate_category_score(&self, category: BenchmarkCategory) -> f64 {
        let (results, _): (&[ValidationResult], _) = match category {
            BenchmarkCategory::FinalState => (&self.final_state_results, "final_state"),
            BenchmarkCategory::ToolCalls => (&self.tool_call_results, "tool_calls"),
            BenchmarkCategory::SuccessCriteria => {
                (&self.success_criteria_results, "success_criteria")
            }
            BenchmarkCategory::Performance => (&self.performance_results, "performance"),
        };

        if results.is_empty() {
            return 0.0;
        }

        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;

        for result in results {
            total_weight += result.weight;
            weighted_score += if result.passed { result.weight } else { 0.0 };
        }

        if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        }
    }
}

/// Result of scoring a flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredResult {
    /// Original flow result
    pub flow_result: FlowResult,
    /// Benchmark score
    pub score: BenchmarkScore,
    /// Detailed validation results
    pub validation_results: ValidationResults,
    /// Timestamp of the evaluation
    pub timestamp: chrono::DateTime<Utc>,
}

/// Complete benchmark report for a flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Flow ID that was evaluated
    pub flow_id: String,
    /// Execution ID for this specific run
    pub execution_id: String,
    /// Original prompt
    pub prompt: String,
    /// Overall score for this execution
    pub overall_score: f64,
    /// Detailed category scores
    pub category_scores: HashMap<String, f64>,
    /// Detailed validation results
    pub validation_results: ValidationResults,
    /// Execution metrics
    pub execution_metrics: ExecutionMetrics,
    /// Improvement suggestions
    pub improvement_suggestions: Vec<String>,
    /// Timestamp of the report
    pub timestamp: chrono::DateTime<Utc>,
}

impl Default for BenchmarkReport {
    fn default() -> Self {
        Self {
            flow_id: String::new(),
            execution_id: String::new(),
            prompt: String::new(),
            overall_score: 0.0,
            category_scores: HashMap::new(),
            validation_results: ValidationResults::default(),
            execution_metrics: ExecutionMetrics::default(),
            improvement_suggestions: Vec::new(),
            timestamp: Utc::now(),
        }
    }
}

/// Execution metrics captured during benchmarking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Number of steps executed
    pub steps_executed: u32,
    /// Number of tool calls made
    pub tool_calls_made: u32,
    /// Number of successful tool calls
    pub successful_tool_calls: u32,
    /// Number of recovery attempts
    pub recovery_attempts: u32,
    /// Memory usage in bytes (if available)
    pub memory_usage_bytes: Option<u64>,
    /// CPU usage percentage (if available)
    pub cpu_usage_percent: Option<f32>,
}
