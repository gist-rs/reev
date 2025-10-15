//! Database reader operations for reev
//!
//! This module provides read-only database operations.

use super::types::*;
use crate::flow::types::FlowLog;
use anyhow::{Context, Result};

impl super::DatabaseWriter {
    /// Retrieves all flow logs for a specific benchmark
    pub async fn get_flow_logs(&self, benchmark_id: &str) -> Result<Vec<FlowLog>> {
        let query =
            "SELECT flow_data FROM flow_logs WHERE benchmark_id = ?1 ORDER BY start_time DESC";
        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare flow logs query")?;

        let mut rows = stmt
            .query([benchmark_id])
            .await
            .context("Failed to execute flow logs query")?;

        let mut flow_logs = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let flow_data: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let flow_log: FlowLog =
                serde_json::from_str(&flow_data).context("Failed to deserialize flow log")?;
            flow_logs.push(flow_log);
        }

        Ok(flow_logs)
    }

    /// Retrieves YML flow logs for a specific benchmark
    pub async fn get_yml_flow_logs(&self, benchmark_id: &str) -> Result<Vec<String>> {
        let query = "
            SELECT flow_data FROM flow_logs
            WHERE benchmark_id = ?1
            ORDER BY created_at DESC
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare YML flow logs query")?;

        let mut rows = stmt
            .query([benchmark_id])
            .await
            .context("Failed to execute YML flow logs query")?;

        let mut yml_logs = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let yml_content_result: Result<String, _> = row.get(0);
            let yml_content =
                yml_content_result.map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;

            if !yml_content.is_empty() {
                yml_logs.push(yml_content);
            }
        }

        Ok(yml_logs)
    }

    /// Retrieves YML TestResult for a specific benchmark and agent
    pub async fn get_yml_testresult(
        &self,
        benchmark_id: &str,
        agent_type: &str,
    ) -> Result<Option<String>> {
        let query = "
            SELECT yml_content FROM yml_testresults
            WHERE benchmark_id = ?1 AND agent_type = ?2
            ORDER BY created_at DESC
            LIMIT 1
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare YML TestResult query")?;

        let mut rows = stmt
            .query([benchmark_id, agent_type])
            .await
            .context("Failed to execute YML TestResult query")?;

        match rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            Some(row) => {
                let yml_content: String = row
                    .get(0)
                    .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
                Ok(Some(yml_content))
            }
            None => Ok(None),
        }
    }

    /// Retrieves agent performance summary by agent type
    pub async fn get_agent_performance(&self) -> Result<Vec<AgentPerformanceSummary>> {
        let query = "
            SELECT
                agent_type,
                COUNT(*) as total_benchmarks,
                AVG(score) as average_score,
                SUM(CASE WHEN final_status = 'Succeeded' THEN 1 ELSE 0 END) * 1.0 / COUNT(*) as success_rate
            FROM agent_performance
            GROUP BY agent_type
            ORDER BY agent_type;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare agent performance query")?;

        let mut rows = stmt
            .query(())
            .await
            .context("Failed to execute agent performance query")?;

        let mut summaries = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let agent_type: String = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let total_benchmarks: i64 = row
                .get(1)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let average_score: f64 = row
                .get(2)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let success_rate: f64 = row
                .get(3)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;

            let results = self.get_agent_results(&agent_type).await?;

            let summary = AgentPerformanceSummary {
                agent_type,
                total_benchmarks,
                average_score,
                success_rate,
                best_benchmarks: Vec::new(),
                worst_benchmarks: Vec::new(),
                results,
            };
            summaries.push(summary);
        }

        Ok(summaries)
    }

    /// Helper method to get detailed results for an agent type
    async fn get_agent_results(&self, agent_type: &str) -> Result<Vec<BenchmarkResult>> {
        let query = "
            SELECT
                id,
                benchmark_id,
                timestamp,
                score,
                final_status,
                execution_time_ms
            FROM agent_performance
            WHERE agent_type = ?1
            ORDER BY timestamp DESC;
        ";

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .context("Failed to prepare agent results query")?;

        let mut rows = stmt
            .query([agent_type])
            .await
            .context("Failed to execute agent results query")?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| anyhow::anyhow!("Row iteration error: {e}"))?
        {
            let id: i64 = row
                .get(0)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let benchmark_id: String = row
                .get(1)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let timestamp: String = row
                .get(2)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let score: f64 = row
                .get(3)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let final_status: String = row
                .get(4)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;
            let _execution_time_ms: u64 = row
                .get(5)
                .map_err(|e| anyhow::anyhow!("Column access error: {e}"))?;

            let result = BenchmarkResult {
                id: Some(id),
                benchmark_id,
                timestamp,
                prompt: String::new(),
                generated_instruction: String::new(),
                final_on_chain_state: String::new(),
                final_status,
                score,
            };
            results.push(result);
        }

        Ok(results)
    }
}
