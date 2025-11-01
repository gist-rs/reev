//! Database writer module
//!
//! Provides modular database write operations organized by functionality:
//! - Core database operations
//! - Session management
//! - Benchmark synchronization
//! - Performance tracking
//! - Database monitoring

pub mod benchmarks;
pub mod core;
pub mod execution_states;
pub mod monitoring;
pub mod performance;
pub mod sessions;

// Re-export main DatabaseWriter for backward compatibility
pub use core::DatabaseWriter;

/// Database writer trait for common operations
#[allow(async_fn_in_trait)]
pub trait DatabaseWriterTrait: Send + Sync {
    /// Store execution state in database
    async fn store_execution_state(
        &self,
        state: &reev_types::ExecutionState,
    ) -> crate::error::Result<()>;

    /// Get execution state by ID
    async fn get_execution_state(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Option<reev_types::ExecutionState>>;

    /// List execution states by benchmark ID
    async fn list_execution_states_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> crate::error::Result<Vec<reev_types::ExecutionState>>;

    /// Insert agent performance data
    async fn insert_agent_performance(
        &self,
        performance: &crate::shared::performance::AgentPerformance,
    ) -> crate::error::Result<()>;

    /// Store session log content
    async fn store_session_log(
        &self,
        session_id: &str,
        log_content: &str,
    ) -> crate::error::Result<()>;

    /// Store tool call data
    async fn store_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        tool_data: &serde_json::Value,
    ) -> crate::error::Result<()>;
}

/// Implementation of DatabaseWriterTrait for DatabaseWriter
impl DatabaseWriterTrait for crate::writer::DatabaseWriter {
    /// Store execution state in database
    async fn store_execution_state(
        &self,
        state: &reev_types::ExecutionState,
    ) -> crate::error::Result<()> {
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(&self.conn);
        writer.store_execution_state(state).await
    }

    /// Get execution state by ID
    async fn get_execution_state(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Option<reev_types::ExecutionState>> {
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(&self.conn);
        writer.get_execution_state(execution_id).await
    }

    /// List execution states by benchmark ID
    async fn list_execution_states_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> crate::error::Result<Vec<reev_types::ExecutionState>> {
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(&self.conn);
        writer
            .list_execution_states_by_benchmark(benchmark_id)
            .await
    }

    /// Insert agent performance data
    async fn insert_agent_performance(
        &self,
        performance: &crate::shared::performance::AgentPerformance,
    ) -> crate::error::Result<()> {
        self.insert_agent_performance(performance).await
    }

    /// Store session log content
    async fn store_session_log(
        &self,
        session_id: &str,
        log_content: &str,
    ) -> crate::error::Result<()> {
        self.store_complete_log(session_id, log_content).await
    }

    /// Store tool call data
    async fn store_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        tool_data: &serde_json::Value,
    ) -> crate::error::Result<()> {
        use crate::writer::sessions::ToolCallData;

        let start_time = tool_data
            .get("start_time")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let execution_time_ms = tool_data
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let input_params = tool_data.get("input").cloned().unwrap_or_default();
        let output_result = tool_data.get("output").cloned().unwrap_or_default();

        let success = tool_data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let error_message = tool_data
            .get("error_message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tool_call_data = ToolCallData {
            session_id: session_id.to_string(),
            tool_name: tool_name.to_string(),
            start_time,
            execution_time_ms,
            input_params,
            output_result,
            status: if success {
                "success".to_string()
            } else {
                "failed".to_string()
            },
            error_message,
        };

        self.store_tool_call_consolidated(&tool_call_data).await
    }
}
