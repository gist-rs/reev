//! Pooled database writer for concurrent operations
//!
//! This module provides a DatabaseWriter wrapper that uses a connection pool
//! to handle concurrent database operations safely.

use crate::{
    config::DatabaseConfig,
    error::Result,
    pool::{ConnectionPool, PooledConnection},
    types::{BenchmarkData, QueryFilter},
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
    ) -> Result<()> {
        let conn = self.get_connection().await?;
        let writer =
            crate::DatabaseWriter::from_connection(conn.connection().clone(), self.config.clone());

        // Create a session result to complete the session
        let result = crate::types::SessionResult {
            end_time: chrono::Utc::now().timestamp(),
            score: 0.0, // Would need to be provided
            final_status: final_status.unwrap_or(status).to_string(),
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

        // Query the session_logs table directly
        let mut rows = writer
            .connection()
            .query(
                "SELECT log_content FROM session_logs WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| crate::error::DatabaseError::query("Failed to get session log", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to search benchmarks", _e))?
        {
            let log_content: String = row
                .get(0)
                .map_err(|_e| crate::error::DatabaseError::generic("Failed to get log content"))?;
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
        let conn = self.get_connection().await?;
        let writer = crate::DatabaseReader::from_connection(conn.connection().clone());

        // Get all agent types first
        let mut agent_rows = writer
            .connection()
            .query(
                "SELECT DISTINCT agent_type FROM agent_performance ORDER BY agent_type",
                (),
            )
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to get agent types", _e))?;

        let mut summaries = Vec::new();

        while let Some(agent_row) = agent_rows
            .next()
            .await
            .map_err(|_e| crate::error::DatabaseError::query("Failed to iterate agent types", _e))?
        {
            let agent_type: String = agent_row
                .get(0)
                .map_err(|_e| crate::error::DatabaseError::generic("Failed to get agent_type"))?;

            // Get performance data for this agent
            let mut perf_rows = writer.connection()
                .query(
                    "SELECT benchmark_id, score, final_status, timestamp FROM agent_performance WHERE agent_type = ? ORDER BY timestamp DESC",
                    (agent_type.as_str(),)
                )
                .await
                .map_err(|_e| crate::error::DatabaseError::query("Failed to get performance data", _e))?;

            let mut results = Vec::new();
            let mut total_score = 0.0;
            let mut success_count = 0;
            let mut benchmark_scores = std::collections::HashMap::new();

            while let Some(perf_row) = perf_rows.next().await.map_err(|_e| {
                crate::error::DatabaseError::query("Failed to iterate performance data", _e)
            })? {
                let benchmark_id: String = perf_row.get(0).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get benchmark_id")
                })?;
                let score: f64 = perf_row
                    .get(1)
                    .map_err(|_e| crate::error::DatabaseError::generic("Failed to get score"))?;
                let final_status: String = perf_row.get(2).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get final_status")
                })?;
                let timestamp: String = perf_row.get(3).map_err(|_e| {
                    crate::error::DatabaseError::generic("Failed to get timestamp")
                })?;

                total_score += score;
                if final_status.to_lowercase() == "completed"
                    || final_status.to_lowercase() == "succeeded"
                {
                    success_count += 1;
                }

                benchmark_scores.insert(benchmark_id.clone(), score);

                // Only include recent results (last 50 per agent)
                if results.len() < 50 {
                    results.push(crate::types::PerformanceResult {
                        id: None,
                        session_id: String::new(), // We don't have this in the current query
                        benchmark_id,
                        score,
                        final_status,
                        timestamp,
                    });
                }
            }

            let total_benchmarks = results.len() as i64;
            let average_score = if total_benchmarks > 0 {
                total_score / total_benchmarks as f64
            } else {
                0.0
            };
            let success_rate = if total_benchmarks > 0 {
                success_count as f64 / total_benchmarks as f64
            } else {
                0.0
            };

            // Sort benchmarks by score for best/worst lists
            let mut sorted_benchmarks: Vec<_> = benchmark_scores.into_iter().collect();
            sorted_benchmarks
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let best_benchmarks: Vec<String> = sorted_benchmarks
                .iter()
                .take(5)
                .map(|(id, _)| id.clone())
                .collect();

            let worst_benchmarks: Vec<String> = sorted_benchmarks
                .iter()
                .rev()
                .take(5)
                .map(|(id, _)| id.clone())
                .collect();

            summaries.push(crate::types::AgentPerformanceSummary {
                agent_type,
                total_benchmarks,
                average_score,
                success_rate,
                best_benchmarks,
                worst_benchmarks,
                results,
            });
        }

        Ok(summaries)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pooled_writer_basic() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path.to_str().unwrap());

        let writer = PooledDatabaseWriter::new(config, 3).await.unwrap();

        // Test basic operations
        let stats = writer.pool_stats().await;
        assert!(stats.current_size >= 1);
        assert!(stats.max_connections == 3);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(db_path.to_str().unwrap());

        let writer = Arc::new(PooledDatabaseWriter::new(config, 5).await.unwrap());

        let mut handles = vec![];

        // Spawn concurrent operations
        for i in 0..10 {
            let writer_clone = Arc::clone(&writer);
            let handle = tokio::spawn(async move {
                // Test database stats operation
                let stats = writer_clone.get_database_stats().await;
                assert!(stats.is_ok());
                i
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Check final pool stats
        let stats = writer.pool_stats().await;
        assert!(stats.active_connections <= stats.max_connections);
    }
}
