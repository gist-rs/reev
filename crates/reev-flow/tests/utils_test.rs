//! Tests for reev-flow utils module

use reev_flow::{
    types::{ExecutionResult, ExecutionStatistics, FlowEventType, ToolResultStatus},
    utils::FlowUtils,
};
use std::collections::HashMap;

#[test]
fn test_flow_log_creation() {
    let flow_log = FlowUtils::create_flow_log(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    assert_eq!(flow_log.session_id, "session-123");
    assert_eq!(flow_log.benchmark_id, "benchmark-456");
    assert_eq!(flow_log.agent_type, "llm");
    assert!(flow_log.end_time.is_none());
    assert!(flow_log.final_result.is_none());
    assert!(flow_log.events.is_empty());
}

#[test]
fn test_add_event() {
    let mut flow_log = FlowUtils::create_flow_log(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    let event = FlowUtils::create_event(
        FlowEventType::LlmRequest,
        1,
        serde_json::json!({"test": "data"}),
        HashMap::new(),
    );

    FlowUtils::add_event(&mut flow_log, event);
    assert_eq!(flow_log.events.len(), 1);
}

#[test]
fn test_llm_event_creation() {
    let event = FlowUtils::create_llm_event(
        1,
        "test prompt".to_string(),
        100,
        "qwen3-coder-30b-a3b-instruct-mlx".to_string(),
        "req-123".to_string(),
    );

    assert!(matches!(event.event_type, FlowEventType::LlmRequest));
    assert_eq!(event.depth, 1);
}

#[test]
fn test_tool_event_creation() {
    let event = FlowUtils::create_tool_event(
        2,
        "test_tool".to_string(),
        "{}".to_string(),
        100,
        ToolResultStatus::Success,
        Some(serde_json::json!({"result": "ok"})),
        None,
    );

    assert!(matches!(event.event_type, FlowEventType::ToolCall));
    assert_eq!(event.depth, 2);
}

#[test]
fn test_event_counting() {
    let mut flow_log = FlowUtils::create_flow_log(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    let llm_event = FlowUtils::create_llm_event(
        1,
        "test prompt".to_string(),
        100,
        "qwen3-coder-30b-a3b-instruct-mlx".to_string(),
        "req-123".to_string(),
    );

    let tool_event = FlowUtils::create_tool_event(
        2,
        "test_tool".to_string(),
        "{}".to_string(),
        100,
        ToolResultStatus::Success,
        None,
        None,
    );

    FlowUtils::add_event(&mut flow_log, llm_event);
    FlowUtils::add_event(&mut flow_log, tool_event);

    let counts = FlowUtils::count_events_by_type(&flow_log);
    assert_eq!(counts.get("LlmRequest"), Some(&1));
    assert_eq!(counts.get("ToolCall"), Some(&1));
}

#[test]
fn test_flow_summary() {
    let mut flow_log = FlowUtils::create_flow_log(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    let event = FlowUtils::create_llm_event(
        1,
        "test prompt".to_string(),
        100,
        "qwen3-coder-30b-a3b-instruct-mlx".to_string(),
        "req-123".to_string(),
    );

    FlowUtils::add_event(&mut flow_log, event);

    let result = ExecutionResult {
        success: true,
        score: 0.85,
        total_time_ms: 1000,
        statistics: ExecutionStatistics::default(),
        scoring_breakdown: None,
    };

    FlowUtils::mark_completed(&mut flow_log, result);

    let summary = FlowUtils::generate_summary(&flow_log);
    assert_eq!(summary.session_id, "session-123");
    assert_eq!(summary.total_events, 1);
    assert!(summary.success);
    assert_eq!(summary.final_score, Some(0.85));
}

#[test]
fn test_llm_event_creation_with_response() {
    let event = FlowUtils::create_llm_event(
        1,
        "test prompt".to_string(),
        100,
        "qwen3-coder-30b-a3b-instruct-mlx".to_string(),
        "req-123".to_string(),
    );

    assert!(matches!(event.event_type, FlowEventType::LlmRequest));
    assert_eq!(event.depth, 1);
}

#[test]
fn test_flow_completion() {
    let mut flow_log = FlowUtils::create_flow_log(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    let result = ExecutionResult {
        success: false,
        score: 0.3,
        total_time_ms: 2000,
        statistics: ExecutionStatistics {
            total_llm_calls: 5,
            total_tool_calls: 3,
            total_tokens: 1000,
            tool_usage: HashMap::new(),
            max_depth: 3,
        },
        scoring_breakdown: None,
    };

    FlowUtils::mark_completed(&mut flow_log, result);
    assert!(flow_log.final_result.is_some());
    assert!(flow_log.end_time.is_some());
}
