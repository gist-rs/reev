//! Execution states management module
//!
//! Provides database operations for managing execution state information
//! stored during benchmark execution via CLI-based runner process.

use crate::error::{DatabaseError, Result};
use reev_types::{ExecutionState, ExecutionStatus};
use tracing::{debug, info};
use turso::Connection;

/// Execution states database operations
pub struct ExecutionStatesWriter<'a> {
    conn: &'a Connection,
}

impl<'a> ExecutionStatesWriter<'a> {
    /// Create new execution states writer
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Store execution state in database
    pub async fn store_execution_state(&self, state: &ExecutionState) -> Result<()> {
        debug!("[DB] Storing execution state: {}", state.execution_id);

        let metadata_json = serde_json::to_string(&state.metadata)
            .map_err(|e| DatabaseError::serialization("Failed to serialize metadata", e))?;

        let result_data_json = state
            .result_data
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| DatabaseError::serialization("Failed to serialize result data", e))?;

        // Use proper UPSERT with ON CONFLICT DO UPDATE to prevent index corruption
        // This approach is proven to work reliably in Turso testing
        let query = r#"
            INSERT INTO execution_states (
                execution_id, benchmark_id, agent, status,
                created_at, updated_at, progress, error_message,
                result_data, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(execution_id) DO UPDATE SET
                benchmark_id = excluded.benchmark_id,
                agent = excluded.agent,
                status = excluded.status,
                updated_at = excluded.updated_at,
                progress = excluded.progress,
                error_message = excluded.error_message,
                result_data = excluded.result_data,
                metadata = excluded.metadata;
        "#;

        self.conn
            .execute(
                query,
                [
                    state.execution_id.clone(),
                    state.benchmark_id.clone(),
                    state.agent.clone(),
                    serde_json::to_string(&state.status).map_err(|e| {
                        DatabaseError::serialization("Failed to serialize status", e)
                    })?,
                    state.created_at.timestamp().to_string(),
                    state.updated_at.timestamp().to_string(),
                    state.progress.unwrap_or(0.0).to_string(),
                    state
                        .error_message
                        .as_ref()
                        .unwrap_or(&String::new())
                        .clone(),
                    result_data_json.unwrap_or_else(|| "null".to_string()),
                    metadata_json,
                ],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(
                    format!("Failed to store execution state: {}", state.execution_id),
                    e,
                )
            })?;

