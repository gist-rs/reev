use serde::{Deserialize, Serialize};

/// Benchmark information loaded from YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt: String,
}

/// API state containing database connection and execution state
#[derive(Clone)]
pub struct ApiState {
    pub db: reev_lib::db::PooledDatabaseWriter,
    pub executions:
        std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, ExecutionState>>>,
    pub agent_configs:
        std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, AgentConfig>>>,
    pub benchmark_executor: std::sync::Arc<crate::services::PooledBenchmarkExecutor>,
}

/// Execution state for tracking benchmark runs
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionState {
    pub id: String,
    pub benchmark_id: String,
    pub agent: String,
    pub status: ExecutionStatus,
    pub progress: u8,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub trace: String,
    pub logs: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: String,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
}

/// Benchmark execution request
#[derive(Debug, Deserialize)]
pub struct BenchmarkExecutionRequest {
    pub agent: String,
    pub config: Option<AgentConfig>,
}

/// Execution response
#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub execution_id: String,
    pub status: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

/// Error response type
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
}
