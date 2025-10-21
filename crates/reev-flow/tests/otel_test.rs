//! Tests for reev-flow OpenTelemetry module

use reev_flow::{
    otel::FlowTracer,
    types::{EventContent, FlowEvent, FlowEventType, LlmRequestContent},
};
use serial_test::serial;
use std::time::SystemTime;

#[test]
#[serial]
fn test_flow_tracer_creation() {
    // OpenTelemetry is always enabled
    let tracer = FlowTracer::new();
    assert!(tracer.is_enabled());
}

#[test]
#[serial]
fn test_flow_tracing_disabled() {
    // OpenTelemetry is always enabled
    let tracer = FlowTracer::new();

    let event = FlowEvent {
        timestamp: SystemTime::now(),
        event_type: FlowEventType::LlmRequest,
        depth: 1,
        content: EventContent {
            data: serde_json::to_value(LlmRequestContent {
                prompt: "test".to_string(),
                context_tokens: 100,
                model: "test-model".to_string(),
                request_id: "test-123".to_string(),
            })
            .unwrap(),
            metadata: std::collections::HashMap::new(),
        },
    };

    // Should not panic when disabled
    tracer.trace_flow_event(&event);
    // No cleanup needed - OpenTelemetry is always enabled
}

#[test]
#[serial]
fn test_flow_tracing_enabled() {
    // OpenTelemetry is always enabled
    let tracer = FlowTracer::new();
    assert!(tracer.is_enabled());

    let event = FlowEvent {
        timestamp: SystemTime::now(),
        event_type: FlowEventType::ToolCall,
        depth: 2,
        content: EventContent {
            data: serde_json::json!({
                "tool_name": "test_tool",
                "execution_time_ms": 1500
            }),
            metadata: std::collections::HashMap::new(),
        },
    };

    // Should not panic when enabled
    tracer.trace_flow_event(&event);
    // No cleanup needed - OpenTelemetry is always enabled
}
