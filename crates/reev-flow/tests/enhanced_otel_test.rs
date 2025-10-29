//! Enhanced OpenTelemetry logging tests
//!
//! Tests for the enhanced JSONL logging system with structured tool call tracking.

use reev_flow::{
    init_enhanced_otel_logging, init_enhanced_otel_logging_with_session, EnhancedOtelLogger,
    EnhancedToolCall, EventType, TimingInfo, ToolInputInfo,
};
use serde_json::json;
use std::thread;
use std::time::Duration;

#[tokio::test]
async fn test_enhanced_otel_logging_integration() {
    // Initialize enhanced logging (or get existing if already initialized)
    let session_id = match init_enhanced_otel_logging() {
        Ok(id) => id,
        Err(reev_flow::EnhancedOtelError::Mutex(_)) => {
            // Logger already initialized, get existing session
            let logger =
                reev_flow::get_enhanced_otel_logger().expect("Failed to get existing logger");
            logger.session_id().to_string()
        }
        Err(e) => panic!("Failed to initialize enhanced otel logging: {e}"),
    };

    // Verify logger is accessible
    let logger = reev_flow::get_enhanced_otel_logger().expect("Failed to get enhanced otel logger");

    assert_eq!(logger.session_id(), session_id);

    // Test tool call logging macro
    let test_args = json!({
        "user_pubkey": "test_pubkey",
        "amount": 1000000,
        "mint": "So11111111111111111111111111111111111111112"
    });

    reev_flow::log_tool_call!("test_tool", &test_args);

    // Allow some time for logging
    thread::sleep(Duration::from_millis(10));

    // Test tool completion logging macro
    let test_result = json!({
        "success": true,
        "transaction_id": "test_tx_id"
    });

    reev_flow::log_tool_completion!("test_tool", 150, &test_result, true);

    // Allow some time for logging
    thread::sleep(Duration::from_millis(10));

    // Verify tool calls were logged
    let tool_calls = logger.get_tool_calls().expect("Failed to get tool calls");
    assert!(!tool_calls.is_empty(), "No tool calls were logged");

    // Verify tool input event was logged
    let tool_input_events: Vec<_> = tool_calls
        .iter()
        .filter(|call| matches!(call.event_type, EventType::ToolInput))
        .collect();
    assert!(
        !tool_input_events.is_empty(),
        "No tool input events were logged"
    );

    // Verify tool output event was logged
    let tool_output_events: Vec<_> = tool_calls
        .iter()
        .filter(|call| matches!(call.event_type, EventType::ToolOutput))
        .collect();
    assert!(
        !tool_output_events.is_empty(),
        "No tool output events were logged"
    );

    // Verify tool input contains expected data
    if let Some(tool_input) = tool_input_events.first() {
        assert_eq!(tool_input.session_id, session_id);
        if let Some(input_info) = &tool_input.tool_input {
            assert_eq!(input_info.tool_name, "test_tool");
            assert!(input_info.tool_args.get("user_pubkey").is_some());
        }
    }

    // Verify tool output contains expected data
    if let Some(tool_output) = tool_output_events.first() {
        assert_eq!(tool_output.session_id, session_id);
        if let Some(output_info) = &tool_output.tool_output {
            assert!(output_info.success);
            assert_eq!(
                output_info
                    .results
                    .get("execution_time_ms")
                    .and_then(|v| v.as_u64()),
                Some(150)
            );
        }
    }
}

#[tokio::test]
async fn test_enhanced_otel_prompt_logging() {
    // Initialize enhanced logging with custom session ID
    let session_id = "test_session_prompt".to_string();
    match init_enhanced_otel_logging_with_session(session_id.clone()) {
        Ok(_) => {} // Successfully initialized
        Err(reev_flow::EnhancedOtelError::Mutex(_)) => {
            // Logger already initialized, use existing one
        }
        Err(e) => panic!("Failed to initialize enhanced otel logging with session: {e}"),
    }

    // Test prompt event logging macro
    let tool_names = vec!["sol_transfer".to_string(), "jupiter_swap".to_string()];
    let user_prompt = "Send 1 SOL to test address".to_string();
    let final_prompt = "Send 1 SOL to test address using sol_transfer tool".to_string();

    reev_flow::log_prompt_event!(&tool_names, &user_prompt, &final_prompt);

    // Allow some time for logging
    thread::sleep(Duration::from_millis(10));

    // Verify prompt event was logged
    let logger = reev_flow::get_enhanced_otel_logger().expect("Failed to get enhanced otel logger");

    let tool_calls = logger.get_tool_calls().expect("Failed to get tool calls");

    // Find prompt event
    let prompt_events: Vec<_> = tool_calls
        .iter()
        .filter(|call| matches!(call.event_type, EventType::Prompt))
        .collect();

    assert!(!prompt_events.is_empty(), "No prompt events were logged");

    // Verify prompt event contains expected data
    if let Some(prompt_event) = prompt_events.first() {
        assert_eq!(prompt_event.session_id, session_id);
        if let Some(prompt_info) = &prompt_event.prompt {
            assert_eq!(prompt_info.tool_name_list, tool_names);
            assert_eq!(prompt_info.user_prompt, user_prompt);
            assert_eq!(prompt_info.final_prompt, final_prompt);
        }
    }
}

