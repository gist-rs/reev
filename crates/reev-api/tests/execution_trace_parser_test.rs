use reev_api::handlers::parsers::execution_trace::ExecutionTraceParser;
use reev_types::ExecutionState;
use reev_types::ExecutionStatus;
use serde_json::json;

#[tokio::test]
async fn test_generate_trace_from_session_data() {
    let parser = ExecutionTraceParser::new();

    let session_data = json!({
        "session_id": "test-123",
        "benchmark_id": "001-sol-transfer",
        "agent_type": "deterministic",
        "success": true,
        "score": 0.95,
        "steps": [
            {
                "action": [{"program_id": "test123"}],
                "observation": {"last_transaction_status": "success"}
            }
        ]
    });

    let result = parser
        .generate_trace_from_session_data(&session_data, "test-123")
        .await;

    assert!(result.is_ok());
    let trace = result.unwrap();
    assert!(trace.contains("🌊"));
    assert!(trace.contains("001-sol-transfer"));
    assert!(trace.contains("deterministic"));
}

#[tokio::test]
async fn test_generate_error_trace() {
    let parser = ExecutionTraceParser::new();

    let error_trace = parser.generate_error_trace("Test error", "exec-123");
    assert!(error_trace.contains("⚠️"));
    assert!(error_trace.contains("exec-123"));
    assert!(error_trace.contains("Test error"));
}

#[tokio::test]
async fn test_generate_trace_from_metadata() {
    let parser = ExecutionTraceParser::new();

    let mut state = ExecutionState::new(
        "test-exec".to_string(),
        "test-benchmark".to_string(),
        "test-agent".to_string(),
    );
    state.update_status(ExecutionStatus::Failed);
    state.set_error("Test error".to_string());

    let result = parser.generate_trace_from_metadata(&state).await;
    assert!(result.is_ok());

    let trace = result.unwrap();
    assert!(trace.contains("test-exec"));
    assert!(trace.contains("Failed"));
    assert!(trace.contains("Test error"));
}
