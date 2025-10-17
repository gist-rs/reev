//! Shared database module for reev
//!
//! This module re-exports database functionality from reev-db to provide
//! a unified interface for database operations used by both web and TUI interfaces.

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

// Re-export all database functionality from reev-db
pub use reev_db::{
    // Types
    AgentPerformance as DbAgentPerformance, // Legacy types from reev-db/types
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

// Re-export session types from types module
pub use reev_db::types::{SessionInfo, SessionResult};

// Re-export shared types for clarity
pub use reev_db::shared::flow::DBFlowLog as SharedFlowLog;
pub use reev_db::shared::performance::AgentPerformance as SharedPerformanceMetrics;

// Compatibility type for backward compatibility
// This matches the old AgentPerformanceData structure from reev-lib
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceData {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub score: f64,
    pub final_status: String,
    pub execution_time_ms: u64,
    pub timestamp: String,
    pub flow_log_id: Option<i64>,
    pub prompt_md5: Option<String>,
}

// Removed conflicting DbAgentPerformance conversion - using shared AgentPerformance instead

impl From<AgentPerformanceData> for SharedPerformanceMetrics {
    fn from(data: AgentPerformanceData) -> Self {
        SharedPerformanceMetrics {
            id: None,
            session_id: data.session_id,
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

// Wrapper struct to implement reev-flow DatabaseWriter trait for reev-db DatabaseWriter
/// Database writer wrapper that implements the reev-flow DatabaseWriter trait
pub struct FlowDatabaseWriter {
    inner: DatabaseWriter,
}

impl FlowDatabaseWriter {
    /// Create a new flow database writer wrapper
    pub fn new(db_writer: DatabaseWriter) -> Self {
        Self { inner: db_writer }
    }

    /// Get a reference to the inner database writer
    pub fn inner(&self) -> &DatabaseWriter {
        &self.inner
    }

    /// Get a mutable reference to the inner database writer
    pub fn inner_mut(&mut self) -> &mut DatabaseWriter {
        &mut self.inner
    }
}

// Implement Deref to provide direct access to DatabaseWriter methods
impl Deref for FlowDatabaseWriter {
    type Target = DatabaseWriter;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Implement DerefMut for mutable access
impl DerefMut for FlowDatabaseWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[async_trait::async_trait]
impl reev_flow::logger::DatabaseWriter for FlowDatabaseWriter {
    async fn insert_flow_log(
        &self,
        flow_log: &reev_flow::database::DBFlowLog,
    ) -> reev_flow::error::FlowResult<i64> {
        // For now, store flow logs as session logs to maintain compatibility
        // TODO: Implement proper flow log storage in reev-db
        let session_id = flow_log.session_id().to_string();
        let log_content = serde_json::to_string(flow_log).map_err(|e| {
            reev_flow::error::FlowError::database(format!("Failed to serialize flow log: {e}"))
        })?;

        self.inner
            .store_complete_log(&session_id, &log_content)
            .await
            .map(|_| 1i64)
            .map_err(|e| {
                let error_msg = format!("Failed to store flow log as session log: {e}");
                reev_flow::error::FlowError::database(error_msg)
            })
    }

    async fn insert_agent_performance(
        &self,
        performance: &reev_flow::logger::AgentPerformanceData,
    ) -> reev_flow::error::FlowResult<i64> {
        // Convert from reev-flow AgentPerformanceData to reev-lib AgentPerformanceData
        // Generate session_id from benchmark and agent info since reev-flow doesn't track sessions
        let generated_session_id =
            format!("{}_{}", performance.benchmark_id, performance.agent_type);
        let lib_performance = AgentPerformanceData {
            session_id: generated_session_id,
            benchmark_id: performance.benchmark_id.clone(),
            agent_type: performance.agent_type.clone(),
            score: performance.score,
            final_status: performance.final_status.clone(),
            execution_time_ms: performance.execution_time_ms,
            timestamp: performance.timestamp.clone(),
            flow_log_id: performance.flow_log_id,
            prompt_md5: performance.prompt_md5.clone(),
        };

        // Convert to DbAgentPerformance
        let db_performance = DbAgentPerformance::from(lib_performance);

        self.inner
            .insert_agent_performance(&db_performance)
            .await
            .map(|_| 1i64) // Return dummy ID since insert_agent_performance returns ()
            .map_err(|e| reev_flow::error::FlowError::database(e.to_string()))
    }

    async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_name: &str,
    ) -> reev_flow::error::FlowResult<Option<String>> {
        self.inner
            .get_prompt_md5_by_benchmark_name(benchmark_name)
            .await
            .map_err(|e| {
                let error_msg =
                    format!("Failed to get prompt MD5 for benchmark '{benchmark_name}': {e}");
                reev_flow::error::FlowError::database(error_msg)
            })
    }
}
