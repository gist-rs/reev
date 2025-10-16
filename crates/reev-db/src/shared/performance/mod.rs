//! Shared performance types for reev ecosystem
//!
//! This module contains performance monitoring types that can be used across
//! different projects. These types are designed to be:
//! - Database-friendly (String timestamps, JSON serializable)
//! - Generic enough for different use cases
//! - Compatible with various performance monitoring systems

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    /// Unique identifier
    pub id: Option<i64>,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Type of agent
    pub agent_type: String,
    /// Performance score (0.0 - 1.0)
    pub score: f64,
    /// Final execution status
    pub final_status: String,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<i64>,
    /// When this performance was recorded
    pub timestamp: String,
    /// Reference to the flow log
    pub flow_log_id: Option<i64>,
    /// Reference to the benchmark prompt
    pub prompt_md5: Option<String>,
    /// Additional performance metrics
    pub additional_metrics: HashMap<String, f64>,
}

/// Performance metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: u64,
    /// Total LLM calls
    pub total_llm_calls: u32,
    /// Total tool calls
    pub total_tool_calls: u32,
    /// Total tokens used
    pub total_tokens: u64,
    /// Success rate as percentage
    pub success_rate: f64,
    /// Error rate as percentage
    pub error_rate: f64,
    /// Average score
    pub average_score: f64,
    /// Performance by agent type
    pub performance_by_agent: HashMap<String, AgentStats>,
    /// Performance over time
    pub performance_timeline: Vec<TimelinePoint>,
}

/// Statistics for a specific agent type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    /// Agent type
    pub agent_type: String,
    /// Number of executions
    pub execution_count: u32,
    /// Success count
    pub success_count: u32,
    /// Failure count
    pub failure_count: u32,
    /// Average score
    pub average_score: f64,
    /// Average execution time
    pub average_execution_time_ms: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Success rate
    pub success_rate: f64,
}

/// Performance data point in timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// Timestamp
    pub timestamp: String,
    /// Score at this point
    pub score: f64,
    /// Execution time
    pub execution_time_ms: u64,
    /// Agent type
    pub agent_type: String,
}

/// Performance filter options
#[derive(Debug, Clone, Default)]
pub struct PerformanceFilter {
    /// Filter by agent type
    pub agent_type: Option<String>,
    /// Filter by minimum score
    pub min_score: Option<f64>,
    /// Filter by maximum score
    pub max_score: Option<f64>,
    /// Filter by date range (start)
    pub date_from: Option<String>,
    /// Filter by date range (end)
    pub date_to: Option<String>,
    /// Filter by execution status
    pub final_status: Option<String>,
    /// Limit number of results
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort direction ("asc" or "desc")
    pub sort_direction: Option<String>,
}

impl PerformanceFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by agent type
    pub fn agent_type<S: Into<String>>(mut self, agent_type: S) -> Self {
        self.agent_type = Some(agent_type.into());
        self
    }

    /// Filter by score range
    pub fn score_range(mut self, min: f64, max: f64) -> Self {
        self.min_score = Some(min);
        self.max_score = Some(max);
        self
    }

    /// Filter by date range
    pub fn date_range<S: Into<String>>(mut self, from: S, to: S) -> Self {
        self.date_from = Some(from.into());
        self.date_to = Some(to.into());
        self
    }

    /// Filter by status
    pub fn status<S: Into<String>>(mut self, status: S) -> Self {
        self.final_status = Some(status.into());
        self
    }

    /// Set pagination
    pub fn paginate(mut self, limit: u32, offset: u32) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }

    /// Set sorting
    pub fn sort_by<S: Into<String>>(mut self, field: S, direction: S) -> Self {
        self.sort_by = Some(field.into());
        self.sort_direction = Some(direction.into());
        self
    }
}

/// Performance analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Overall statistics
    pub overall_stats: AgentStats,
    /// Performance trends
    pub trends: PerformanceTrends,
    /// Outliers detection
    pub outliers: Vec<PerformanceOutlier>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Analysis timestamp
    pub analyzed_at: String,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Score trend (improving, declining, stable)
    pub score_trend: TrendDirection,
    /// Execution time trend
    pub execution_time_trend: TrendDirection,
    /// Success rate trend
    pub success_rate_trend: TrendDirection,
    /// Trend confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Time period analyzed
    pub period_days: u32,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

/// Performance outlier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOutlier {
    /// Performance record ID
    pub id: i64,
    /// Agent type
    pub agent_type: String,
    /// Benchmark ID
    pub benchmark_id: String,
    /// Outlier type (low_score, slow_execution, failure)
    pub outlier_type: OutlierType,
    /// Outlier score (how unusual)
    pub outlier_score: f64,
    /// Reason for being an outlier
    pub reason: String,
    /// Timestamp
    pub timestamp: String,
}

