//! Tests for reev-flow database module

use reev_flow::{
    database::{DBFlowLog, DBStorageParams, FlowLogDB, FlowLogQuery},
    types::{ExecutionResult, ExecutionStatistics, FlowEventType},
    DBFlowLogConverter,
};

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