        info!("[DB] Stored execution state: {}", state.execution_id);
        Ok(())
    }

    /// Get execution state by ID
    pub async fn get_execution_state(&self, execution_id: &str) -> Result<Option<ExecutionState>> {
        debug!("[DB] Getting execution state: {}", execution_id);

        let mut rows = self
            .conn
            .query(
                r#"
                SELECT execution_id, benchmark_id, agent, status,
                       created_at, updated_at, progress, error_message,
                       result_data, metadata
                FROM execution_states
                WHERE execution_id = ?
                "#,
                [execution_id.to_string()],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(format!("Failed to get execution state: {execution_id}"), e)
            })?;

        if let Some(row) = rows.next().await? {
            let state = self.row_to_execution_state(row)?;
            debug!("[DB] Found execution state: {}", execution_id);
            Ok(Some(state))
        } else {
            debug!("[DB] Execution state not found: {}", execution_id);
            Ok(None)
        }
    }

    /// Update execution status
    pub async fn update_execution_status(
        &self,
        execution_id: &str,
        status: ExecutionStatus,
    ) -> Result<()> {
        debug!(
            "[DB] Updating execution status: {} -> {:?}",
            execution_id, status
        );

        let now = chrono::Utc::now().timestamp();

        self.conn
            .execute(
                r#"
                UPDATE execution_states
                SET status = ?, updated_at = ?
                WHERE execution_id = ?
                "#,
                [
                    serde_json::to_string(&status).map_err(|e| {
                        DatabaseError::serialization("Failed to serialize status", e)
                    })?,
                    now.to_string(),
                    execution_id.to_string(),
                ],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(
                    format!("Failed to update execution status: {execution_id}"),
                    e,
                )
            })?;

        info!(
            "[DB] Updated execution status: {} -> {:?}",
            execution_id, status
        );
        Ok(())
    }

    /// Update execution progress
    pub async fn update_execution_progress(&self, execution_id: &str, progress: f64) -> Result<()> {
        debug!(
            "[DB] Updating execution progress: {} -> {}",
            execution_id, progress
        );

        let now = chrono::Utc::now().timestamp();

        self.conn
            .execute(
                r#"
                UPDATE execution_states
                SET progress = ?, updated_at = ?
                WHERE execution_id = ?
                "#,
                [
                    progress.to_string(),
                    now.to_string(),
                    execution_id.to_string(),
                ],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(
                    format!("Failed to update execution progress: {execution_id}"),
                    e,
                )
            })?;

        debug!(
            "[DB] Updated execution progress: {} -> {}",
            execution_id, progress
        );
        Ok(())
    }

    /// Set execution error
    pub async fn set_execution_error(&self, execution_id: &str, error_message: &str) -> Result<()> {
        debug!("[DB] Setting execution error: {}", execution_id);

        let now = chrono::Utc::now().timestamp();
        let status = ExecutionStatus::Failed;

        self.conn
            .execute(
                r#"
                UPDATE execution_states
                SET status = ?, error_message = ?, updated_at = ?
                WHERE execution_id = ?
                "#,
                [
                    serde_json::to_string(&status).map_err(|e| {
                        DatabaseError::serialization("Failed to serialize status", e)
                    })?,
                    error_message.to_string(),
                    now.to_string(),
                    execution_id.to_string(),
                ],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(format!("Failed to set execution error: {execution_id}"), e)
            })?;

        info!(
            "[DB] Set execution error: {} - {}",
            execution_id, error_message
        );
        Ok(())
    }

    /// Complete execution with result data
    pub async fn complete_execution(
        &self,
        execution_id: &str,
        result_data: &serde_json::Value,
    ) -> Result<()> {
        debug!("[DB] Completing execution: {}", execution_id);

        let now = chrono::Utc::now().timestamp();
        let status = ExecutionStatus::Completed;
        let result_json = serde_json::to_string(result_data)
            .map_err(|e| DatabaseError::serialization("Failed to serialize result data", e))?;

        self.conn
            .execute(
                r#"
                UPDATE execution_states
                SET status = ?, result_data = ?, progress = 1.0, updated_at = ?
                WHERE execution_id = ?
                "#,
                [
                    serde_json::to_string(&status).map_err(|e| {
                        DatabaseError::serialization("Failed to serialize status", e)
                    })?,
                    result_json,
                    now.to_string(),
                    execution_id.to_string(),
                ],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(format!("Failed to complete execution: {execution_id}"), e)
            })?;

        info!("[DB] Completed execution: {}", execution_id);
        Ok(())
    }

    /// List executions by status
    pub async fn list_executions_by_status(
        &self,
        status: ExecutionStatus,
        limit: Option<i32>,
    ) -> Result<Vec<ExecutionState>> {
        debug!("[DB] Listing executions with status: {:?}", status);

        let status_json = serde_json::to_string(&status)
            .map_err(|e| DatabaseError::serialization("Failed to serialize status", e))?;

        let mut results = Vec::new();

        if let Some(limit) = limit {
            let mut rows = self
                .conn
                .query(
                    r#"
                    SELECT execution_id, benchmark_id, agent, status,
                           created_at, updated_at, progress, error_message,
                           result_data, metadata
                    FROM execution_states
                    WHERE status = ?
                    ORDER BY created_at DESC
                    LIMIT ?
                    "#,
                    [status_json, limit.to_string()],
                )
                .await
                .map_err(|e| {
                    DatabaseError::query(
                        format!("Failed to list executions by status: {status:?}"),
                        e,
                    )
                })?;

            while let Some(row) = rows.next().await? {
                results.push(self.row_to_execution_state(row)?);
            }
        } else {
            let mut rows = self
                .conn
                .query(
                    r#"
                    SELECT execution_id, benchmark_id, agent, status,
                           created_at, updated_at, progress, error_message,
                           result_data, metadata
                    FROM execution_states
                    WHERE status = ?
                    ORDER BY created_at DESC
                    "#,
                    [status_json],
                )
                .await
                .map_err(|e| {
                    DatabaseError::query(
                        format!("Failed to list executions by status: {status:?}"),
                        e,
                    )
                })?;

            while let Some(row) = rows.next().await? {
                results.push(self.row_to_execution_state(row)?);
            }
        }

        debug!(
            "[DB] Found {} executions with status: {:?}",
            results.len(),
            status
        );
        Ok(results)
    }

    /// Clean up old completed executions
    pub async fn cleanup_old_executions(&self, days_old: i32) -> Result<u32> {
        debug!("[DB] Cleaning up executions older than {} days", days_old);

        let cutoff_time =
            (chrono::Utc::now() - chrono::Duration::days(days_old as i64)).timestamp();

        let result = self
            .conn
            .execute(
                r#"
                DELETE FROM execution_states
                WHERE status IN ('completed', 'failed', 'stopped', 'timeout')
                AND updated_at < ?
                "#,
                [cutoff_time.to_string()],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(
                    format!("Failed to cleanup old executions ({days_old} days)"),
                    e,
                )
            })?;

        info!("[DB] Cleaned up {} old executions", result);
        Ok(result as u32)
    }

    /// Convert database row to ExecutionState
    fn row_to_execution_state(&self, row: turso::Row) -> Result<ExecutionState> {
        let status_str: String = row.get(3)?; // status is at index 3
        let status: ExecutionStatus = serde_json::from_str(&status_str)
            .map_err(|e| DatabaseError::serialization("Failed to deserialize status", e))?;

        let metadata_str: String = row.get(9)?; // metadata is at index 9
        let metadata: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&metadata_str)
                .map_err(|e| DatabaseError::serialization("Failed to deserialize metadata", e))?;

        let result_data_str: String = row.get(8)?; // result_data is at index 8
        let result_data = if result_data_str == "null" {
            None
        } else {
            Some(serde_json::from_str(&result_data_str).map_err(|e| {
                DatabaseError::serialization("Failed to deserialize result data", e)
            })?)
        };

        let created_at_timestamp: i64 = row.get(4)?; // created_at is at index 4 (INTEGER)
        let updated_at_timestamp: i64 = row.get(5)?; // updated_at is at index 5 (INTEGER)

        Ok(ExecutionState {
            execution_id: row.get(0)?, // execution_id is at index 0
            benchmark_id: row.get(1)?, // benchmark_id is at index 1
            agent: row.get(2)?,        // agent is at index 2
            status,
            created_at: chrono::DateTime::from_timestamp(created_at_timestamp, 0)
                .unwrap_or_else(chrono::Utc::now),
            updated_at: chrono::DateTime::from_timestamp(updated_at_timestamp, 0)
                .unwrap_or_else(chrono::Utc::now),
            progress: Some(row.get::<f64>(6)?), // progress is at index 6 (REAL)
            error_message: {
                let error_str: String = row.get(7)?; // error_message is at index 7
                if error_str.is_empty() {
                    None
                } else {
                    Some(error_str)
                }
            },
            result_data,
            metadata,
        })
    }

    /// List executions by benchmark ID
    pub async fn list_execution_states_by_benchmark(
        &self,
        benchmark_id: &str,
    ) -> Result<Vec<ExecutionState>> {
        debug!(
            "[DB] Listing execution states for benchmark: {}",
            benchmark_id
        );

        // Reconstruct full path format that's stored in database
        let full_benchmark_id = if benchmark_id.starts_with("benchmarks/") {
            benchmark_id.to_string()
        } else {
            format!("benchmarks/{benchmark_id}.yml")
        };

        debug!(
            "[DB] Using full benchmark path for query: {}",
            full_benchmark_id
        );

        let mut results = Vec::new();

        let mut rows = self
            .conn
            .query(
                r#"
                    SELECT execution_id, benchmark_id, agent, status,
                           created_at, updated_at, progress, error_message,
                           result_data, metadata
                    FROM execution_states
                    WHERE benchmark_id = ?
                    ORDER BY created_at DESC
                    LIMIT 10
                    "#,
                [full_benchmark_id],
            )
            .await
            .map_err(|e| {
                DatabaseError::query(
                    format!("Failed to list execution states for benchmark: {benchmark_id}"),
                    e,
                )
            })?;

        while let Some(row) = rows.next().await? {
            results.push(self.row_to_execution_state(row)?);
        }

        debug!(
            "[DB] Found {} execution states for benchmark: {}",
            results.len(),
            benchmark_id
        );
        Ok(results)
    }
}

