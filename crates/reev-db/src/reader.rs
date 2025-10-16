//! Database reader module for reev-db
//!
//! Provides read-only database operations with efficient querying,
//! filtering, and result aggregation capabilities.

use crate::{
    error::{DatabaseError, Result},
    types::{AgentPerformance, DBFlowLog, QueryFilter, TestResult, YmlTestResult},
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Database reader for efficient read operations
pub struct DatabaseReader {
    conn: turso::Connection,
}

impl DatabaseReader {
    /// Create a new database reader with an existing connection
    pub fn new(conn: turso::Connection) -> Self {
        Self { conn }
    }

    /// Create a new database reader from configuration
    pub async fn from_config(config: crate::DatabaseConfig) -> Result<Self> {
        let db = turso::Builder::new_local(&config.path)
            .build()
            .await
            .map_err(|e| {
                DatabaseError::connection_with_source(
                    format!("Failed to create local database: {}", config.path),
                    e,
                )
            })?;

        let conn = db.connect().map_err(|e| {
            DatabaseError::connection_with_source("Failed to establish database connection", e)
        })?;

        Ok(Self { conn })
    }

    /// Get test results with optional filtering
    pub async fn get_test_results(&self, filter: Option<QueryFilter>) -> Result<Vec<TestResult>> {
        let mut query = "
            SELECT id, benchmark_id, timestamp, prompt, generated_instruction,
                   final_on_chain_state, final_status, score, prompt_md5
            FROM results
        "
        .to_string();

        let mut params = Vec::new();
        let mut where_clauses = Vec::new();

        if let Some(f) = filter {
            // Build WHERE clause
            if let Some(benchmark_id) = f.benchmark_name {
                where_clauses.push("benchmark_id LIKE ?");
                params.push(format!("%{benchmark_id}%"));
            }

            if let Some(agent_type) = f.agent_type {
                where_clauses.push("final_status LIKE ?");
                params.push(format!("%{agent_type}%"));
            }

            if let Some(min_score) = f.min_score {
                where_clauses.push("score >= ?");
                params.push(min_score.to_string());
            }

            if let Some(max_score) = f.max_score {
                where_clauses.push("score <= ?");
                params.push(max_score.to_string());
            }

            if let Some(date_from) = f.date_from {
                where_clauses.push("timestamp >= ?");
                params.push(date_from);
            }

            if let Some(date_to) = f.date_to {
                where_clauses.push("timestamp <= ?");
                params.push(date_to);
            }

            if !where_clauses.is_empty() {
                query.push_str(" WHERE ");
                query.push_str(&where_clauses.join(" AND "));
            }

            // Add ORDER BY
            if let Some(sort_by) = f.sort_by {
                let direction = f.sort_direction.as_deref().unwrap_or("DESC");
                query.push_str(&format!(" ORDER BY {sort_by} {direction}"));
            } else {
                query.push_str(" ORDER BY timestamp DESC");
            }

            // Add LIMIT and OFFSET
            if let Some(limit) = f.limit {
                query.push_str(&format!(" LIMIT {limit}"));
                if let Some(offset) = f.offset {
                    query.push_str(&format!(" OFFSET {offset}"));
                }
            }
        } else {
            query.push_str(" ORDER BY timestamp DESC");
        }

        debug!("[DB] Querying test results: {}", query);
        debug!("[DB] Query params: {:?}", params);

        let mut stmt = self
            .conn
            .prepare(&query)
            .await
            .map_err(|e| DatabaseError::query("Failed to prepare test results query", e))?;

        // Handle dynamic parameters based on count
        let mut rows = match params.len() {
            0 => stmt
                .query(())
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            1 => stmt
                .query([params[0].as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            2 => stmt
                .query([params[0].as_str(), params[1].as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            3 => stmt
                .query([params[0].as_str(), params[1].as_str(), params[2].as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            4 => stmt
                .query([
                    params[0].as_str(),
                    params[1].as_str(),
                    params[2].as_str(),
                    params[3].as_str(),
                ])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            5 => stmt
                .query([
                    params[0].as_str(),
                    params[1].as_str(),
                    params[2].as_str(),
                    params[3].as_str(),
                    params[4].as_str(),
                ])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            6 => stmt
                .query([
                    params[0].as_str(),
                    params[1].as_str(),
                    params[2].as_str(),
                    params[3].as_str(),
                    params[4].as_str(),
                    params[5].as_str(),
                ])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?,
            _ => {
                warn!("[DB] Too many parameters ({}), limiting to 6", params.len());
                stmt.query([
                    params.first().map(|s| s.as_str()).unwrap_or(""),
                    params.get(1).map(|s| s.as_str()).unwrap_or(""),
                    params.get(2).map(|s| s.as_str()).unwrap_or(""),
                    params.get(3).map(|s| s.as_str()).unwrap_or(""),
                    params.get(4).map(|s| s.as_str()).unwrap_or(""),
                    params.get(5).map(|s| s.as_str()).unwrap_or(""),
                ])
                .await
                .map_err(|e| DatabaseError::query("Failed to query test results", e))?
            }
        };

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate test results", e))?
        {
            results.push(TestResult {
                id: Some(row.get(0).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get result ID", e)
                })?),
                benchmark_id: row.get(1).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get benchmark ID", e)
                })?,
                timestamp: row.get(2).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get timestamp", e)
                })?,
                prompt: row
                    .get(3)
                    .map_err(|e| DatabaseError::generic_with_source("Failed to get prompt", e))?,
                generated_instruction: row.get(4).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get generated instruction", e)
                })?,
                final_on_chain_state: row.get(5).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get final on-chain state", e)
                })?,
                final_status: row.get(6).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get final status", e)
                })?,
                score: row
                    .get(7)
                    .map_err(|e| DatabaseError::generic_with_source("Failed to get score", e))?,
                prompt_md5: row.get(8).ok(),
                execution_time_ms: None,
                metadata: HashMap::new(),
            });
        }

        info!("[DB] Retrieved {} test results", results.len());
        Ok(results)
    }

    /// Get flow logs for a specific benchmark
    pub async fn get_flow_logs(&self, benchmark_id: &str) -> Result<Vec<DBFlowLog>> {
        let query = "
            SELECT id, session_id, benchmark_id, agent_type, start_time, end_time,
                   final_result, flow_data, created_at
            FROM flow_logs
            WHERE benchmark_id = ?
            ORDER BY start_time DESC
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .map_err(|e| DatabaseError::query("Failed to prepare flow logs query", e))?;

        // TODO: Implement proper dynamic parameter handling
        // For now, skip dynamic filtering
        // For now, use empty params until we implement proper dynamic parameter handling
        let mut rows = stmt
            .query([benchmark_id])
            .await
            .map_err(|e| DatabaseError::query("Failed to query flow logs", e))?;

        let mut logs = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate flow logs", e))?
        {
            logs.push(DBFlowLog {
                id: Some(
                    row.get(0).map_err(|e| {
                        DatabaseError::generic_with_source("Failed to get log ID", e)
                    })?,
                ),
                session_id: row.get(1).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get session ID", e)
                })?,
                benchmark_id: row.get(2).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get benchmark ID", e)
                })?,
                agent_type: row.get(3).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get agent type", e)
                })?,
                start_time: row.get(4).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get start time", e)
                })?,
                end_time: row.get(5).ok(),
                final_result: row.get(6).ok(),
                flow_data: row.get(7).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get flow data", e)
                })?,
                created_at: row.get(8).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get created_at", e)
                })?,
            });
        }

        info!(
            "[DB] Retrieved {} flow logs for benchmark '{}'",
            logs.len(),
            benchmark_id
        );
        Ok(logs)
    }

    /// Get agent performance metrics
    pub async fn get_agent_performance(
        &self,
        filter: Option<QueryFilter>,
    ) -> Result<Vec<AgentPerformance>> {
        let mut query = "
            SELECT id, benchmark_id, agent_type, score, final_status,
                   execution_time_ms, timestamp, flow_log_id, prompt_md5
            FROM agent_performance
        "
        .to_string();

        let mut params = Vec::new();
        let mut where_clauses = Vec::new();

        if let Some(f) = filter {
            if let Some(agent_type) = f.agent_type {
                where_clauses.push("agent_type = ?");
                params.push(agent_type);
            }

            if let Some(benchmark_id) = f.benchmark_name {
                where_clauses.push("benchmark_id LIKE ?");
                params.push(format!("%{benchmark_id}%"));
            }

            if let Some(min_score) = f.min_score {
                where_clauses.push("score >= ?");
                params.push(min_score.to_string());
            }

            if let Some(max_score) = f.max_score {
                where_clauses.push("score <= ?");
                params.push(max_score.to_string());
            }

            if !where_clauses.is_empty() {
                query.push_str(" WHERE ");
                query.push_str(&where_clauses.join(" AND "));
            }

            query.push_str(" ORDER BY timestamp DESC");

            if let Some(limit) = f.limit {
                query.push_str(&format!(" LIMIT {limit}"));
            }
        } else {
            query.push_str(" ORDER BY timestamp DESC");
        }

        let mut stmt =
            self.conn.prepare(&query).await.map_err(|e| {
                DatabaseError::query("Failed to prepare agent performance query", e)
            })?;

        // Simplify by not using dynamic parameters for now
        let mut rows = stmt
            .query(())
            .await
            .map_err(|e| DatabaseError::query("Failed to query agent performance", e))?;

        let mut performances = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate performance results", e))?
        {
            performances.push(AgentPerformance {
                id: Some(row.get(0).map_err(|e| {
                    DatabaseError::generic_with_source("Failed to get performance ID", e)
                })?),
                benchmark_id: row
                    .get(1)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark ID"))?,
                agent_type: row
                    .get(2)
                    .map_err(|_| DatabaseError::generic("Failed to get agent type"))?,
                score: row
                    .get(3)
                    .map_err(|_| DatabaseError::generic("Failed to get score"))?,
                final_status: row
                    .get(4)
                    .map_err(|_| DatabaseError::generic("Failed to get final status"))?,
                execution_time_ms: row.get(5).ok(),
                timestamp: row
                    .get(6)
                    .map_err(|_| DatabaseError::generic("Failed to get timestamp"))?,
                flow_log_id: row.get(7).ok(),
                prompt_md5: row.get(8).ok(),
                additional_metrics: HashMap::new(),
            });
        }

        info!(
            "[DB] Retrieved {} agent performance records",
            performances.len()
        );
        Ok(performances)
    }

    /// Get YAML test results for a specific benchmark and agent
    pub async fn get_yml_test_results(
        &self,
        benchmark_id: &str,
        agent_type: &str,
    ) -> Result<Vec<YmlTestResult>> {
        let query = "
            SELECT id, benchmark_id, agent_type, yml_content, created_at
            FROM yml_testresults
            WHERE benchmark_id = ? AND agent_type = ?
            ORDER BY created_at DESC
        ";

        let mut stmt =
            self.conn.prepare(query).await.map_err(|e| {
                DatabaseError::query("Failed to prepare YAML test results query", e)
            })?;

        let mut rows = stmt
            .query([benchmark_id, agent_type])
            .await
            .map_err(|e| DatabaseError::query("Failed to query YAML test results", e))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate YML test results", e))?
        {
            results.push(YmlTestResult {
                id: Some(
                    row.get(0)
                        .map_err(|_| DatabaseError::generic("Failed to get result ID"))?,
                ),
                benchmark_id: row
                    .get(1)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark ID"))?,
                agent_type: row
                    .get(2)
                    .map_err(|_| DatabaseError::generic("Failed to get agent type"))?,
                yml_content: row
                    .get(3)
                    .map_err(|_| DatabaseError::generic("Failed to get YML content"))?,
                created_at: row
                    .get(4)
                    .map_err(|_| DatabaseError::generic("Failed to get created_at"))?,
            });
        }

        info!(
            "[DB] Retrieved {} YML test results for '{}' agent '{}' ",
            results.len(),
            benchmark_id,
            agent_type
        );
        Ok(results)
    }

    /// Get benchmark statistics
    pub async fn get_benchmark_stats(&self) -> Result<BenchmarkStats> {
        let total_benchmarks = self
            .conn
            .query("SELECT COUNT(*) FROM benchmarks", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to count benchmarks", e))?
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get benchmark count", e))?
            .map(|row| {
                row.get::<i64>(0)
                    .map_err(|_| DatabaseError::generic("Failed to parse benchmark count"))
            })
            .unwrap_or(Ok(0))?;

        let total_results = self
            .conn
            .query("SELECT COUNT(*) FROM results", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to count results", e))?
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get result count", e))?
            .map(|row| {
                row.get::<i64>(0)
                    .map_err(|_| DatabaseError::generic("Failed to parse result count"))
            })
            .unwrap_or(Ok(0))?;

        let avg_score = self
            .conn
            .query("SELECT AVG(score) FROM results WHERE score IS NOT NULL", ())
            .await
            .map_err(|e| DatabaseError::query("Failed to calculate average score", e))?
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to get average score", e))?
            .map(|row| {
                row.get::<Option<f64>>(0)
                    .map_err(|_| DatabaseError::generic("Failed to parse average score"))
            })
            .unwrap_or(Ok(None))?;

        Ok(BenchmarkStats {
            total_benchmarks,
            total_results,
            average_score: avg_score.unwrap_or(0.0),
        })
    }

    /// Search benchmarks by text
    pub async fn search_benchmarks(
        &self,
        query_text: &str,
    ) -> Result<Vec<crate::types::BenchmarkData>> {
        let query = "
            SELECT id, benchmark_name, prompt, content, created_at, updated_at
            FROM benchmarks
            WHERE benchmark_name LIKE ? OR prompt LIKE ? OR content LIKE ?
            ORDER BY benchmark_name
        ";

        let search_pattern = format!("%{query_text}%");
        let mut stmt =
            self.conn.prepare(query).await.map_err(|e| {
                DatabaseError::query("Failed to prepare search benchmarks query", e)
            })?;

        let mut rows = stmt
            .query([
                search_pattern.as_str(),
                search_pattern.as_str(),
                search_pattern.as_str(),
            ])
            .await
            .map_err(|e| DatabaseError::query("Failed to search benchmarks", e))?;

        let mut benchmarks = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DatabaseError::query("Failed to iterate search results", e))?
        {
            benchmarks.push(crate::types::BenchmarkData {
                id: row
                    .get(0)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark ID"))?,
                benchmark_name: row
                    .get(1)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark name"))?,
                prompt: row
                    .get(2)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark prompt"))?,
                content: row
                    .get(3)
                    .map_err(|_| DatabaseError::generic("Failed to get benchmark content"))?,
                created_at: row
                    .get(4)
                    .map_err(|_| DatabaseError::generic("Failed to get created_at"))?,
                updated_at: row
                    .get(5)
                    .map_err(|_| DatabaseError::generic("Failed to get updated_at"))?,
            });
        }

        info!(
            "[DB] Found {} benchmarks matching '{}'",
            benchmarks.len(),
            query_text
        );
        Ok(benchmarks)
    }

    /// Get the underlying connection for advanced operations
    pub fn connection(&self) -> &turso::Connection {
        &self.conn
    }
}

/// Simple benchmark statistics
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub total_benchmarks: i64,
    pub total_results: i64,
    pub average_score: f64,
}
