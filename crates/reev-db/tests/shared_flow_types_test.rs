//! Tests for shared flow types utilities

use reev_db::shared::flow::types::FlowLogUtils;
use reev_flow::{EventContent, FlowEvent, FlowEventType};
use std::collections::HashMap;

#[test]
fn test_flow_log_creation() {
    let flow_log = FlowLogUtils::create(
        "session-123".to_string(),
        "benchmark-456".to_string(),
        "llm".to_string(),
    );

    assert_eq!(flow_log.session_id(), "session-123");
    assert_eq!(flow_log.benchmark_id(), "benchmark-456");
    assert_eq!(flow_log.agent_type(), "llm");
    assert!(!flow_log.is_completed());
    assert!(flow_log.flow.final_result.is_none());
}

#[test]
fn test_event_serialization() {
    let event = FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: FlowEventType::LlmRequest,
        depth: 1,
        content: EventContent {
            data: serde_json::json!({"test": "data"}),
            metadata: HashMap::new(),
        },
    };

    let events = vec![event.clone()];
    let json = FlowLogUtils::serialize_events(&events).unwrap();
    let deserialized = FlowLogUtils::deserialize_events(&json).unwrap();

    assert_eq!(events.len(), deserialized.len());
    assert_eq!(events[0].depth, deserialized[0].depth);
}

#[test]
fn test_system_time_conversion() {
    let now = std::time::SystemTime::now();
    let rfc3339 = FlowLogUtils::system_time_to_rfc3339(now).unwrap();
    let converted_back = FlowLogUtils::rfc3339_to_system_time(&rfc3339).unwrap();

    // Allow small difference due to precision
    let diff = now
        .duration_since(converted_back)
        .unwrap_or_else(|_| converted_back.duration_since(now).unwrap());
    assert!(diff.as_millis() < 1000); // Less than 1 second difference
}
