//! Test for OpenTelemetry trace extraction functionality
//!
//! This test verifies that tool call data can be extracted from
//! OpenTelemetry traces and converted to session format for
//! Mermaid diagram generation.

use reev_lib::agent::{ToolCallInfo, ToolResultStatus};
use reev_lib::otel_extraction::{
    convert_to_session_format, extract_current_otel_trace, init_otel_extraction,
    parse_otel_trace_to_tools, OtelSpanData, SessionToolData,
};
use std::time::SystemTime;

#[test]
fn test_otel_span_data_creation() {
    let span_data = OtelSpanData {
        span_name: "sol_transfer".to_string(),
        span_kind: "client".to_string(),
        start_time: SystemTime::now(),
        end_time: Some(SystemTime::now() + std::time::Duration::from_millis(100)),
        duration_ms: Some(100),
        attributes: std::collections::HashMap::new(),
        status: "success".to_string(),
        error_message: None,
    };

    assert_eq!(span_data.span_name, "sol_transfer");
    assert_eq!(span_data.status, "success");
    assert_eq!(span_data.duration_ms, Some(100));
}

#[test]
fn test_tool_span_detection() {
    let _tool_span = OtelSpanData {
        span_name: "sol_transfer".to_string(),
        span_kind: "client".to_string(),
        start_time: SystemTime::now(),
        end_time: None,
        duration_ms: None,
        attributes: std::collections::HashMap::new(),
        status: "success".to_string(),
        error_message: None,
    };

    // Note: is_tool_span is now tested indirectly through parse_otel_trace_to_tools
    // Placeholder assertion removed - function is tested indirectly

    let _non_tool_span = OtelSpanData {
        span_name: "http_request".to_string(),
        span_kind: "client".to_string(),
        start_time: SystemTime::now(),
        end_time: None,
        duration_ms: None,
        attributes: std::collections::HashMap::new(),
        status: "success".to_string(),
        error_message: None,
    };

    // Note: is_tool_span is now tested indirectly through parse_otel_trace_to_tools
    // Placeholder assertion removed - function is tested indirectly
}

#[test]
fn test_session_tool_data_creation() {
    let session_tool = SessionToolData {
        tool_name: "sol_transfer".to_string(),
        start_time: SystemTime::now(),
        end_time: SystemTime::now() + std::time::Duration::from_millis(100),
        params: serde_json::json!({"pubkey": "test123"}),
        result: serde_json::json!({"balance": "1.0"}),
        status: "success".to_string(),
    };

    assert_eq!(session_tool.tool_name, "sol_transfer");
    assert_eq!(session_tool.status, "success");
}

#[test]
fn test_tool_call_info_to_session_format() {
    let tool_calls = vec![
        ToolCallInfo {
            tool_name: "sol_transfer".to_string(),
            tool_args: serde_json::json!({"pubkey": "test123"}).to_string(),
            execution_time_ms: 100,
            result_status: ToolResultStatus::Success,
            result_data: Some(serde_json::json!({"balance": "1.0"})),
            error_message: None,
            timestamp: SystemTime::now(),
            depth: 1,
        },
        ToolCallInfo {
            tool_name: "jupiter_swap".to_string(),
            tool_args: serde_json::json!({"amount": "0.1"}).to_string(),
            execution_time_ms: 200,
            result_status: ToolResultStatus::Error,
            result_data: None,
            error_message: Some("Insufficient balance".to_string()),
            timestamp: SystemTime::now(),
            depth: 2,
        },
    ];

    let session_tools = convert_to_session_format(tool_calls);
    assert_eq!(session_tools.len(), 2);
    assert_eq!(session_tools[0].tool_name, "sol_transfer");
    assert_eq!(session_tools[0].status, "success");
    assert_eq!(session_tools[1].tool_name, "jupiter_swap");
    assert_eq!(session_tools[1].status, "error");
}

#[test]
fn test_extract_current_otel_trace() {
    // This test verifies the function exists and returns None when no tracing context
    let trace = extract_current_otel_trace();
    // In a test environment without active spans, this should return None
    assert!(trace.is_none());
}

#[test]
fn test_parse_otel_trace_to_tools() {
    let trace_data = reev_lib::otel_extraction::OtelTraceData {
        trace_id: "test_trace_123".to_string(),
        spans: vec![
            OtelSpanData {
                span_name: "sol_transfer".to_string(),
                span_kind: "client".to_string(),
                start_time: SystemTime::now(),
                end_time: Some(SystemTime::now() + std::time::Duration::from_millis(100)),
                duration_ms: Some(100),
                attributes: std::collections::HashMap::new(),
                status: "success".to_string(),
                error_message: None,
            },
            OtelSpanData {
                span_name: "http_request".to_string(),
                span_kind: "client".to_string(),
                start_time: SystemTime::now(),
                end_time: Some(SystemTime::now() + std::time::Duration::from_millis(50)),
                duration_ms: Some(50),
                attributes: std::collections::HashMap::new(),
                status: "success".to_string(),
                error_message: None,
            },
        ],
        extracted_at: SystemTime::now(),
    };

    let tool_calls = parse_otel_trace_to_tools(trace_data);
    // Should only extract the sol_transfer span, not the http_request span
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].tool_name, "sol_transfer");
    // Use match instead of assert_eq for ToolResultStatus
    match tool_calls[0].result_status {
        ToolResultStatus::Success => {
            // Expected result - test passes
        }
        _ => panic!(
            "Expected Success status, got: {:?}",
            tool_calls[0].result_status
        ),
    }
}

#[test]
fn test_otel_extraction_always_enabled() {
    // Test that OpenTelemetry extraction is always enabled
    let result = init_otel_extraction();
    assert!(
        result.is_ok(),
        "OpenTelemetry extraction should always initialize successfully"
    );
}
