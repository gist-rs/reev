//! Conversion module for reev-lib flow types
//!
//! This module provides conversions between reev-lib domain-specific types
//! and the database-friendly types from reev-flow. Since the core types are
//! now shared, this mainly handles database-specific wrapping.

use crate::flow::types::*;
use reev_flow::database::{DBFlowLog, DBFlowLogConverter, DBStorageFormat};
use reev_flow::FlowError;

/// Converter for reev-lib FlowLog to database FlowLog
/// Extension trait for FlowLog to provide database conversion methods
pub trait FlowLogExt {
    /// Convert from database storage format
    fn from_db_storage(params: reev_flow::database::DBStorageParams) -> Result<Self, FlowError>
    where
        Self: Sized;

    /// Convert to database storage format
    fn to_db_storage(&self) -> Result<DBStorageFormat, FlowError>;
}

impl FlowLogExt for FlowLog {
    fn from_db_storage(params: reev_flow::database::DBStorageParams) -> Result<Self, FlowError>
    where
        Self: Sized,
    {
        // Convert from database storage format to reev-lib FlowLog
        let db_flow_log = DBFlowLog::from_db_storage(params)?;

        Ok(db_flow_log.flow)
    }

    fn to_db_storage(&self) -> Result<DBStorageFormat, FlowError> {
        // Wrap reev-lib FlowLog in DBFlowLog and convert to storage format
        let db_flow_log = DBFlowLog::new(self.clone());
        db_flow_log.to_db_storage()
    }
}

/// Extension trait to provide convenient conversion methods
pub trait FlowLogDbExt {
    /// Convert to database-friendly format
    fn to_db_flow_log(self) -> DBFlowLog;

    /// Convert from database-friendly format
    fn from_db_flow_log(db_flow_log: DBFlowLog) -> Self;
}

impl FlowLogDbExt for FlowLog {
    fn to_db_flow_log(self) -> DBFlowLog {
        DBFlowLog::new(self)
    }

    fn from_db_flow_log(db_flow_log: DBFlowLog) -> Self {
        db_flow_log.flow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_flow_log_db_conversion() {
        let reev_lib_flow = FlowLog {
            session_id: "test-session".to_string(),
            benchmark_id: "test-benchmark".to_string(),
            agent_type: "test-agent".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            events: vec![],
            final_result: None,
        };

        // Convert to database format
        let db_flow = reev_lib_flow.clone().to_db_flow_log();
        assert_eq!(db_flow.session_id(), reev_lib_flow.session_id);
        assert_eq!(db_flow.benchmark_id(), reev_lib_flow.benchmark_id);
        assert_eq!(db_flow.agent_type(), reev_lib_flow.agent_type);

        // Convert back to reev-lib format
        let converted_back = FlowLog::from_db_flow_log(db_flow);
        assert_eq!(converted_back.session_id, reev_lib_flow.session_id);
        assert_eq!(converted_back.benchmark_id, reev_lib_flow.benchmark_id);
        assert_eq!(converted_back.agent_type, reev_lib_flow.agent_type);
    }

    #[test]
    fn test_storage_format_conversion() {
        let flow = FlowLog {
            session_id: "session-123".to_string(),
            benchmark_id: "benchmark-456".to_string(),
            agent_type: "llm".to_string(),
            start_time: SystemTime::now(),
            end_time: Some(SystemTime::now()),
            events: vec![],
            final_result: None,
        };

        // Convert to storage format
        let storage = flow.to_db_storage().unwrap();
        assert_eq!(storage.session_id, flow.session_id);
        assert_eq!(storage.benchmark_id, flow.benchmark_id);
        assert_eq!(storage.agent_type, flow.agent_type);

        // Convert back from storage format
        let params = reev_flow::database::DBStorageParams {
            session_id: storage.session_id,
            benchmark_id: storage.benchmark_id,
            agent_type: storage.agent_type,
            start_time: storage.start_time,
            end_time: storage.end_time,
            flow_data: storage.flow_data,
            final_result: storage.final_result,
            id: storage.id,
            created_at: storage.created_at,
        };

        let restored = FlowLog::from_db_storage(params).unwrap();

        assert_eq!(restored.session_id, flow.session_id);
        assert_eq!(restored.benchmark_id, flow.benchmark_id);
        assert_eq!(restored.agent_type, flow.agent_type);
    }
}
