//! Shared database module for reev
//!
//! This module re-exports database functionality from reev-db to provide
//! a unified interface for database operations used by both web and TUI interfaces.

use serde::{Deserialize, Serialize};

// Re-export all database functionality from reev-db
pub use reev_db::{
    // Types
    AgentPerformance,
    BatchError,
    BatchResult,
    BenchmarkData,
    BenchmarkYml,
    DatabaseConfig,
    DatabaseError,
    DatabaseReader,
    DatabaseStats,
    DatabaseWriter,
    DuplicateRecord,
    FlowLog,
    QueryFilter,
    Result,
    SyncError,
    SyncResult,
    SyncedBenchmark,
    TestResult,
    YmlTestResult,
    // Convenience functions
    VERSION,
};

// Compatibility type for backward compatibility
// This matches the old AgentPerformanceData structure from reev-lib
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceData {
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub flow_log_id: Option<i64>,
    pub prompt_md5: Option<String>,
}

impl From<AgentPerformanceData> for reev_db::AgentPerformance {
    fn from(data: AgentPerformanceData) -> Self {
        Self {
            id: None,
            benchmark_id: data.benchmark_id,
            agent_type: data.agent_type,
            score: data.score,
            final_status: data.final_status,
            execution_time_ms: Some(data.execution_time_ms as i64),
            timestamp: data.timestamp,
            flow_log_id: data.flow_log_id,
            prompt_md5: data.prompt_md5,
            additional_metrics: std::collections::HashMap::new(),
        }
    }
}

// Additional reev-lib specific database extensions can be added here if needed
