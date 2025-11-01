use chrono::Utc;
use reev_flow::enhanced_otel::{
    EnhancedOtelLogger, EnhancedToolCall, EventType, TimingInfo, ToolInputInfo,
};

#[test]
fn test_enhanced_otel_logger_creation() {
    let logger = EnhancedOtelLogger::new();
    assert!(logger.is_ok());
}

#[test]
fn test_tool_call_logging() {
    let logger = EnhancedOtelLogger::new().unwrap();
    let tool_call = EnhancedToolCall {
        timestamp: Utc::now(),
        session_id: logger.session_id().to_string(),
        reev_runner_version: "0.1.0".to_string(),
        reev_agent_version: "0.1.0".to_string(),
        event_type: EventType::ToolInput,
        prompt: None,
        tool_input: Some(ToolInputInfo {
            tool_name: "test_tool".to_string(),
            tool_args: serde_json::json!({"param": "value"}),
        }),
        tool_output: None,
        timing: TimingInfo {
            flow_timeuse_ms: 100,
            step_timeuse_ms: 50,
        },
        metadata: serde_json::json!({}),
    };

    let result = logger.log_tool_call(tool_call);
    assert!(result.is_ok());
}

#[test]
fn test_session_summary() {
    let logger = EnhancedOtelLogger::new().unwrap();
    let result = logger.write_summary();
    assert!(result.is_ok());
}
