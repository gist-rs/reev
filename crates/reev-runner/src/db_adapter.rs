use anyhow::Result;
use async_trait::async_trait;
use reev_lib::flow::types::FlowLog;
use reev_lib::flow::{FlowError, FlowLogDatabase};
use std::sync::Arc;
use tracing::{debug, error};

use crate::db;

use super::db::Db;

/// Database adapter for FlowLogger
pub struct FlowLogDatabaseAdapter {
    db: Arc<Db>,
}

impl FlowLogDatabaseAdapter {
    /// Create a new adapter with database connection
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FlowLogDatabase for FlowLogDatabaseAdapter {
    async fn insert_flow_log(&self, flow_log: &FlowLog) -> Result<i64, FlowError> {
        debug!(
            session_id = %flow_log.session_id,
            benchmark_id = %flow_log.benchmark_id,
            "Inserting flow log into database"
        );

        self.db.insert_flow_log(flow_log).await.map_err(|e| {
            error!("Failed to insert flow log: {}", e);
            FlowError::database(format!("Database insertion failed: {e}"))
        })
    }

    async fn insert_agent_performance(
        &self,
        benchmark_id: &str,
        agent_type: &str,
        score: f64,
        final_status: &str,
        execution_time_ms: u64,
        timestamp: &str,
        flow_log_id: Option<i64>,
    ) -> Result<(), FlowError> {
        debug!(
            benchmark_id = %benchmark_id,
            agent_type = %agent_type,
            score = %score,
            "Inserting agent performance into database"
        );

        let performance_data = db::AgentPerformanceData {
            benchmark_id: benchmark_id.to_string(),
            agent_type: agent_type.to_string(),
            score,
            final_status: final_status.to_string(),
            execution_time_ms,
            timestamp: timestamp.to_string(),
            flow_log_id,
        };

        self.db
            .insert_agent_performance(&performance_data)
            .await
            .map_err(|e| {
                error!("Failed to insert agent performance: {}", e);
                FlowError::database(format!("Agent performance insertion failed: {e}"))
            })
    }
}
