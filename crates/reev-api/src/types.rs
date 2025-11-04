use serde::{Deserialize, Serialize};

use crate::services::PooledBenchmarkExecutor;

/// API-specific wrapper for BenchmarkInfo with additional fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkInfo {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt: String,
}

/// Benchmark execution summary for recent executions list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkExecution {
    pub execution_id: String,
    pub agent_type: String,
    pub status: String,
    pub created_at: String,
    pub score: Option<f64>,
}

/// Benchmark details with recent executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkWithExecutions {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub prompt: String,
    pub recent_executions: Vec<BenchmarkExecution>,
    pub latest_execution_id: Option<String>,
}

/// API state containing database connection (no in-memory cache)
#[derive(Clone)]
pub struct ApiState {
    pub db: reev_lib::db::PooledDatabaseWriter,
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

/// Dynamic flow execution request
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields are used in handlers but not directly in this module
pub struct DynamicFlowRequest {
    pub prompt: String,
    pub wallet: String,
    pub agent: String,
    pub execution_id: Option<String>,
    pub config: Option<AgentConfig>,
    pub shared_surfpool: bool,
    pub atomic_mode: Option<reev_types::flow::AtomicMode>,
}

/// Recovery flow execution request
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields are used in handlers but not directly in this module
pub struct RecoveryFlowRequest {
    pub prompt: String,
    pub wallet: String,
    pub agent: Option<String>,
    pub recovery_config: Option<RecoveryConfig>,
}

/// Recovery configuration
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields are used in handlers but not directly in this module
pub struct RecoveryConfig {
    pub base_retry_delay_ms: Option<u64>,
    pub max_retry_delay_ms: Option<u64>,
    pub backoff_multiplier: Option<f64>,
    pub max_recovery_time_ms: Option<u64>,
    pub enable_alternative_flows: Option<bool>,
    pub enable_user_fulfillment: Option<bool>,
}

/// Error response type
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
}