/// Extension trait for DatabaseWriter to add execution state methods
#[allow(async_fn_in_trait)]
pub trait ExecutionStateExt {
    /// Store execution state in database
    async fn store_execution_state(&self, state: &ExecutionState) -> Result<()>;

    /// Get execution state by ID
    async fn get_execution_state(&self, execution_id: &str) -> Result<Option<ExecutionState>>;

    /// Update execution status
    async fn update_execution_status(
        &self,
        execution_id: &str,
        status: ExecutionStatus,
    ) -> Result<()>;

    /// Update execution progress
    async fn update_execution_progress(&self, execution_id: &str, progress: f64) -> Result<()>;

    /// Set execution error
    async fn set_execution_error(&self, execution_id: &str, error_message: &str) -> Result<()>;

    /// Complete execution with result data
    async fn complete_execution(
        &self,
        execution_id: &str,
        result_data: &serde_json::Value,
    ) -> Result<()>;

    /// List executions by status
    async fn list_executions_by_status(
        &self,
        status: ExecutionStatus,
        limit: Option<i32>,
    ) -> Result<Vec<ExecutionState>>;

    /// Clean up old completed executions
    async fn cleanup_old_executions(&self, days_old: i32) -> Result<u32>;
}

// TODO: Fix ExecutionStateExt blanket implementation
// The blanket implementation is temporarily disabled due to connection access issues
// impl<T: DatabaseWriterTrait> ExecutionStateExt for T {
//     // Implementation will be added here once connection access is resolved
// }
