//! Database-specific types and utilities for flow logging
//!
//! This module provides database-friendly versions of the flow types,
//! conversion utilities, and helper functions for database operations.

use crate::error::FlowError;
use crate::types::*;
use crate::utils::FlowUtils;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database-friendly flow log structure
///
/// This wraps the core FlowLog with database-specific fields and
/// uses string timestamps for better database compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBFlowLog {
    /// Core flow log data
    pub flow: FlowLog,
    /// Database-specific fields
    pub id: Option<i64>,
    /// Creation timestamp
    pub created_at: Option<String>,
}

impl DBFlowLog {
    /// Create a new DBFlowLog from a FlowLog
    pub fn new(flow: FlowLog) -> Self {
        Self {
            flow,
            id: None,
            created_at: Some(Utc::now().to_rfc3339()),
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.flow.session_id
    }

    /// Get benchmark ID
    pub fn benchmark_id(&self) -> &str {
        &self.flow.benchmark_id
    }

    /// Get agent type
    pub fn agent_type(&self) -> &str {
        &self.flow.agent_type
    }

    /// Get start time as RFC3339 string
    pub fn start_time(&self) -> Result<String, FlowError> {
        FlowUtils::system_time_to_rfc3339(self.flow.start_time)
    }

    /// Get end time as RFC3339 string
    pub fn end_time(&self) -> Result<Option<String>, FlowError> {
        match self.flow.end_time {
            Some(time) => Ok(Some(FlowUtils::system_time_to_rfc3339(time)?)),
            None => Ok(None),
        }
    }

    /// Get events as JSON string
    pub fn events_json(&self) -> Result<String, FlowError> {
        serde_json::to_string(&self.flow.events).map_err(Into::into)
    }

    /// Get final result as JSON string
    pub fn final_result_json(&self) -> Result<Option<String>, FlowError> {
        match &self.flow.final_result {
            Some(result) => Ok(Some(serde_json::to_string(result)?)),
            None => Ok(None),
        }
    }

    /// Check if flow is completed
    pub fn is_completed(&self) -> bool {
        self.flow.end_time.is_some()
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<u64> {
        FlowUtils::calculate_duration(&self.flow).map(|d| d.as_millis() as u64)
    }
}

/// Conversion trait between FlowLog and database storage
pub trait DBFlowLogConverter {
    /// Convert from database storage format
    fn from_db_storage(params: DBStorageParams) -> Result<Self, FlowError>
    where
        Self: Sized;

    /// Convert to database storage format
    fn to_db_storage(&self) -> Result<DBStorageFormat, FlowError>;
}

/// Database storage format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBStorageFormat {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub flow_data: String,
    pub final_result: Option<String>,
    pub id: Option<i64>,
    pub created_at: Option<String>,
}

/// Parameters for database storage operations
#[derive(Debug, Clone)]
pub struct DBStorageParams {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub flow_data: String,
    pub final_result: Option<String>,
    pub id: Option<i64>,
    pub created_at: Option<String>,
}

impl DBFlowLogConverter for DBFlowLog {
    fn from_db_storage(params: DBStorageParams) -> Result<Self, FlowError>
    where
        Self: Sized,
    {
        let start_system_time = FlowUtils::rfc3339_to_system_time(&params.start_time)?;
        let end_system_time = params
            .end_time
            .as_ref()
            .map(|t| FlowUtils::rfc3339_to_system_time(t))
            .transpose()?;

        let events: Vec<FlowEvent> = serde_json::from_str(&params.flow_data)?;
        let final_result: Option<ExecutionResult> = params
            .final_result
            .as_ref()
            .map(|r| serde_json::from_str(r))
            .transpose()?;

        let flow = FlowLog {
            session_id: params.session_id,
            benchmark_id: params.benchmark_id,
            agent_type: params.agent_type,
            start_time: start_system_time,
            end_time: end_system_time,
            events,
            final_result,
        };

        Ok(Self {
            flow,
            id: params.id,
            created_at: params.created_at,
        })
    }

    fn to_db_storage(&self) -> Result<DBStorageFormat, FlowError> {
        Ok(DBStorageFormat {
            session_id: self.flow.session_id.clone(),
            benchmark_id: self.flow.benchmark_id.clone(),
            agent_type: self.flow.agent_type.clone(),
            start_time: self.start_time()?,
            end_time: self.end_time()?,
            flow_data: self.events_json()?,
            final_result: self.final_result_json()?,
            id: self.id,
            created_at: self.created_at.clone(),
        })
    }
}

/// Database query parameters for filtering flow logs
#[derive(Debug, Clone, Default)]
pub struct FlowLogQuery {
    pub session_id: Option<String>,
    pub benchmark_id: Option<String>,
    pub agent_type: Option<String>,
    pub start_after: Option<String>,
    pub start_before: Option<String>,
    pub completed_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Database operations for flow logs
pub struct FlowLogDB;

impl FlowLogDB {
    /// Create a new flow log for database storage
    pub fn create(session_id: String, benchmark_id: String, agent_type: String) -> DBFlowLog {
        let flow = FlowUtils::create_flow_log(session_id, benchmark_id, agent_type);
        DBFlowLog::new(flow)
    }

