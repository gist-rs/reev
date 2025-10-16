//! Shared benchmark types for reev ecosystem
//!
//! This module contains benchmark-related types that can be used across
//! different projects. These types are designed to be:
//! - Database-friendly (String timestamps, JSON serializable)
//! - Generic enough for different use cases
//! - Compatible with YAML-based benchmark definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Benchmark data structure from YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkYml {
    /// Unique identifier for the benchmark
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// The prompt text sent to the agent
    pub prompt: String,
    /// Initial state configuration
    pub initial_state: Vec<serde_yaml::Value>,
    /// Expected ground truth results
    pub ground_truth: serde_yaml::Value,
}

/// Benchmark data structure stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkData {
    /// MD5 hash of benchmark_name:prompt (primary key)
    pub id: String,
    /// Benchmark name (e.g., "001-spl-transfer")
    pub benchmark_name: String,
    /// The prompt text
    pub prompt: String,
    /// Full YAML content
    pub content: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Unique identifier
    pub id: Option<String>,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Execution timestamp
    pub timestamp: String,
    /// The prompt used
    pub prompt: String,
    /// Generated instruction
    pub generated_instruction: String,
    /// Final on-chain state
    pub final_on_chain_state: String,
    /// Final execution status
    pub final_status: String,
    /// Performance score (0.0 - 1.0)
    pub score: f64,
    /// Prompt MD5 for linking to benchmarks
    pub prompt_md5: Option<String>,
    /// Execution duration in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// YAML-based test result for structured testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmlTestResult {
    /// Unique identifier
    pub id: Option<i64>,
    /// Benchmark identifier
    pub benchmark_id: String,
    /// Agent type that executed this test
    pub agent_type: String,
    /// Full YAML content of the test result
    pub yml_content: String,
    /// When this result was created
    pub created_at: String,
}

/// Query filter options for benchmarks
#[derive(Debug, Clone, Default)]
pub struct BenchmarkFilter {
    /// Filter by benchmark name (partial match)
    pub benchmark_name: Option<String>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by date range (start)
    pub date_from: Option<String>,
    /// Filter by date range (end)
    pub date_to: Option<String>,
    /// Limit number of results
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort direction ("asc" or "desc")
    pub sort_direction: Option<String>,
}

impl BenchmarkFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by benchmark name
    pub fn benchmark_name<S: Into<String>>(mut self, name: S) -> Self {
        self.benchmark_name = Some(name.into());
        self
    }

    /// Filter by tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Filter by date range
    pub fn date_range<S: Into<String>>(mut self, from: S, to: S) -> Self {
        self.date_from = Some(from.into());
        self.date_to = Some(to.into());
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

/// Benchmark statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStats {
    /// Total number of benchmarks
    pub total_benchmarks: i64,
    /// Number of benchmarks by category
    pub benchmarks_by_category: HashMap<String, i64>,
    /// Average benchmark difficulty
    pub average_difficulty: Option<f64>,
    /// Most recently updated
    pub last_updated: String,
}

impl Default for BenchmarkStats {
    fn default() -> Self {
        Self {
            total_benchmarks: 0,
            benchmarks_by_category: HashMap::new(),
            average_difficulty: None,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Utility functions for benchmark operations
pub struct BenchmarkUtils;

impl BenchmarkUtils {
    /// Generate MD5 hash for benchmark
    pub fn generate_md5(benchmark_name: &str, prompt: &str) -> String {
        let digest = md5::compute(format!("{benchmark_name}:{prompt}"));
        format!("{digest:x}")
    }

    /// Validate benchmark structure
    pub fn validate_benchmark(benchmark: &BenchmarkYml) -> Result<(), String> {
        if benchmark.id.is_empty() {
            return Err("Benchmark ID cannot be empty".to_string());
        }
        if benchmark.prompt.is_empty() {
            return Err("Benchmark prompt cannot be empty".to_string());
        }
        if benchmark.initial_state.is_empty() {
            return Err("Benchmark initial state cannot be empty".to_string());
        }
        Ok(())
    }

    /// Convert benchmark to YAML string
    pub fn to_yaml_string(benchmark: &BenchmarkYml) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(benchmark)
    }

    /// Parse benchmark from YAML string
    pub fn from_yaml_string(yaml: &str) -> Result<BenchmarkYml, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Extract tags from benchmark content
    pub fn extract_tags(benchmark: &BenchmarkYml) -> Vec<String> {
        let mut tags = benchmark.tags.clone();

        // Extract tags from description if needed
        if tags.is_empty() {
            let desc_tags: Vec<String> = benchmark
                .description
                .split_whitespace()
                .filter(|word| word.starts_with('#'))
                .map(|tag| tag.trim_start_matches('#').to_string())
                .collect();
            tags.extend(desc_tags);
        }

        tags
    }

    /// Calculate benchmark difficulty based on content
    pub fn calculate_difficulty(benchmark: &BenchmarkYml) -> f64 {
        let mut difficulty = 0.0;

        // Base difficulty from prompt length
        difficulty += (benchmark.prompt.len() as f64 / 1000.0).min(2.0);

        // Difficulty from initial state complexity
        difficulty += (benchmark.initial_state.len() as f64 / 10.0).min(1.5);

        // Difficulty from ground truth complexity
        if let Ok(ground_truth_str) = serde_yaml::to_string(&benchmark.ground_truth) {
            difficulty += (ground_truth_str.len() as f64 / 2000.0).min(1.5);
        }

        difficulty.min(5.0) // Cap at 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn test_benchmark_md5_generation() {
        let md5_1 = BenchmarkUtils::generate_md5("test", "prompt");
        let md5_2 = BenchmarkUtils::generate_md5("test", "prompt");
        let md5_3 = BenchmarkUtils::generate_md5("test", "different");

        assert_eq!(md5_1, md5_2);
        assert_ne!(md5_1, md5_3);
    }

    #[test]
    fn test_benchmark_validation() {
        let valid_benchmark = BenchmarkYml {
            id: "test-1".to_string(),
            description: "Test benchmark".to_string(),
            tags: vec!["test".to_string()],
            prompt: "Test prompt".to_string(),
            initial_state: vec![Value::Null],
            ground_truth: Value::Null,
        };

        assert!(BenchmarkUtils::validate_benchmark(&valid_benchmark).is_ok());

        let invalid_benchmark = BenchmarkYml {
            id: "".to_string(),
            description: "Test benchmark".to_string(),
            tags: vec![],
            prompt: "".to_string(),
            initial_state: vec![],
            ground_truth: Value::Null,
        };

        assert!(BenchmarkUtils::validate_benchmark(&invalid_benchmark).is_err());
    }

    #[test]
    fn test_benchmark_difficulty() {
        let benchmark = BenchmarkYml {
            id: "test-1".to_string(),
            description: "Test benchmark".to_string(),
            tags: vec!["test".to_string()],
            prompt: "A simple test prompt".to_string(),
            initial_state: vec![Value::Null],
            ground_truth: Value::Null,
        };

        let difficulty = BenchmarkUtils::calculate_difficulty(&benchmark);
        assert!(difficulty > 0.0);
        assert!(difficulty <= 5.0);
    }
}