#[tokio::test]
async fn test_enhanced_otel_step_completion_logging() {
    // Initialize enhanced logging with custom session ID
    let session_id = "test_session_step".to_string();
    match init_enhanced_otel_logging_with_session(session_id.clone()) {
        Ok(_) => {} // Successfully initialized
        Err(reev_flow::EnhancedOtelError::Mutex(_)) => {
            // Logger already initialized, use existing one
        }
        Err(e) => panic!("Failed to initialize enhanced otel logging with session: {e}"),
    }

    // Test step completion logging macro
    let step_name = "test_step".to_string();
    let flow_time_ms = 5000;
    let step_time_ms = 1500;

    reev_flow::log_step_complete!(&step_name, flow_time_ms, step_time_ms);

    // Allow some time for logging
    thread::sleep(Duration::from_millis(10));

    // Verify step completion event was logged
    let logger = reev_flow::get_enhanced_otel_logger().expect("Failed to get enhanced otel logger");

    let tool_calls = logger.get_tool_calls().expect("Failed to get tool calls");

    // Find step completion event
    let step_events: Vec<_> = tool_calls
        .iter()
        .filter(|call| matches!(call.event_type, EventType::StepComplete))
        .collect();

    assert!(
        !step_events.is_empty(),
        "No step completion events were logged"
    );

    // Verify step event contains expected data
    if let Some(step_event) = step_events.first() {
        assert_eq!(step_event.session_id, session_id);
        assert_eq!(step_event.timing.flow_timeuse_ms, flow_time_ms);
        assert_eq!(step_event.timing.step_timeuse_ms, step_time_ms);
        assert_eq!(
            step_event
                .metadata
                .get("step_name")
                .and_then(|v| v.as_str()),
            Some(step_name.as_str())
        );
    }
}

#[tokio::test]
async fn test_enhanced_otel_logger_direct_usage() {
    // Test direct usage of EnhancedOtelLogger
    let logger = EnhancedOtelLogger::new().expect("Failed to create logger");
    let session_id = logger.session_id().to_string();

    // Create and log a tool call directly
    let tool_call = EnhancedToolCall {
        timestamp: chrono::Utc::now(),
        session_id: session_id.clone(),
        reev_runner_version: "0.1.0-test".to_string(),
        reev_agent_version: "0.1.0-test".to_string(),
        event_type: EventType::ToolInput,
        prompt: None,
        tool_input: Some(ToolInputInfo {
            tool_name: "direct_test_tool".to_string(),
            tool_args: json!({"test": "data"}),
        }),
        tool_output: None,
        timing: TimingInfo {
            flow_timeuse_ms: 0,
            step_timeuse_ms: 0,
        },
        metadata: json!({"test": "direct_usage"}),
    };

    let result = logger.log_tool_call(tool_call);
    assert!(result.is_ok(), "Failed to log tool call directly");

    // Verify the call was stored
    let stored_calls = logger.get_tool_calls().expect("Failed to get stored calls");
    assert!(!stored_calls.is_empty(), "No calls were stored");

    // Verify the stored call matches our logged call
    let stored_call = stored_calls.last().expect("No stored call found");
    assert_eq!(stored_call.session_id, session_id);
    if let Some(input) = &stored_call.tool_input {
        assert_eq!(input.tool_name, "direct_test_tool");
    }
}

#[test]
fn test_enhanced_otel_disabled() {
    // Test that macros work when enhanced otel is disabled
    std::env::set_var("REEV_ENHANCED_OTEL", "0");

    // These should not panic or cause issues even when disabled
    reev_flow::log_tool_call!("disabled_test", &json!({"test": "data"}));
    reev_flow::log_tool_completion!("disabled_test", 100, &json!({"result": "ok"}), true);
    reev_flow::log_prompt_event!(&["tool1".to_string()], "test prompt", "test final prompt");
    reev_flow::log_step_complete!("disabled_step", 1000, 500);

    // Reset environment
    std::env::set_var("REEV_ENHANCED_OTEL", "1");
}