    /// Add an event to a database flow log
    pub fn add_event(
        db_flow_log: &mut DBFlowLog,
        event_type: FlowEventType,
        depth: u32,
        data: serde_json::Value,
        metadata: HashMap<String, String>,
    ) -> Result<(), FlowError> {
        let event = FlowUtils::create_event(event_type, depth, data, metadata);
        FlowUtils::add_event(&mut db_flow_log.flow, event);
        Ok(())
    }

    /// Mark a flow log as completed
    pub fn mark_completed(
        db_flow_log: &mut DBFlowLog,
        result: ExecutionResult,
    ) -> Result<(), FlowError> {
        FlowUtils::mark_completed(&mut db_flow_log.flow, result);
        Ok(())
    }

    /// Generate summary from database flow log
    pub fn generate_summary(db_flow_log: &DBFlowLog) -> crate::utils::FlowSummary {
        FlowUtils::generate_summary(&db_flow_log.flow)
    }

    /// Convert query to SQL WHERE clause parameters
    pub fn query_to_sql_params(query: &FlowLogQuery) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(session_id) = &query.session_id {
            conditions.push("session_id = ?");
            params.push(session_id.clone());
        }

        if let Some(benchmark_id) = &query.benchmark_id {
            conditions.push("benchmark_id = ?");
            params.push(benchmark_id.clone());
        }

        if let Some(agent_type) = &query.agent_type {
            conditions.push("agent_type = ?");
            params.push(agent_type.clone());
        }

        if let Some(start_after) = &query.start_after {
            conditions.push("start_time > ?");
            params.push(start_after.clone());
        }

        if let Some(start_before) = &query.start_before {
            conditions.push("start_time < ?");
            params.push(start_before.clone());
        }

        if let Some(completed_only) = query.completed_only {
            if completed_only {
                conditions.push("end_time IS NOT NULL");
            } else {
                conditions.push("end_time IS NULL");
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_flow_log_creation() {
        let db_flow_log = FlowLogDB::create(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        assert_eq!(db_flow_log.session_id(), "session-123");
        assert_eq!(db_flow_log.benchmark_id(), "benchmark-456");
        assert_eq!(db_flow_log.agent_type(), "llm");
        assert!(!db_flow_log.is_completed());
        assert!(db_flow_log.id.is_none());
        assert!(db_flow_log.created_at.is_some());
    }

    #[test]
    fn test_db_flow_log_conversion() {
        let db_flow_log = FlowLogDB::create(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        let storage_format = db_flow_log.to_db_storage().unwrap();
        assert_eq!(storage_format.session_id, "session-123");
        assert_eq!(storage_format.benchmark_id, "benchmark-456");
        assert_eq!(storage_format.agent_type, "llm");

        let params = DBStorageParams {
            session_id: storage_format.session_id,
            benchmark_id: storage_format.benchmark_id,
            agent_type: storage_format.agent_type,
            start_time: storage_format.start_time,
            end_time: storage_format.end_time,
            flow_data: storage_format.flow_data,
            final_result: storage_format.final_result,
            id: storage_format.id,
            created_at: storage_format.created_at,
        };

        let converted_back = DBFlowLog::from_db_storage(params).unwrap();

        assert_eq!(converted_back.session_id(), db_flow_log.session_id());
        assert_eq!(converted_back.benchmark_id(), db_flow_log.benchmark_id());
        assert_eq!(converted_back.agent_type(), db_flow_log.agent_type());
    }

    #[test]
    fn test_query_to_sql_params() {
        let query = FlowLogQuery {
            session_id: Some("session-123".to_string()),
            agent_type: Some("llm".to_string()),
            completed_only: Some(true),
            ..Default::default()
        };

        let (where_clause, params) = FlowLogDB::query_to_sql_params(&query);
        assert!(where_clause.contains("session_id = ?"));
        assert!(where_clause.contains("agent_type = ?"));
        assert!(where_clause.contains("end_time IS NOT NULL"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_add_event_to_db_flow() {
        let mut db_flow_log = FlowLogDB::create(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        FlowLogDB::add_event(
            &mut db_flow_log,
            FlowEventType::LlmRequest,
            1,
            serde_json::json!({"test": "data"}),
            HashMap::new(),
        )
        .unwrap();

        assert_eq!(db_flow_log.flow.events.len(), 1);
    }

    #[test]
    fn test_mark_completed_db_flow() {
        let mut db_flow_log = FlowLogDB::create(
            "session-123".to_string(),
            "benchmark-456".to_string(),
            "llm".to_string(),
        );

        let result = ExecutionResult {
            success: true,
            score: 0.85,
            total_time_ms: 1000,
            statistics: ExecutionStatistics::default(),
            scoring_breakdown: None,
        };

        FlowLogDB::mark_completed(&mut db_flow_log, result).unwrap();
        assert!(db_flow_log.is_completed());
        assert!(db_flow_log.flow.final_result.is_some());
    }
}
