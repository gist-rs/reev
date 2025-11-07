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
pub use sessions::ToolCallData;

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

    /// Store individual step session (for dynamic mode)
    async fn store_step_session(
        &self,
        execution_id: &str,
        step_index: usize,
        session_content: &str,
    ) -> crate::error::Result<()>;

    /// Get all sessions for consolidation (supports ping-pong)
    async fn get_sessions_for_consolidation(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Vec<crate::shared::performance::SessionLog>>;

    /// Store consolidated session (ping-pong result)
    async fn store_consolidated_session(
        &self,
        consolidated_id: &str,
        execution_id: &str,
        content: &str,
        metadata: &crate::shared::performance::ConsolidationMetadata,
    ) -> crate::error::Result<()>;

    /// Get consolidated session (for Mermaid generation)
    async fn get_consolidated_session(
        &self,
        consolidated_id: &str,
    ) -> crate::error::Result<Option<String>>;

    /// Begin transaction for step storage
    async fn begin_transaction(&self, execution_id: &str) -> crate::error::Result<()>;

    /// Commit transaction
    async fn commit_transaction(&self, execution_id: &str) -> crate::error::Result<()>;

    /// Rollback transaction on failure
    async fn rollback_transaction(&self, execution_id: &str) -> crate::error::Result<()>;
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

    /// Store individual step session (for dynamic mode)
    async fn store_step_session(
        &self,
        execution_id: &str,
        step_index: usize,
        session_content: &str,
    ) -> crate::error::Result<()> {
        let session_id = format!("{execution_id}_step_{step_index}");
        self.store_complete_log(&session_id, session_content).await
    }

    /// Get all sessions for consolidation (supports ping-pong)
    async fn get_sessions_for_consolidation(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Vec<crate::shared::performance::SessionLog>> {
        let mut sessions = Vec::new();

        // Query all step sessions for the execution
        let query = "SELECT session_id, content, created_at FROM session_logs WHERE session_id LIKE ? ORDER BY session_id";
        let pattern = format!("{execution_id}_step_%");

        let mut rows = self.conn.query(query, [pattern]).await.map_err(|e| {
            crate::error::DatabaseError::query(
                format!("Failed to query sessions for consolidation: {execution_id}"),
                e,
            )
        })?;

        while let Some(row) = rows.next().await? {
            let session_id: String = row.get(0).map_err(|e| {
                crate::error::DatabaseError::generic_with_source("Failed to parse session_id", e)
            })?;
            let content: String = row.get(1).map_err(|e| {
                crate::error::DatabaseError::generic_with_source("Failed to parse content", e)
            })?;
            let created_at_i64: i64 = row.get(2).map_err(|e| {
                crate::error::DatabaseError::generic_with_source("Failed to parse created_at", e)
            })?;
            let created_at = created_at_i64.to_string();

            sessions.push(crate::shared::performance::SessionLog {
                session_id,
                execution_id: execution_id.to_string(),
                content,
                timestamp: created_at,
                status: "completed".to_string(),
            });
        }

        Ok(sessions)
    }

    /// Store consolidated session (ping-pong result)
    async fn store_consolidated_session(
        &self,
        consolidated_id: &str,
        execution_id: &str,
        content: &str,
        metadata: &crate::shared::performance::ConsolidationMetadata,
    ) -> crate::error::Result<()> {
        let original_session_ids = serde_json::json!([]);

        self.conn
            .execute(
                "INSERT INTO consolidated_sessions (
                execution_id,
                consolidated_session_id,
                consolidated_content,
                original_session_ids,
                avg_score,
                total_tools,
                success_rate,
                execution_duration_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    execution_id,
                    consolidated_id,
                    content,
                    original_session_ids.to_string(),
                    metadata.avg_score,
                    metadata.total_tools,
                    metadata.success_rate,
                    metadata.execution_duration_ms,
                ),
            )
            .await
            .map_err(|e| {
                crate::error::DatabaseError::query(
                    format!("Failed to store consolidated session: {consolidated_id}"),
                    e,
                )
            })?;

        Ok(())
    }

    /// Get consolidated session (for Mermaid generation)
    async fn get_consolidated_session(
        &self,
        consolidated_id: &str,
    ) -> crate::error::Result<Option<String>> {
        let mut rows = self.conn.query(
            "SELECT consolidated_content FROM consolidated_sessions WHERE consolidated_session_id = ?",
            [consolidated_id]
        ).await.map_err(|e| {
            crate::error::DatabaseError::query(
                format!("Failed to query consolidated session: {consolidated_id}"),
                e,
            )
        })?;

        if let Some(row) = rows.next().await? {
            let content: String = row.get(0).map_err(|e| {
                crate::error::DatabaseError::generic_with_source(
                    "Failed to parse consolidated content",
                    e,
                )
            })?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Begin transaction for step storage
    async fn begin_transaction(&self, _execution_id: &str) -> crate::error::Result<()> {
        self.conn
            .execute("BEGIN TRANSACTION", ())
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to begin transaction", e))?;
        Ok(())
    }

    /// Commit transaction
    async fn commit_transaction(&self, _execution_id: &str) -> crate::error::Result<()> {
        self.conn
            .execute("COMMIT", ())
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to commit transaction", e))?;
        Ok(())
    }

    /// Rollback transaction on failure
    async fn rollback_transaction(&self, _execution_id: &str) -> crate::error::Result<()> {
        self.conn
            .execute("ROLLBACK", ())
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to rollback transaction", e))?;
        Ok(())
    }
}
