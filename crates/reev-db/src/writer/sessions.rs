//! Session management operations
//!
//! Provides unified session tracking for both TUI and Web interfaces,
//! ensuring consistent database writes and reliable logging.

use crate::{
    error::{DatabaseError, Result},
    types::{SessionFilter, SessionInfo, SessionResult},
};
use serde_json::Value;
use tracing::info;

use super::core::DatabaseWriter;

/// Tool call data for database storage
#[derive(Debug, Clone)]
pub struct ToolCallData {
    pub session_id: String,
    pub tool_name: String,
    pub start_time: u64,
    pub execution_time_ms: u64,
    pub input_params: Value,
    pub output_result: Value,
    pub status: String,
    pub error_message: Option<String>,
}

impl DatabaseWriter {
    /// Create a new execution session
    pub async fn create_session(&self, session: &SessionInfo) -> Result<()> {
        info!(
            session_id = %session.session_id,
            benchmark_id = %session.benchmark_id,
            agent_type = %session.agent_type,
            interface = %session.interface,
            "Creating execution session"
        );

        self.conn
            .execute(
                "INSERT INTO execution_sessions (session_id, benchmark_id, agent_type, interface, start_time, status)
                 VALUES (?, ?, ?, ?, ?, 'running')",
                [
                    session.session_id.clone(),
                    session.benchmark_id.clone(),
                    session.agent_type.clone(),
                    session.interface.clone(),
                    session.start_time.to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to create session", e))?;

        info!("Session created successfully: {}", session.session_id);
        Ok(())
    }

    /// Complete an execution session with results
    pub async fn complete_session(&self, session_id: &str, result: &SessionResult) -> Result<()> {
        info!(
            session_id = %session_id,
            score = %result.score,
            final_status = %result.final_status,
            "Completing execution session"
        );

        self.conn
            .execute(
                "UPDATE execution_sessions
             SET end_time = ?, status = ?, score = ?, final_status = ?
             WHERE session_id = ?",
                [
                    result.end_time.to_string(),
                    result.final_status.clone(),
                    result.score.to_string(),
                    result.final_status.clone(),
                    session_id.to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to complete session", e))?;

        info!("Session completed successfully: {}", session_id);
        Ok(())
    }

    /// Store complete session log content
    pub async fn store_complete_log(&self, session_id: &str, log_content: &str) -> Result<()> {
        info!(
            session_id = %session_id,
            content_length = log_content.len(),
            "Storing complete session log"
        );

        // Update session with log file reference
        let log_file_path = format!("logs/sessions/{session_id}.json");
        self.conn
            .execute(
                "UPDATE execution_sessions SET log_file_path = ? WHERE session_id = ?",
                [log_file_path.clone(), session_id.to_string()],
            )
            .await
            .map_err(|e| {
                DatabaseError::operation_with_source("Failed to update session log path", e)
            })?;

        // Store full content in session_logs table - use INSERT without OR REPLACE for compatibility
        self.conn
            .execute(
                "INSERT INTO session_logs (session_id, content, file_size) VALUES (?, ?, ?)",
                [
                    session_id.to_string(),
                    log_content.to_string(),
                    log_content.len().to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to store session log", e))?;

        info!("Session log stored successfully: {}", session_id);
        Ok(())
    }

    /// Get complete session log content
    pub async fn get_session_log(&self, session_id: &str) -> Result<String> {
        info!(session_id = %session_id, "Retrieving session log");

        let mut rows = self
            .conn
            .query(
                "SELECT content FROM session_logs WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to retrieve session log", e))?;

        if let Some(row) = rows.next().await? {
            let log_content: String = row.get(0).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse session log", e)
            })?;

            info!("Session log retrieved: {} chars", log_content.len());
            Ok(log_content)
        } else {
            Err(DatabaseError::record_not_found(session_id, "session_logs"))
        }
    }

    /// List sessions with optional filtering
    pub async fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionInfo>> {
        let query = if filter.benchmark_id.is_some() {
            "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
             FROM execution_sessions WHERE benchmark_id = ? ORDER BY start_time DESC"
        } else if filter.agent_type.is_some() {
            "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
             FROM execution_sessions WHERE agent_type = ? ORDER BY start_time DESC"
        } else if filter.interface.is_some() {
            "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
             FROM execution_sessions WHERE interface = ? ORDER BY start_time DESC"
        } else if filter.status.is_some() {
            "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
             FROM execution_sessions WHERE status = ? ORDER BY start_time DESC"
        } else {
            "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
             FROM execution_sessions ORDER BY start_time DESC"
        };

        let mut stmt = self
            .conn
            .prepare(query)
            .await
            .map_err(|e| DatabaseError::query("Failed to prepare sessions query", e))?;

        let sessions = if let Some(benchmark_id) = &filter.benchmark_id {
            let mut rows = stmt
                .query([benchmark_id.as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query sessions", e))?;

            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(SessionInfo {
                    session_id: row.get(0)?,
                    benchmark_id: row.get(1)?,
                    agent_type: row.get(2)?,
                    interface: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    status: row.get(6)?,
                    score: row.get(7)?,
                    final_status: row.get(8)?,
                });
            }
            results
        } else if let Some(agent_type) = &filter.agent_type {
            let mut rows = stmt
                .query([agent_type.as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query sessions", e))?;

            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(SessionInfo {
                    session_id: row.get(0)?,
                    benchmark_id: row.get(1)?,
                    agent_type: row.get(2)?,
                    interface: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    status: row.get(6)?,
                    score: row.get(7)?,
                    final_status: row.get(8)?,
                });
            }
            results
        } else if let Some(interface) = &filter.interface {
            let mut rows = stmt
                .query([interface.as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query sessions", e))?;

            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(SessionInfo {
                    session_id: row.get(0)?,
                    benchmark_id: row.get(1)?,
                    agent_type: row.get(2)?,
                    interface: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    status: row.get(6)?,
                    score: row.get(7)?,
                    final_status: row.get(8)?,
                });
            }
            results
        } else if let Some(status) = &filter.status {
            let mut rows = stmt
                .query([status.as_str()])
                .await
                .map_err(|e| DatabaseError::query("Failed to query sessions", e))?;

            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(SessionInfo {
                    session_id: row.get(0)?,
                    benchmark_id: row.get(1)?,
                    agent_type: row.get(2)?,
                    interface: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    status: row.get(6)?,
                    score: row.get(7)?,
                    final_status: row.get(8)?,
                });
            }
            results
        } else {
            let mut rows = stmt
                .query(())
                .await
                .map_err(|e| DatabaseError::query("Failed to query sessions", e))?;

            let mut results = Vec::new();
            while let Some(row) = rows.next().await? {
                results.push(SessionInfo {
                    session_id: row.get(0)?,
                    benchmark_id: row.get(1)?,
                    agent_type: row.get(2)?,
                    interface: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    status: row.get(6)?,
                    score: row.get(7)?,
                    final_status: row.get(8)?,
                });
            }
            results
        };

        info!("Retrieved {} sessions", sessions.len());
        Ok(sessions)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionInfo>> {
        let mut rows = self
            .conn
            .query(
                "SELECT session_id, benchmark_id, agent_type, interface, start_time, end_time, status, score, final_status
                 FROM execution_sessions WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get session", e))?;

        if let Some(row) = rows.next().await? {
            Ok(Some(SessionInfo {
                session_id: row.get(0)?,
                benchmark_id: row.get(1)?,
                agent_type: row.get(2)?,
                interface: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                status: row.get(6)?,
                score: row.get(7)?,
                final_status: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete session and all associated data
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        info!(session_id = %session_id, "Deleting session");

        // Delete session log first (foreign key dependency)
        self.conn
            .execute(
                "DELETE FROM session_logs WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to delete session log", e))?;

        // Delete session
        self.conn
            .execute(
                "DELETE FROM execution_sessions WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::operation_with_source("Failed to delete session", e))?;

        info!("Session deleted successfully: {}", session_id);
        Ok(())
    }

    /// Update session status
    pub async fn update_session_status(&self, session_id: &str, status: &str) -> Result<()> {
        info!(
            session_id = %session_id,
            status = %status,
            "Updating session status"
        );

        self.conn
            .execute(
                "UPDATE execution_sessions SET status = ? WHERE session_id = ?",
                [status.to_string(), session_id.to_string()],
            )
            .await
            .map_err(|e| {
                DatabaseError::operation_with_source("Failed to update session status", e)
            })?;

        Ok(())
    }

    /// Store tool call details for a session
    pub async fn store_tool_call(&self, tool_call: &ToolCallData) -> Result<()> {
        info!(
            session_id = %tool_call.session_id,
            tool_name = %tool_call.tool_name,
            execution_time_ms = tool_call.execution_time_ms,
            status = %tool_call.status,
            "Storing tool call details"
        );

        let input_params_json = serde_json::to_string(&tool_call.input_params)
            .map_err(|e| DatabaseError::serialization("Failed to serialize input_params", e))?;

        let output_result_json = serde_json::to_string(&tool_call.output_result)
            .map_err(|e| DatabaseError::serialization("Failed to serialize output_result", e))?;

        // Metadata field removed

        self.conn
            .execute(
                "INSERT INTO session_tool_calls
                 (session_id, tool_name, start_time, execution_time_ms, input_params, output_result, status, error_message)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                [
                    tool_call.session_id.clone(),
                    tool_call.tool_name.clone(),
                    tool_call.start_time.to_string(),
                    tool_call.execution_time_ms.to_string(),
                    input_params_json,
                    output_result_json,
                    tool_call.status.clone(),
                    tool_call.error_message.clone().unwrap_or_default(),
                ],
            )
            .await
            .map_err(|_e| DatabaseError::operation("Failed to store tool call"))?;

        Ok(())
    }

    /// Store multiple tool calls for a session with consolidation
    pub async fn store_tool_calls(&self, tool_calls: &[ToolCallData]) -> Result<()> {
        if tool_calls.is_empty() {
            return Ok(());
        }

        info!(
            session_id = %tool_calls[0].session_id,
            count = tool_calls.len(),
            "Storing multiple tool call details with consolidation"
        );

        for tool_call in tool_calls {
            self.store_tool_call_consolidated(tool_call).await?;
        }

        Ok(())
    }

    /// Store tool call with automatic consolidation logic
    /// Detects and merges duplicate entries for the same tool execution
    pub async fn store_tool_call_consolidated(&self, tool_call: &ToolCallData) -> Result<()> {
        info!(
            session_id = %tool_call.session_id,
            tool_name = %tool_call.tool_name,
            execution_time_ms = tool_call.execution_time_ms,
            status = %tool_call.status,
            "Storing tool call with consolidation logic"
        );

        // Check for existing entry with same session_id, tool_name, and similar start_time
        let mut existing_calls = self
            .conn
            .query(
                "SELECT id, input_params, output_result, execution_time_ms, status
                 FROM session_tool_calls
                 WHERE session_id = ? AND tool_name = ?
                 AND ABS(start_time - ?) <= 1
                 ORDER BY created_at DESC",
                [
                    tool_call.session_id.clone(),
                    tool_call.tool_name.clone(),
                    tool_call.start_time.to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to check for existing tool calls", e))?;

        if let Some(row) = existing_calls.next().await? {
            // Found existing entry - consolidate with new data
            let existing_id: i64 = row.get(0)?;
            let existing_input: String = row.get(1)?;
            let existing_output: String = row.get(2)?;
            let existing_execution_time: i64 = row.get(3)?;
            let _existing_status: String = row.get(4)?;

            // Merge input_params (prefer non-empty)
            let merged_input = if existing_input.trim().is_empty() || existing_input == "{}" {
                serde_json::to_string(&tool_call.input_params).map_err(|e| {
                    DatabaseError::serialization("Failed to serialize input_params", e)
                })?
            } else {
                existing_input
            };

            // Merge output_result (prefer non-empty from either source)
            let new_output_has_content = match &tool_call.output_result {
                serde_json::Value::Object(o) => !o.is_empty(),
                serde_json::Value::Array(a) => !a.is_empty(),
                serde_json::Value::String(s) => !s.is_empty(),
                _ => false,
            };

            let merged_output = if new_output_has_content {
                serde_json::to_string(&tool_call.output_result).map_err(|e| {
                    DatabaseError::serialization("Failed to serialize output_result", e)
                })?
            } else {
                existing_output
            };

            // Prefer the non-zero execution time, but always prefer larger execution time
            let merged_execution_time = if tool_call.execution_time_ms > 0 {
                let tool_time = tool_call.execution_time_ms;
                if tool_time > existing_execution_time as u64 {
                    tool_time
                } else {
                    existing_execution_time as u64
                }
            } else {
                existing_execution_time as u64
            };

            // Update the existing entry with merged data
            self.conn
                .execute(
                    "UPDATE session_tool_calls
                     SET input_params = ?, output_result = ?, execution_time_ms = ?, status = ?
                     WHERE id = ?",
                    [
                        merged_input,
                        merged_output,
                        merged_execution_time.to_string(),
                        tool_call.status.clone(),
                        existing_id.to_string(),
                    ],
                )
                .await
                .map_err(|_e| {
                    DatabaseError::operation("Failed to update consolidated tool call")
                })?;

            info!(existing_id, "Consolidated tool call with existing entry");
        } else {
            // No existing entry - create new one
            self.store_tool_call(tool_call).await?;
        }

        Ok(())
    }

    /// Get tool calls for a specific session
    pub async fn get_session_tool_calls(&self, session_id: &str) -> Result<Vec<ToolCallData>> {
        info!(
            session_id = %session_id,
            "Retrieving tool calls for session"
        );

        let mut rows = self
            .conn
            .query(
                "SELECT session_id, tool_name, start_time, execution_time_ms, input_params, output_result, status, error_message
                 FROM session_tool_calls
                 WHERE session_id = ?
                 ORDER BY start_time ASC",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to retrieve tool calls", e))?;

        let mut tool_calls = Vec::new();
        while let Some(row) = rows.next().await? {
            let input_params: String = row.get(4).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse input_params", e)
            })?;
            let input_params_value: Value = serde_json::from_str(&input_params).map_err(|e| {
                DatabaseError::serialization("Failed to deserialize input_params", e)
            })?;

            let output_result: String = row.get(5).map_err(|e| {
                DatabaseError::generic_with_source("Failed to parse output_result", e)
            })?;
            let output_result_value: Value = serde_json::from_str(&output_result).map_err(|e| {
                DatabaseError::serialization("Failed to deserialize output_result", e)
            })?;

            tool_calls.push(ToolCallData {
                session_id: row.get(0)?,
                tool_name: row.get(1)?,
                start_time: row.get(2)?,
                execution_time_ms: row.get(3)?,
                input_params: input_params_value,
                output_result: output_result_value,
                status: row.get(6)?,
                error_message: row.get(7).ok(),
            });
        }

        info!(
            "Retrieved {} tool calls for session {}",
            tool_calls.len(),
            session_id
        );
        Ok(tool_calls)
    }

    /// Get tool calls aggregated by tool name for analysis
    pub async fn get_tool_call_stats(&self, session_id: &str) -> Result<Value> {
        info!(
            session_id = %session_id,
            "Getting tool call statistics"
        );

        let mut rows = self
            .conn
            .query(
                "SELECT
                    tool_name,
                    COUNT(*) as call_count,
                    AVG(execution_time_ms) as avg_time_ms,
                    MIN(execution_time_ms) as min_time_ms,
                    MAX(execution_time_ms) as max_time_ms,
                    SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success_count,
                    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as error_count,
                    SUM(CASE WHEN status = 'timeout' THEN 1 ELSE 0 END) as timeout_count
                 FROM session_tool_calls
                 WHERE session_id = ?
                 GROUP BY tool_name
                 ORDER BY call_count DESC",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::query("Failed to get tool call stats", e))?;

        let mut stats = Vec::new();
        while let Some(row) = rows.next().await? {
            stats.push(serde_json::json!({
                "tool_name": row.get::<String>(0)?,
                "call_count": row.get::<i64>(1)?,
                "avg_time_ms": row.get::<f64>(2)?,
                "min_time_ms": row.get::<i64>(3)?,
                "max_time_ms": row.get::<i64>(4)?,
                "success_count": row.get::<i64>(5)?,
                "error_count": row.get::<i64>(6)?,
                "timeout_count": row.get::<i64>(7)?
            }));
        }

        Ok(serde_json::json!({
            "session_id": session_id,
            "tool_stats": stats,
            "total_tools": stats.len()
        }))
    }
}
