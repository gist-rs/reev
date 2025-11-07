//! Pooled database writer for concurrent operations
//!
//! This module provides a DatabaseWriter wrapper that uses a connection pool
//! to handle concurrent database operations safely.

use crate::{
    config::DatabaseConfig,
    error::Result,
    pool::{ConnectionPool, PooledConnection},
    types::{BenchmarkData, QueryFilter},
    writer::DatabaseWriterTrait,
};
use std::sync::Arc;
use tracing::{debug, info};

/// Pooled database writer that uses connection pool for concurrent operations
pub struct PooledDatabaseWriter {
    pool: Arc<ConnectionPool>,
    config: DatabaseConfig,
}

impl PooledDatabaseWriter {
    /// Create a new pooled database writer
    pub async fn new(config: DatabaseConfig, max_connections: usize) -> Result<Self> {
        info!(
            "[POOLED_WRITER] Creating pooled database writer with max {} connections",
            max_connections
        );

        let pool = Arc::new(ConnectionPool::new(config.clone(), max_connections).await?);

        Ok(Self { pool, config })
    }

    /// Get a connection from the pool for operations
    async fn get_connection(&self) -> Result<PooledConnection> {
        debug!("[POOLED_WRITER] Getting connection from pool");
        self.pool.get_connection().await
    }

    /// Get pool statistics
    pub async fn pool_stats(&self) -> crate::pool::PoolStats {
        self.pool.stats().await
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    // Benchmark operations
    pub async fn sync_benchmarks_from_dir(&self, dir_path: &str) -> Result<crate::SyncResult> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.sync_benchmarks_from_dir(dir_path).await
    }

