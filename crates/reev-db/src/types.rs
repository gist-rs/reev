//! Database types module for reev-db
//!
//! Defines common data structures used throughout the database operations
//! including benchmark data, results, flow logs, and performance metrics.
//!
//! Note: FlowLog and related types are now in the `shared` module to enable
//! reuse across projects and eliminate duplication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export shared types for backward compatibility
pub use crate::shared::flow::{DBFlowLog, ExecutionResult};

/// Session information for unified execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub interface: String, // 'tui' or 'web'
    pub start_time: i64,   // Unix timestamp
    pub end_time: Option<i64>,
    pub status: String, // 'running', 'completed', 'failed'
    pub score: Option<f64>,
    pub final_status: Option<String>,
}

/// Result of a completed session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    pub end_time: i64,
    pub score: f64,
    pub final_status: String,
}

/// Filter for querying sessions
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub benchmark_id: Option<String>,
    pub agent_type: Option<String>,
    pub interface: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
}

/// Log event for structured session logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: i64,     // Unix timestamp
    pub event_type: String, // 'prompt', 'tool_call', 'transaction', 'result'
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Complete session log structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub interface: String,
    pub start_time: i64,
    pub events: Vec<LogEvent>,
}

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

// FlowLog is now imported from shared module
// This comment marks the location where FlowLog was previously defined
// The shared FlowLog type provides the same functionality with better structure

/// Agent performance summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceSummary {
    /// Agent type
    pub agent_type: String,
    /// Number of executions
    pub execution_count: i64,
    /// Average score
    pub avg_score: f64,
    /// Latest timestamp
    pub latest_timestamp: String,
    /// Recent results (simplified for API)
    pub results: Vec<PerformanceResult>,
}

/// Simplified performance result for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    pub id: Option<i64>,
    pub benchmark_id: String,
    pub score: f64,
    pub final_status: String,
    pub timestamp: String,
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    /// Unique identifier
    pub id: Option<i64>,
    /// Session identifier for unified tracking
    pub session_id: String,
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

/// Database statistics for monitoring
/// Database statistics for monitoring and health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total number of benchmarks
    pub total_benchmarks: i64,
    /// Number of duplicate records detected
    pub duplicate_count: i64,
    /// Details about duplicates (id, name, count)
    pub duplicate_details: Vec<(String, String, i64)>,
    /// Total number of execution sessions
    pub total_results: i64,
    /// Total number of session logs
    pub total_flow_logs: i64,
    /// Total number of agent performance records
    pub total_performance_records: i64,
    /// Database file size in bytes (if available)
    pub database_size_bytes: Option<u64>,
    /// Last updated timestamp
    pub last_updated: String,
}

impl Default for DatabaseStats {
    fn default() -> Self {
        Self {
            total_benchmarks: 0,
            duplicate_count: 0,
            duplicate_details: Vec::new(),
            total_results: 0,
            total_flow_logs: 0,
            total_performance_records: 0,
            database_size_bytes: None,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Duplicate record information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateRecord {
    /// The duplicate ID
    pub id: String,
    /// Benchmark name
    pub benchmark_name: String,
    /// Number of occurrences
    pub count: i64,
    /// First occurrence timestamp
    pub first_created_at: String,
    /// Last occurrence timestamp
    pub last_updated_at: String,
}

/// Sync operation result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncResult {
    /// Number of benchmarks processed
    pub processed_count: usize,
    /// Number of new benchmarks created
    pub new_count: usize,
    /// Number of existing benchmarks updated
    pub updated_count: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Duration of sync operation in milliseconds
    pub duration_ms: u64,
    /// List of synced benchmarks with their details
    pub synced_benchmarks: Vec<SyncedBenchmark>,
    /// List of errors encountered during sync
    pub errors: Vec<SyncError>,
}

/// Information about a synced benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedBenchmark {
    /// Benchmark name
    pub name: String,
    /// Generated MD5
    pub md5: String,
    /// Operation performed ("created" or "updated")
    pub operation: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Error encountered during sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    /// File path where error occurred
    pub file_path: String,
    /// Error message
    pub error_message: String,
    /// Error type
    pub error_type: String,
    /// Timestamp when error occurred
    pub timestamp: String,
}

/// Query filter options
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// Filter by benchmark name (partial match)
    pub benchmark_name: Option<String>,
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
    /// Limit number of results
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Sort by field
    pub sort_by: Option<String>,
    /// Sort direction ("asc" or "desc")
    pub sort_direction: Option<String>,
}

impl QueryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by benchmark name
    pub fn benchmark_name<S: Into<String>>(mut self, name: S) -> Self {
        self.benchmark_name = Some(name.into());
        self
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

    /// Filter by minimum score
    pub fn min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Filter by maximum score
    pub fn max_score(mut self, score: f64) -> Self {
        self.max_score = Some(score);
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

/// Batch operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult<T> {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Successful results
    pub successes: Vec<T>,
    /// Failed operations with errors
    pub failures: Vec<BatchError>,
}

/// Error from a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Identifier of the item that failed
    pub identifier: String,
    /// Error message
    pub error_message: String,
    /// Error type
    pub error_type: String,
}

impl<T> BatchResult<T> {
    /// Create a new empty batch result
    pub fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            successes: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// Add a successful result
    pub fn add_success(&mut self, result: T) {
        self.successes.push(result);
        self.success_count += 1;
    }

    /// Add a failed result
    pub fn add_failure(&mut self, identifier: String, error: String) {
        self.failures.push(BatchError {
            identifier,
            error_message: error.clone(),
            error_type: "generic".to_string(),
        });
        self.failure_count += 1;
    }

    /// Check if all operations succeeded
    pub fn is_complete_success(&self) -> bool {
        self.failure_count == 0
    }

    /// Get total number of operations
    pub fn total_count(&self) -> usize {
        self.success_count + self.failure_count
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_count() == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.total_count() as f64) * 100.0
        }
    }
}

impl<T> Default for BatchResult<T> {
    fn default() -> Self {
        Self::new()
    }
}
