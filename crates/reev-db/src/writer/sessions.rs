//! Session management operations
//!
//! Provides unified session tracking for both TUI and Web interfaces,
//! ensuring consistent database writes and reliable logging.

use crate::{
    error::{DatabaseError, Result},
    types::{SessionFilter, SessionInfo, SessionResult},
};
use tracing::info;

use super::core::DatabaseWriter;

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
            .map_err(|e| DatabaseError::operation("Failed to create session", e))?;

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
             SET end_time = ?, status = 'completed', score = ?, final_status = ?
             WHERE session_id = ?",
                [
                    result.end_time.to_string(),
                    result.score.to_string(),
                    result.final_status.clone(),
                    session_id.to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation("Failed to complete session", e))?;

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
            .map_err(|e| DatabaseError::operation("Failed to update session log path", e))?;

        // Store full content in session_logs table
        self.conn
            .execute(
                "INSERT OR REPLACE INTO session_logs (session_id, content, file_size) VALUES (?, ?, ?)",
                [
                    session_id.to_string(),
                    log_content.to_string(),
                    log_content.len().to_string(),
                ],
            )
            .await
            .map_err(|e| DatabaseError::operation("Failed to store session log", e))?;

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
                    start_time: {
                        let time_str: String = row.get(4)?;
                        time_str.parse().unwrap_or(0)
                    },
                    end_time: {
                        let time_opt: Option<String> = row.get(5)?;
                        time_opt.and_then(|s| s.parse().ok())
                    },
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
                    start_time: {
                        let time_str: String = row.get(4)?;
                        time_str.parse().unwrap_or(0)
                    },
                    end_time: {
                        let time_opt: Option<String> = row.get(5)?;
                        time_opt.and_then(|s| s.parse().ok())
                    },
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
                    start_time: {
                        let time_str: String = row.get(4)?;
                        time_str.parse().unwrap_or(0)
                    },
                    end_time: {
                        let time_opt: Option<String> = row.get(5)?;
                        time_opt.and_then(|s| s.parse().ok())
                    },
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
                    start_time: {
                        let time_str: String = row.get(4)?;
                        time_str.parse().unwrap_or(0)
                    },
                    end_time: {
                        let time_opt: Option<String> = row.get(5)?;
                        time_opt.and_then(|s| s.parse().ok())
                    },
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
                    start_time: {
                        let time_str: String = row.get(4)?;
                        time_str.parse().unwrap_or(0)
                    },
                    end_time: {
                        let time_opt: Option<String> = row.get(5)?;
                        time_opt.and_then(|s| s.parse().ok())
                    },
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
                start_time: {
                    let time_str: String = row.get(4)?;
                    time_str.parse().unwrap_or(0)
                },
                end_time: {
                    let time_opt: Option<String> = row.get(5)?;
                    time_opt.and_then(|s| s.parse().ok())
                },
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
            .map_err(|e| DatabaseError::operation("Failed to delete session log", e))?;

        // Delete session
        self.conn
            .execute(
                "DELETE FROM execution_sessions WHERE session_id = ?",
                [session_id],
            )
            .await
            .map_err(|e| DatabaseError::operation("Failed to delete session", e))?;

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
            .map_err(|e| DatabaseError::operation("Failed to update session status", e))?;

        Ok(())
    }
}
