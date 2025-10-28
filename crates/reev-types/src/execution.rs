use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Execution status for tracking runner jobs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// Execution is queued and waiting to start
    Queued,
    /// Execution is currently running
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed with an error
    Failed,
    /// Execution was stopped by user
    Stopped,
    /// Execution timed out
    Timeout,
}

/// Execution state stored in database for inter-process communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Unique execution identifier
    pub execution_id: String,
    /// Benchmark being executed
    pub benchmark_id: String,
    /// Agent type used for execution
    pub agent: String,
    /// Current execution status
    pub status: ExecutionStatus,
    /// When execution was created
    pub created_at: DateTime<Utc>,
    /// When execution was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional execution progress (0.0 to 1.0)
    pub progress: Option<f64>,
    /// Optional error message if failed
    pub error_message: Option<String>,
    /// Optional execution result data
    pub result_data: Option<serde_json::Value>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionState {
    /// Create a new execution state
    pub fn new(execution_id: String, benchmark_id: String, agent: String) -> Self {
        let now = Utc::now();
        Self {
            execution_id,
            benchmark_id,
            agent,
            status: ExecutionStatus::Queued,
            created_at: now,
            updated_at: now,
            progress: None,
            error_message: None,
            result_data: None,
            metadata: HashMap::new(),
        }
    }

    /// Update execution status
    pub fn update_status(&mut self, status: ExecutionStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: f64) {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self.updated_at = Utc::now();
    }

    /// Set error message and update status to failed
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
        self.status = ExecutionStatus::Failed;
        self.updated_at = Utc::now();
    }

    /// Mark as completed with result data
    pub fn complete(&mut self, result_data: serde_json::Value) {
        self.result_data = Some(result_data);
        self.status = ExecutionStatus::Completed;
        self.updated_at = Utc::now();
        self.progress = Some(1.0);
    }

    /// Add metadata key-value pair
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = Utc::now();
    }
}

/// Execution request payload for queueing jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique request identifier
    pub request_id: String,
    /// Execution identifier (will be generated if not provided)
    pub execution_id: Option<String>,
    /// Path to benchmark file
    pub benchmark_path: String,
    /// Agent type to use
    pub agent: String,
    /// Request priority (lower number = higher priority)
    pub priority: i32,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to use shared surfpool
    pub shared_surfpool: bool,
    /// Additional request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionRequest {
    /// Create a new execution request
    pub fn new(benchmark_path: String, agent: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            execution_id: None,
            benchmark_path,
            agent,
            priority: 0,
            timeout_seconds: 300, // 5 minutes default
            shared_surfpool: false,
            metadata: HashMap::new(),
        }
    }

    /// Generate execution ID if not set
    pub fn ensure_execution_id(&mut self) -> &str {
        if self.execution_id.is_none() {
            self.execution_id = Some(Uuid::new_v4().to_string());
        }
        self.execution_id.as_ref().unwrap()
    }
}

/// Execution response with detailed results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    /// Execution identifier
    pub execution_id: String,
    /// Final execution status
    pub status: ExecutionStatus,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Optional result data
    pub result: Option<serde_json::Value>,
    /// Optional error message
    pub error: Option<String>,
    /// Execution logs (if available)
    pub logs: Vec<String>,
    /// Tool calls made during execution (if available)
    pub tool_calls: Vec<ToolCallSummary>,
}

/// Summary of a tool call made during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallSummary {
    /// Tool name
    pub tool_name: String,
    /// Tool call timestamp
    pub timestamp: DateTime<Utc>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Whether the call was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Timeout configuration for RPC calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default timeout in seconds
    pub default_timeout_seconds: u64,
    /// Maximum allowed timeout in seconds
    pub max_timeout_seconds: u64,
    /// Timeout for status checks
    pub status_check_timeout_seconds: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 300, // 5 minutes
            max_timeout_seconds: 3600,    // 1 hour
            status_check_timeout_seconds: 10,
        }
    }
}
