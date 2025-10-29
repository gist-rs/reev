use reev_types::ExecutionState;
use serde::{Deserialize, Serialize};

use crate::services::PooledBenchmarkExecutor;

// API-specific wrapper for BenchmarkInfo with additional fields
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
    pub benchmark_executor: std::sync::Arc<PooledBenchmarkExecutor>,
}

// ExecutionStatus now imported from reev-types
// ExecutionStatus defined locally for API compatibility

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
