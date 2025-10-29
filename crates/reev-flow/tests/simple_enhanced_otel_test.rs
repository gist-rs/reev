//! Simple Enhanced OpenTelemetry verification test
//!
//! Basic test to verify enhanced OpenTelemetry logging is working.

use reev_flow::{EnhancedOtelLogger, EventType, ToolInputInfo, ToolOutputInfo};
use serde_json::json;

#[test]
fn test_enhanced_otel_logger_creation() {
    // Test basic logger creation
    let logger = EnhancedOtelLogger::new();
    assert!(logger.is_ok(), "Failed to create enhanced otel logger");
}

#[test]
fn test_enhanced_otel_tool_call_logging() {
    // Test direct tool call logging
    let logger = EnhancedOtelLogger::new().expect("Failed to create logger");
    let session_id = logger.session_id().to_string();

    // Create a tool input event
    let tool_call = reev_flow::EnhancedToolCall {
        timestamp: chrono::Utc::now(),
        session_id: session_id.clone(),
        reev_runner_version: "0.1.0-test".to_string(),
        reev_agent_version: "0.1.0-test".to_string(),
        event_type: EventType::ToolInput,
        prompt: None,
        tool_input: Some(ToolInputInfo {
            tool_name: "test_tool".to_string(),
            tool_args: json!({"test": "data"}),
        }),
        tool_output: None,
        timing: reev_flow::TimingInfo {
            flow_timeuse_ms: 0,
            step_timeuse_ms: 0,
        },
        metadata: json!({"test": "direct_usage"}),
    };

    // Log the tool call
    let result = logger.log_tool_call(tool_call);
    assert!(result.is_ok(), "Failed to log tool call directly");

    // Verify the call was stored
    let stored_calls = logger.get_tool_calls().expect("Failed to get stored calls");
    assert!(!stored_calls.is_empty(), "No calls were stored");

    // Verify the stored call matches our expectations
    let stored_call = stored_calls.last().expect("No stored call found");
    assert_eq!(stored_call.session_id, session_id);
    assert!(matches!(stored_call.event_type, EventType::ToolInput));

    if let Some(input) = &stored_call.tool_input {
        assert_eq!(input.tool_name, "test_tool");
        assert!(input.tool_args.get("test").is_some());
    }
}

#[test]
fn test_enhanced_otel_full_workflow() {
    // Test a complete workflow with input and output events
    let logger = EnhancedOtelLogger::new().expect("Failed to create logger");
    let session_id = logger.session_id().to_string();

    // Step 1: Log tool input
    let input_call = reev_flow::EnhancedToolCall {
        timestamp: chrono::Utc::now(),
        session_id: session_id.clone(),
        reev_runner_version: "0.1.0".to_string(),
        reev_agent_version: "0.1.0".to_string(),
        event_type: EventType::ToolInput,
        prompt: None,
        tool_input: Some(ToolInputInfo {
            tool_name: "workflow_tool".to_string(),
            tool_args: json!({"action": "test", "value": 42}),
        }),
        tool_output: None,
        timing: reev_flow::TimingInfo {
            flow_timeuse_ms: 0,
            step_timeuse_ms: 0,
        },
        metadata: json!({}),
    };

    let result = logger.log_tool_call(input_call);
    assert!(result.is_ok(), "Failed to log tool input");

    // Step 2: Log tool output
    let output_call = reev_flow::EnhancedToolCall {
        timestamp: chrono::Utc::now(),
        session_id: session_id.clone(),
        reev_runner_version: "0.1.0".to_string(),
        reev_agent_version: "0.1.0".to_string(),
        event_type: EventType::ToolOutput,
        prompt: None,
        tool_input: None,
        tool_output: Some(ToolOutputInfo {
            success: true,
            results: json!({"result": "success", "processed": true}),
            error_message: None,
        }),
        timing: reev_flow::TimingInfo {
            flow_timeuse_ms: 0,
            step_timeuse_ms: 250,
        },
        metadata: json!({"step_complete": true}),
    };

    let result = logger.log_tool_call(output_call);
    assert!(result.is_ok(), "Failed to log tool output");

    // Verify both events were stored
    let stored_calls = logger.get_tool_calls().expect("Failed to get stored calls");
    assert_eq!(
        stored_calls.len(),
        2,
        "Expected 2 calls, got {}",
        stored_calls.len()
    );

    // Verify input event
    let input_event = &stored_calls[0];
    assert_eq!(input_event.session_id, session_id);
    assert!(matches!(input_event.event_type, EventType::ToolInput));
    assert!(input_event.tool_input.is_some());
    assert!(input_event.tool_output.is_none());

    // Verify output event
    let output_event = &stored_calls[1];
    assert_eq!(output_event.session_id, session_id);
    assert!(matches!(output_event.event_type, EventType::ToolOutput));
    assert!(output_event.tool_input.is_none());
    assert!(output_event.tool_output.is_some());

    if let Some(output) = &output_event.tool_output {
        assert!(output.success);
    }
    // Timing is on the tool call, not the output
    assert_eq!(output_event.timing.step_timeuse_ms, 250);
}

#[test]
fn test_enhanced_otel_summary_generation() {
    // Test summary generation
    let logger = EnhancedOtelLogger::new().expect("Failed to create logger");

    // Log a few events
    for i in 0..3 {
        let call = reev_flow::EnhancedToolCall {
            timestamp: chrono::Utc::now(),
            session_id: logger.session_id().to_string(),
            reev_runner_version: "0.1.0".to_string(),
            reev_agent_version: "0.1.0".to_string(),
            event_type: EventType::ToolInput,
            prompt: None,
            tool_input: Some(ToolInputInfo {
                tool_name: format!("test_tool_{i}"),
                tool_args: json!({"index": i}),
            }),
            tool_output: None,
            timing: reev_flow::TimingInfo {
                flow_timeuse_ms: 0,
                step_timeuse_ms: 0,
            },
            metadata: json!({}),
        };

        logger.log_tool_call(call).expect("Failed to log call");
    }

    // Test summary generation (this is called automatically on Drop)
    let result = logger.write_summary();
    assert!(result.is_ok(), "Failed to write summary");
}
