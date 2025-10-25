//! Tests for reev-flow renderer module

use reev_flow::{
    renderer::FlowLogRenderer,
    types::{EventContent, FlowEvent, FlowEventType, FlowLog},
};
use std::time::SystemTime;

#[test]
fn test_render_basic_flow() {
    let flow = FlowLog {
        session_id: "test-session".to_string(),
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "test-agent".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![],
        final_result: None,
    };

    let rendered = flow.render_as_ascii_tree();
    assert!(rendered.contains("test-benchmark"));
    assert!(rendered.contains("test-agent"));
    assert!(rendered.contains("RUNNING"));
}

#[test]
fn test_render_flow_with_events() {
    let flow = FlowLog {
        session_id: "test-session".to_string(),
        benchmark_id: "test-benchmark".to_string(),
        agent_type: "test-agent".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now()),
        events: vec![FlowEvent {
            timestamp: SystemTime::now(),
            event_type: FlowEventType::LlmRequest,
            depth: 1,
            content: EventContent {
                data: serde_json::json!({
                    "model": "test-model",
                    "context_tokens": 100,
                    "prompt": "test prompt"
                }),
            },
        }],
        final_result: None,
    };

    let rendered = flow.render_as_ascii_tree();
    assert!(rendered.contains("LLM Request"));
    assert!(rendered.contains("test-model"));
    assert!(rendered.contains("100 tokens"));
}