    pub async fn upsert_benchmark(&self, benchmark_data: &BenchmarkData) -> Result<String> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer
            .upsert_benchmark(
                &benchmark_data.benchmark_name,
                &benchmark_data.prompt,
                &benchmark_data.content,
            )
            .await
    }

    pub async fn get_benchmark_by_id(&self, benchmark_id: &str) -> Result<Option<BenchmarkData>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Use search_benchmarks to find by ID
        let results = writer.search_benchmarks(benchmark_id).await?;
        Ok(results
            .into_iter()
            .find(|b| b.benchmark_name == benchmark_id))
    }

    pub async fn list_benchmarks(&self, filter: &QueryFilter) -> Result<Vec<BenchmarkData>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Use search_benchmarks with empty query to get all, then filter
        let all_benchmarks = writer.search_benchmarks("").await?;

        let mut filtered = Vec::new();
        for benchmark in all_benchmarks {
            let mut include = true;

            if let Some(_agent_type) = &filter.agent_type {
                // This would need to be implemented based on actual filtering logic
                // For now, include all
            }

            if let Some(benchmark_name) = &filter.benchmark_name {
                if !benchmark.benchmark_name.contains(benchmark_name) {
                    include = false;
                }
            }

            if include {
                filtered.push(benchmark);
            }
        }

        // Apply limit if specified
        if let Some(limit) = filter.limit {
            filtered.truncate(limit as usize);
        }

        Ok(filtered)
    }

    /// Get all benchmarks from database
    pub async fn get_all_benchmarks(&self) -> Result<Vec<BenchmarkData>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());
        writer.get_all_benchmarks().await
    }

    // Session operations
    pub async fn create_session(&self, session: &crate::types::SessionInfo) -> Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.create_session(session).await
    }

    pub async fn update_session_status(
        &self,
        session_id: &str,
        status: &str,
        final_status: Option<&str>,
        score: f64,
    ) -> Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());

        // Create a session result to complete the session
        let result = crate::types::SessionResult {
            end_time: chrono::Utc::now().timestamp(),
            score,
            final_status: final_status
                .or(Some(status))
                .filter(|s| !s.is_empty())
                .unwrap_or("unknown")
                .to_lowercase(),
        };

        writer.complete_session(session_id, &result).await
    }

    pub async fn store_complete_log(&self, session_id: &str, log_content: &str) -> Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.store_complete_log(session_id, log_content).await
    }

    pub async fn get_session_log(&self, session_id: &str) -> Result<Option<String>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Query the session_logs table directly - use 'content' column name to match original
        let mut rows = writer
            .connection()
            .query(
                "SELECT content FROM session_logs WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to retrieve session log", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to iterate session log", e))?
        {
            let log_content: String = row.get(0).map_err(|e| {
                crate::error::DatabaseError::generic_with_source("Failed to parse session log", e)
            })?;

            info!("Session log retrieved: {} chars", log_content.len());
            Ok(Some(log_content))
        } else {
            Ok(None)
        }
    }

    pub async fn list_sessions(
        &self,
        filter: &crate::types::SessionFilter,
    ) -> Result<Vec<crate::types::SessionInfo>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Build query based on filter
        let mut query = "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, final_status FROM execution_sessions".to_string();
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(benchmark_id) = &filter.benchmark_id {
            where_clauses.push("benchmark_id = ?");
            params.push(benchmark_id.as_str());
        }

        if let Some(agent_type) = &filter.agent_type {
            where_clauses.push("agent_type = ?");
            params.push(agent_type.as_str());
        }

        if let Some(status) = &filter.status {
            where_clauses.push("status = ?");
            params.push(status.as_str());
        }

        if !where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clauses.join(" AND "));
        }

        query.push_str(" ORDER BY start_time DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut stmt = writer.connection().prepare(&query).await.map_err(|e| {
            crate::error::DatabaseError::query("Failed to prepare sessions query", e)
        })?;

        let mut rows =
            match params.len() {
                0 => stmt.query(()).await.map_err(|e| {
                    crate::error::DatabaseError::query("Failed to query sessions", e)
                })?,
                1 => stmt.query([params[0]]).await.map_err(|e| {
                    crate::error::DatabaseError::query("Failed to query sessions", e)
                })?,
                2 => stmt.query([params[0], params[1]]).await.map_err(|e| {
                    crate::error::DatabaseError::query("Failed to query sessions", e)
                })?,
                3 => stmt
                    .query([params[0], params[1], params[2]])
                    .await
                    .map_err(|e| {
                        crate::error::DatabaseError::query("Failed to query sessions", e)
                    })?,
                _ => stmt.query(params).await.map_err(|e| {
                    crate::error::DatabaseError::query("Failed to query sessions", e)
                })?,
            };

        let mut sessions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to iterate sessions", _e))?
        {
            sessions.push(crate::types::SessionInfo {
                session_id: row.get(0).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get session_id")
                })?,
                benchmark_id: row.get(1).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get benchmark_id")
                })?,
                agent_type: row.get(2).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get agent_type")
                })?,
                interface: row.get(3).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get interface")
                })?,
                start_time: row.get(4).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get start_time")
                })?,
                end_time: row.get(5).ok(),
                status: row
                    .get(6)
                    .map_err(|_e| crate::error::DatabaseError::generic("Failed to get status"))?,
                score: None, // Not available in this query
                final_status: row.get(7).ok(),
            });
        }

        Ok(sessions)
    }

    /// Get a single session by session_id
    pub async fn get_session(&self, session_id: &str) -> Result<Option<crate::types::SessionInfo>> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());

        writer.get_session(session_id).await
    }

    /// List execution states by benchmark ID
    pub async fn list_execution_states_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> crate::error::Result<Vec<reev_types::ExecutionState>> {
        let conn = self.get_connection().await?;
        let writer = crate::writer::execution_states::ExecutionStatesWriter::new(conn.connection());
        writer
            .list_execution_states_by_benchmark(benchmark_id)
            .await
    }

    // Performance operations
    pub async fn insert_agent_performance(
        &self,
        performance: &crate::types::AgentPerformance,
    ) -> Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());

        // Convert to the expected AgentPerformance type for the writer
        let writer_performance = crate::shared::performance::AgentPerformance {
            id: performance.id,
            session_id: performance.session_id.clone(),
            benchmark_id: performance.benchmark_id.clone(),
            agent_type: performance.agent_type.clone(),
            score: performance.score,
            final_status: performance.final_status.clone(),
            execution_time_ms: performance.execution_time_ms,
            timestamp: performance.timestamp.clone(),
            flow_log_id: performance.flow_log_id,
            prompt_md5: performance.prompt_md5.clone(),
            additional_metrics: performance.additional_metrics.clone(),
        };

        writer.insert_agent_performance(&writer_performance).await
    }

    pub async fn get_agent_performance(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<crate::types::AgentPerformance>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());
        writer.get_agent_performance(Some(filter.clone())).await
    }

    pub async fn get_agent_performance_summary(
        &self,
    ) -> Result<Vec<crate::types::AgentPerformanceSummary>> {
        info!("[DEBUG] Starting safe get_agent_performance_summary");

        // Use the original working method to avoid Turso panics
        match self.get_agent_performance(&QueryFilter::new()).await {
            Ok(performances) => {
                info!("[DEBUG] Got {} raw performance records", performances.len());

                // Group by agent type in Rust instead of complex SQL
                let mut agent_data: std::collections::HashMap<
                    String,
                    Vec<crate::types::AgentPerformance>,
                > = std::collections::HashMap::new();

                for perf in performances {
                    agent_data
                        .entry(perf.agent_type.clone())
                        .or_default()
                        .push(perf);
                }

                let mut summaries = Vec::new();

                for (agent_type, records) in agent_data {
                    let total_benchmarks = records.len() as i64;
                    let total_score: f64 = records.iter().map(|r| r.score).sum();
                    let average_score = if total_benchmarks > 0 {
                        total_score / total_benchmarks as f64
                    } else {
                        0.0
                    };

                    let success_count = records
                        .iter()
                        .filter(|r| {
                            r.final_status.to_lowercase() == "completed"
                                || r.final_status.to_lowercase() == "succeeded"
                        })
                        .count();
                    let success_rate = if total_benchmarks > 0 {
                        success_count as f64 / total_benchmarks as f64
                    } else {
                        0.0
                    };

                    // Convert to PerformanceResult format (include all results)
                    let results: Vec<crate::types::PerformanceResult> = records
                        .into_iter()
                        .map(|perf| crate::types::PerformanceResult {
                            id: perf.id,
                            session_id: perf.session_id,
                            benchmark_id: perf.benchmark_id,
                            score: perf.score,
                            final_status: perf.final_status,
                            timestamp: perf.timestamp,
                        })
                        .collect();

                    summaries.push(crate::types::AgentPerformanceSummary {
                        agent_type,
                        total_benchmarks,
                        average_score,
                        success_rate,
                        best_benchmarks: vec![],  // TODO: Calculate properly
                        worst_benchmarks: vec![], // TODO: Calculate properly
                        results,
                    });
                }

                info!("[DEBUG] Created {} summaries safely", summaries.len());
                Ok(summaries)
            }
            Err(e) => {
                info!(
                    "[DEBUG] Failed to get performance data: {}, returning empty",
                    e
                );
                Ok(vec![]) // Return empty instead of panicking
            }
        }
    }

    // Monitoring operations
    pub async fn get_database_stats(&self) -> Result<crate::DatabaseStats> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());
        let stats = writer.get_benchmark_stats().await?;

        Ok(crate::DatabaseStats {
            total_benchmarks: stats.total_benchmarks,
            duplicate_count: 0,
            duplicate_details: vec![],
            total_results: stats.total_results,
            total_flow_logs: 0,
            total_performance_records: stats.total_results,
            database_size_bytes: None,
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn check_for_duplicates(&self) -> Result<Vec<crate::DuplicateRecord>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Query for duplicate benchmark names
        let mut rows = writer.connection()
            .query("SELECT benchmark_name, COUNT(*) as count FROM benchmarks GROUP BY benchmark_name HAVING count > 1", ())
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to check duplicates", e))?;

        let mut duplicates = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to iterate duplicates", _e))?
        {
            let benchmark_name: String = row.get(0).map_err(|_e| {
                crate::error::DatabaseError::generic("Failed to get benchmark_name")
            })?;

            duplicates.push(crate::DuplicateRecord {
                id: benchmark_name.clone(),
                benchmark_name,
                count: row
                    .get(1)
                    .map_err(|_e| crate::error::DatabaseError::generic("Failed to get count"))?,
                first_created_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(duplicates)
    }

    pub async fn get_prompt_md5_by_benchmark_name(
        &self,
        benchmark_name: &str,
    ) -> Result<Option<String>> {
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Query for prompt MD5
        let mut rows = writer
            .connection()
            .query(
                "SELECT prompt_md5 FROM benchmarks WHERE benchmark_name = ?",
                [benchmark_name],
            )
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to get prompt MD5", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to iterate prompt MD5", _e))?
        {
            let md5: Option<String> = row.get(0).ok();
            Ok(md5)
        } else {
            Ok(None)
        }
    }

    /// Gracefully shutdown the connection pool
    pub async fn shutdown(&self) -> Result<()> {
        info!("[POOLED_WRITER] Shutting down database connection pool...");

        // Close all connections in the pool
        self.pool.close().await?;

        info!("[POOLED_WRITER] Database connection pool shutdown complete");
        Ok(())
    }
}

impl DatabaseWriterTrait for PooledDatabaseWriter {
    /// Store execution state in database
    async fn store_execution_state(
        &self,
        state: &reev_types::ExecutionState,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(conn.connection());
        writer.store_execution_state(state).await
    }

    /// Get execution state by ID
    async fn get_execution_state(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Option<reev_types::ExecutionState>> {
        let conn = self.get_connection().await?;
        use crate::writer::execution_states::ExecutionStatesWriter;
        let writer = ExecutionStatesWriter::new(conn.connection());
        writer.get_execution_state(execution_id).await
    }

    /// List execution states by benchmark ID
    async fn list_execution_states_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> crate::error::Result<Vec<reev_types::ExecutionState>> {
        let conn = self.get_connection().await?;
        let writer = crate::writer::execution_states::ExecutionStatesWriter::new(conn.connection());
        writer
            .list_execution_states_by_benchmark(benchmark_id)
            .await
    }

    /// Insert agent performance data
    async fn insert_agent_performance(
        &self,
        performance: &crate::shared::performance::AgentPerformance,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer = crate::writer::core::DatabaseWriter {
            conn: conn.connection().clone(),
            config: self.config.clone(),
        };
        writer.insert_agent_performance(performance).await
    }

    /// Store session log content
    async fn store_session_log(
        &self,
        session_id: &str,
        log_content: &str,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer = crate::writer::core::DatabaseWriter {
            conn: conn.connection().clone(),
            config: self.config.clone(),
        };
        writer.store_session_log(session_id, log_content).await
    }

    /// Store tool call data
    async fn store_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        tool_data: &serde_json::Value,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer = crate::writer::core::DatabaseWriter {
            conn: conn.connection().clone(),
            config: self.config.clone(),
        };
        // Convert tool_data to ToolCallData format
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

        writer.store_tool_call_consolidated(&tool_call_data).await
    }

    /// Store individual step session (for dynamic mode)
    async fn store_step_session(
        &self,
        execution_id: &str,
        step_index: usize,
        session_content: &str,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer
            .store_step_session(execution_id, step_index, session_content)
            .await
    }

    /// Get all sessions for consolidation (supports ping-pong)
    async fn get_sessions_for_consolidation(
        &self,
        execution_id: &str,
    ) -> crate::error::Result<Vec<crate::shared::performance::SessionLog>> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.get_sessions_for_consolidation(execution_id).await
    }

    /// Store consolidated session (ping-pong result)
    async fn store_consolidated_session(
        &self,
        consolidated_id: &str,
        execution_id: &str,
        content: &str,
        metadata: &crate::shared::performance::ConsolidationMetadata,
    ) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer
            .store_consolidated_session(consolidated_id, execution_id, content, metadata)
            .await
    }

    /// Get consolidated session (for Mermaid generation)
    async fn get_consolidated_session(
        &self,
        consolidated_id: &str,
    ) -> crate::error::Result<Option<String>> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.get_consolidated_session(consolidated_id).await
    }

    /// Begin transaction for step storage
    async fn begin_transaction(&self, execution_id: &str) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.begin_transaction(execution_id).await
    }

    /// Commit transaction
    async fn commit_transaction(&self, execution_id: &str) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.commit_transaction(execution_id).await
    }

    /// Rollback transaction on failure
    async fn rollback_transaction(&self, execution_id: &str) -> crate::error::Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());
        writer.rollback_transaction(execution_id).await
    }
}

// Implement Clone for PooledDatabaseWriter
impl Clone for PooledDatabaseWriter {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            config: self.config.clone(),
        }
    }
}
