//! Database types for shared database operations

use serde::{Deserialize, Serialize};

/// Agent performance data for database insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceData {
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub flow_log_id: Option<i64>,
}

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

impl DatabaseConfig {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// Database result for benchmark evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub id: Option<i64>,
    pub benchmark_id: String,
    pub timestamp: String,
    pub prompt: String,
    pub generated_instruction: String,
    pub final_on_chain_state: String,
    pub final_status: String,
    pub score: f64,
}

/// Agent performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceSummary {
    pub agent_type: String,
    pub total_benchmarks: i64,
    pub average_score: f64,
    pub success_rate: f64,
    pub best_benchmarks: Vec<String>,
    pub worst_benchmarks: Vec<String>,
    pub results: Vec<BenchmarkResult>,
}
