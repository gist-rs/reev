//! Agent performance tracking operations
//!
//! Provides storage and retrieval of agent performance metrics,
//! execution times, and scoring data across sessions.

use crate::{
    error::{DatabaseError, Result},
    shared::performance::AgentPerformance,
    types::{AgentPerformanceSummary, PerformanceResult},
};
use std::collections::HashMap;
use tracing::{debug, info};

use super::core::DatabaseWriter;

impl DatabaseWriter {
    /// Helper method to create AgentPerformance from database row
    fn create_agent_performance_from_row(
        &self,
        row: &turso::Row,
        id_offset: usize,
    ) -> Result<AgentPerformance> {
        Ok(AgentPerformance {
            id: row.get::<Option<i64>>(id_offset)?,
            session_id: row.get(id_offset + 1)?,
            benchmark_id: row.get(id_offset + 2)?,
            agent_type: row.get(id_offset + 3)?,
            score: row.get(id_offset + 4)?,
            final_status: row.get(id_offset + 5)?,
            execution_time_ms: row
                .get::<Option<String>>(id_offset + 6)?
                .and_then(|s| s.parse().ok()),
            timestamp: row.get(id_offset + 7)?,
            flow_log_id: None,
            prompt_md5: row.get::<Option<String>>(id_offset + 8)?,
            additional_metrics: HashMap::new(),
        })
    }

    /// Helper method to get results for an agent type
    async fn get_agent_results(
        &self,
        agent_type: &str,
        limit: Option<i32>,
    ) -> Result<Vec<PerformanceResult>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {l}")).unwrap_or_default();
        let results_query = format!(
            "SELECT id, session_id, benchmark_id, score, final_status, created_at
             FROM agent_performance
             WHERE agent_type = ?
             ORDER BY created_at DESC{limit_clause}"
        );

        let mut results_rows = self
            .conn
            .prepare(&results_query)
            .await
            .map_err(|e| DatabaseError::query("Failed to prepare results query", e))?
            .query([agent_type])
            .await
            .map_err(|e| DatabaseError::query("Failed to query results", e))?;