/// Outlier type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierType {
    LowScore,
    SlowExecution,
    Failure,
    HighResourceUsage,
}

/// Utility functions for performance operations
pub struct PerformanceUtils;

impl PerformanceUtils {
    /// Calculate success rate from successes and total
    pub fn calculate_success_rate(successes: u32, total: u32) -> f64 {
        if total == 0 {
            0.0
        } else {
            (successes as f64 / total as f64) * 100.0
        }
    }

    /// Calculate average score from a list of scores
    pub fn calculate_average_score(scores: &[f64]) -> f64 {
        if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        }
    }

    /// Detect outliers using standard deviation method
    pub fn detect_outliers(values: &[f64], threshold_multiplier: f64) -> Vec<(usize, f64)> {
        if values.len() < 3 {
            return Vec::new();
        }

        let mean = Self::calculate_average_score(values);
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let threshold = threshold_multiplier * std_dev;
        values
            .iter()
            .enumerate()
            .filter_map(|(i, &value)| {
                let deviation = (value - mean).abs();
                if deviation > threshold {
                    Some((i, deviation / std_dev))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Determine trend direction from time series data
    pub fn determine_trend(values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        // Simple linear regression to determine trend
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        // Determine trend based on slope magnitude
        if slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Declining
        }
    }

    /// Generate performance recommendations
    pub fn generate_recommendations(analysis: &PerformanceAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        match analysis.trends.score_trend {
            TrendDirection::Declining => {
                recommendations.push(
                    "Score performance is declining. Consider reviewing recent changes."
                        .to_string(),
                );
            }
            TrendDirection::Improving => {
                recommendations.push(
                    "Great! Score performance is improving. Continue current approach.".to_string(),
                );
            }
            TrendDirection::Stable => {
                recommendations.push(
                    "Score performance is stable. Consider optimization for improvements."
                        .to_string(),
                );
            }
        }

        if analysis.overall_stats.success_rate < 80.0 {
            recommendations
                .push("Success rate is below 80%. Focus on improving reliability.".to_string());
        }

        if analysis.overall_stats.average_execution_time_ms > 30000 {
            recommendations.push(
                "Average execution time exceeds 30 seconds. Consider performance optimizations."
                    .to_string(),
            );
        }

        if !analysis.outliers.is_empty() {
            recommendations.push(format!(
                "Found {} performance outliers. Review these cases for patterns.",
                analysis.outliers.len()
            ));
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_rate_calculation() {
        assert_eq!(PerformanceUtils::calculate_success_rate(8, 10), 80.0);
        assert_eq!(PerformanceUtils::calculate_success_rate(0, 10), 0.0);
        assert_eq!(PerformanceUtils::calculate_success_rate(10, 10), 100.0);
        assert_eq!(PerformanceUtils::calculate_success_rate(0, 0), 0.0);
    }

    #[test]
    fn test_average_score_calculation() {
        assert!(
            (PerformanceUtils::calculate_average_score(&[0.8, 0.9, 0.7]) - 0.8).abs()
                < f64::EPSILON
        );
        assert_eq!(PerformanceUtils::calculate_average_score(&[]), 0.0);
        assert_eq!(PerformanceUtils::calculate_average_score(&[1.0]), 1.0);
    }

    #[test]
    fn test_outlier_detection() {
        let normal_values = vec![0.8, 0.9, 0.85, 0.95, 0.75];
        let with_outlier = vec![0.8, 0.9, 0.85, 0.95, 0.1]; // 0.1 is an outlier

        let normal_outliers = PerformanceUtils::detect_outliers(&normal_values, 2.0);
        assert_eq!(normal_outliers.len(), 0);

        let outlier_results = PerformanceUtils::detect_outliers(&with_outlier, 1.5);
        assert!(!outlier_results.is_empty());
    }

    #[test]
    fn test_trend_determination() {
        let increasing = vec![0.7, 0.75, 0.8, 0.85, 0.9];
        let decreasing = vec![0.9, 0.85, 0.8, 0.75, 0.7];
        let stable = vec![0.8, 0.81, 0.79, 0.82, 0.8];

        assert!(matches!(
            PerformanceUtils::determine_trend(&increasing),
            TrendDirection::Improving
        ));
        assert!(matches!(
            PerformanceUtils::determine_trend(&decreasing),
            TrendDirection::Declining
        ));
        assert!(matches!(
            PerformanceUtils::determine_trend(&stable),
            TrendDirection::Stable
        ));
    }
}