        let mut results = Vec::new();
        while let Some(result_row) = results_rows.next().await? {
            let id: i64 = result_row.get(0)?;
            let session_id: String = result_row.get(1)?;
            let benchmark_id: String = result_row.get(2)?;
            let score: f64 = result_row.get(3)?;
            let final_status: String = result_row.get(4)?;
            let timestamp: String = result_row.get(5)?;

            results.push(PerformanceResult {
                id: Some(id),
                session_id,
                benchmark_id,
                score,
                final_status,
                timestamp,
            });
        }
        Ok(results)
    }

    /// Insert agent performance data
    pub async fn insert_agent_performance(&self, performance: &AgentPerformance) -> Result<()> {
        debug!(
            "[DB] Storing performance for agent: {} on benchmark: {}",
            performance.agent_type, performance.benchmark_id
        );

        self.conn
            .execute(
                "INSERT INTO agent_performance
                 (session_id, benchmark_id, agent_type, score, final_status, execution_time_ms, created_at, prompt_md5)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                [
                    performance.session_id.clone(),
                    performance.benchmark_id.clone(),
                    performance.agent_type.clone(),
                    performance.score.to_string(),
                    performance.final_status.clone(),
                    performance.execution_time_ms.map(|t| t.to_string()).unwrap_or_default(),
                    performance.timestamp.clone(),
                    performance.prompt_md5.as_ref().unwrap_or(&String::new()).clone(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to insert agent performance", e))?;

        info!("[DB] Agent performance stored successfully");
        Ok(())
    }

    /// Get agent performance summaries
    pub async fn get_agent_performance(&self) -> Result<Vec<AgentPerformanceSummary>> {
        debug!("[DB] Getting agent performance summaries");

        let query = "
            SELECT
                agent_type,
                COUNT(*) as execution_count,
                AVG(score) as avg_score,
                MAX(created_at) as latest_timestamp
            FROM agent_performance
            GROUP BY agent_type
            ORDER BY agent_type
        ";

        let mut rows =
            self.conn.query(query, ()).await.map_err(|e| {
                DatabaseError::query("Failed to get agent performance summaries", e)
            })?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            let agent_type: String = row.get(0)?;
            let execution_count: i64 = row.get(1)?;
            let avg_score: f64 = row.get(2)?;
            let _latest_timestamp: String = row.get(0)?;

            // Get recent results for this agent type
            let results = self.get_agent_results(&agent_type, None).await?;

            summaries.push(AgentPerformanceSummary {
                agent_type: agent_type.clone(),
                total_benchmarks: execution_count,
                average_score: avg_score,
                success_rate: 0.0,        // TODO: Calculate success rate
                best_benchmarks: vec![],  // TODO: Calculate best benchmarks
                worst_benchmarks: vec![], // TODO: Calculate worst benchmarks
                results,
            });
        }

        info!(
            "[DB] Retrieved {} agent performance summaries",
            summaries.len()
        );
        Ok(summaries)
    }

    /// Get performance data for a specific agent
    pub async fn get_agent_performance_by_type(
        &self,
        agent_type: &str,
    ) -> Result<Vec<AgentPerformance>> {
        debug!("[DB] Getting performance data for agent: {}", agent_type);

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, benchmark_id, agent_type, score, final_status, execution_time_ms, created_at, prompt_md5
                 FROM agent_performance WHERE agent_type = ? ORDER BY created_at DESC",
                [agent_type],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get agent performance by type", e))?;

        let mut performances = Vec::new();
        while let Some(row) = rows.next().await? {
            performances.push(self.create_agent_performance_from_row(&row, 0)?);
        }

        info!(
            "[DB] Retrieved {} performance records for agent: {}",
            performances.len(),
            agent_type
        );
        Ok(performances)
    }

    /// Get performance data for a specific benchmark
    pub async fn get_performance_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> Result<Vec<AgentPerformance>> {
        info!(
            "[DB] Getting performance data for benchmark: {}",
            benchmark_id
        );

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, benchmark_id, agent_type, score, final_status, execution_time_ms, created_at, prompt_md5
                 FROM agent_performance WHERE benchmark_id = ? ORDER BY created_at DESC",
                [benchmark_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get performance by benchmark", e))?;

        let mut performances = Vec::new();
        while let Some(row) = rows.next().await? {
            performances.push(self.create_agent_performance_from_row(&row, 0)?);
        }

        info!(
            "[DB] Retrieved {} performance records for benchmark: {}",
            performances.len(),
            benchmark_id
        );
        Ok(performances)
    }

    /// Get performance data for a specific session
    pub async fn get_performance_by_session(
        &self,
        session_id: &str,
    ) -> Result<Option<AgentPerformance>> {
        info!("[DB] Getting performance data for session: {}", session_id);

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, benchmark_id, agent_type, score, final_status, execution_time_ms, created_at, prompt_md5
                 FROM agent_performance WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get performance by session", e))?;

        if let Some(row) = rows.next().await? {
            let performance = self.create_agent_performance_from_row(&row, 0)?;
            info!("[DB] Found performance data for session: {}", session_id);
            Ok(Some(performance))
        } else {
            info!("[DB] No performance data found for session: {}", session_id);
            Ok(None)
        }
    }

    /// Get top performing agents by average score
    pub async fn get_top_agents(&self, limit: i32) -> Result<Vec<AgentPerformanceSummary>> {
        info!("[DB] Getting top {} performing agents", limit);

        let mut rows = self
            .conn
            .query(
                "SELECT
                     agent_type,
                     COUNT(*) as execution_count,
                     AVG(score) as avg_score,
                     MAX(created_at) as latest_timestamp
                 FROM agent_performance
                 GROUP BY agent_type
                 HAVING execution_count >= 3
                 ORDER BY avg_score DESC, execution_count DESC
                 LIMIT ?",
                [limit.to_string()],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get top agents", e))?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            let agent_type: String = row.get(0)?;
            let execution_count: i64 = row.get(1)?;
            let avg_score: f64 = row.get(2)?;
            let _latest_timestamp: String = row.get(0)?;

            // Get all results for this top agent
            let results = self.get_agent_results(&agent_type, None).await?;

            summaries.push(AgentPerformanceSummary {
                agent_type,
                total_benchmarks: execution_count,
                average_score: avg_score,
                success_rate: 0.0,        // TODO: Calculate success rate
                best_benchmarks: vec![],  // TODO: Calculate best benchmarks
                worst_benchmarks: vec![], // TODO: Calculate worst benchmarks
                results,
            });
        }

        info!("[DB] Retrieved top {} performing agents", summaries.len());
        Ok(summaries)
    }

    /// Get performance trend over time for an agent
    pub async fn get_agent_performance_trend(
        &self,
        agent_type: &str,
        limit: i32,
    ) -> Result<Vec<AgentPerformance>> {
        info!(
            "[DB] Getting performance trend for agent: {} (limit: {})",
            agent_type, limit
        );

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, benchmark_id, agent_type, score, final_status, execution_time_ms, created_at, prompt_md5
                 FROM agent_performance WHERE agent_type = ? ORDER BY created_at DESC LIMIT ?",
                [agent_type, &limit.to_string()],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get agent performance trend", e))?;

        let mut performances = Vec::new();
        while let Some(row) = rows.next().await? {
            performances.push(self.create_agent_performance_from_row(&row, 0)?);
        }

        info!(
            "[DB] Retrieved {} performance records for trend analysis",
            performances.len()
        );
        Ok(performances)
    }

    /// Delete performance records for a session
    pub async fn delete_performance_by_session(&self, session_id: &str) -> Result<()> {
        info!(
            "[DB] Deleting performance records for session: {}",
            session_id
        );

        let rows_affected = self
            .conn
            .execute(
                "DELETE FROM agent_performance WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| {
                DatabaseError::operation_with_source("Failed to delete performance records", e)
            })?;

        info!(
            "[DB] Deleted {} performance records for session: {}",
            rows_affected, session_id
        );
        Ok(())
    }

    /// Get performance statistics for an agent
    pub async fn get_agent_performance_stats(
        &self,
        agent_type: &str,
    ) -> Result<HashMap<String, f64>> {
        info!(
            "[DB] Getting performance statistics for agent: {}",
            agent_type
        );

        let mut rows = self
            .conn
            .query(
                "SELECT
                     COUNT(*) as total_executions,
                     AVG(score) as avg_score,
                     MIN(score) as min_score,
                     MAX(score) as max_score,
                     AVG(execution_time_ms) as avg_execution_time,
                     COUNT(CASE WHEN final_status = 'completed' THEN 1 END) as successful_executions
                 FROM agent_performance WHERE agent_type = ?",
                [agent_type],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get agent performance stats", e))?;

        let mut stats = HashMap::new();
        if let Some(row) = rows.next().await? {
            stats.insert("total_executions".to_string(), row.get::<i64>(0)? as f64);
            stats.insert("avg_score".to_string(), row.get::<f64>(1)?);
            stats.insert("min_score".to_string(), row.get::<f64>(2)?);
            stats.insert("max_score".to_string(), row.get::<f64>(3)?);
            stats.insert(
                "avg_execution_time".to_string(),
                row.get::<Option<f64>>(4)?.unwrap_or(0.0),
            );
            let successful: i64 = row.get(5)?;
            let total: i64 = row.get(0)?;
            let success_rate = if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            };
            stats.insert("success_rate".to_string(), success_rate);
        }

        info!(
            "[DB] Retrieved performance statistics for agent: {}",
            agent_type
        );
        Ok(stats)
    }
}
