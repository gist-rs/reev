use reev_lib::flow::{
    ErrorContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType, FlowLogger,
    LlmRequestContent, ToolCallContent, ToolResultStatus, WebsiteExporter,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;

#[test]
fn test_flow_logger_creation() {
    let temp_dir = TempDir::new().unwrap();
    let logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    assert_eq!(logger.benchmark_id, "test-benchmark");
    assert_eq!(logger.agent_type, "test-agent");
    assert_eq!(logger.events.len(), 0);
}

#[test]
fn test_llm_request_logging() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    let llm_request = LlmRequestContent {
        prompt: "Test prompt".to_string(),
        context_tokens: 100,
        model: "test-model".to_string(),
        request_id: "req-123".to_string(),
    };

    logger.log_llm_request(llm_request, 1);

    assert_eq!(logger.events.len(), 1);
    let event = &logger.events[0];
    assert!(matches!(event.event_type, FlowEventType::LlmRequest));
    assert_eq!(event.depth, 1);
}

#[test]
fn test_tool_call_logging() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    let tool_call = ToolCallContent {
        tool_name: "jupiter_swap".to_string(),
        tool_args: "{\"input_token\": \"USDC\"}".to_string(),
        execution_time_ms: 1500,
        result_status: ToolResultStatus::Success,
        result_data: Some(serde_json::json!({"result": "success"})),
        error_message: None,
    };

    logger.log_tool_call(tool_call.clone(), 2);
    logger.log_tool_result(tool_call, 2);

    assert_eq!(logger.events.len(), 2);

    // Check tool call event
    let call_event = &logger.events[0];
    assert!(matches!(call_event.event_type, FlowEventType::ToolCall));
    assert_eq!(call_event.depth, 2);

    // Check tool result event
    let result_event = &logger.events[1];
    assert!(matches!(result_event.event_type, FlowEventType::ToolResult));
    assert_eq!(result_event.depth, 2);
}

#[test]
fn test_error_logging() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    let error_content = ErrorContent {
        error_type: "TestError".to_string(),
        message: "Test error message".to_string(),
        stack_trace: Some("stack trace".to_string()),
        context: {
            let mut ctx = HashMap::new();
            ctx.insert("key".to_string(), "value".to_string());
            ctx
        },
    };

    logger.log_error(error_content, 3);

    assert_eq!(logger.events.len(), 1);
    let event = &logger.events[0];
    assert!(matches!(event.event_type, FlowEventType::Error));
    assert_eq!(event.depth, 3);
}

#[test]
fn test_statistics_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Log an LLM request
    let llm_request = LlmRequestContent {
        prompt: "Test prompt".to_string(),
        context_tokens: 200,
        model: "test-model".to_string(),
        request_id: "req-123".to_string(),
    };
    logger.log_llm_request(llm_request, 1);

    // Log multiple tool calls
    for i in 0..3 {
        let tool_call = ToolCallContent {
            tool_name: if i == 0 {
                "jupiter_swap"
            } else {
                "jupiter_lend"
            }
            .to_string(),
            tool_args: "{}".to_string(),
            execution_time_ms: 1000,
            result_status: ToolResultStatus::Success,
            result_data: None,
            error_message: None,
        };
        logger.log_tool_call(tool_call, 2);
    }

    let stats = logger.get_current_statistics();
    assert_eq!(stats.total_llm_calls, 1);
    assert_eq!(stats.total_tool_calls, 3);
    assert_eq!(stats.total_tokens, 200);
    assert_eq!(stats.max_depth, 2);
    assert_eq!(stats.tool_usage.get("jupiter_swap"), Some(&1));
    assert_eq!(stats.tool_usage.get("jupiter_lend"), Some(&2));
}

#[test]
fn test_flow_completion() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Add some events
    let llm_request = LlmRequestContent {
        prompt: "Test prompt".to_string(),
        context_tokens: 100,
        model: "test-model".to_string(),
        request_id: "req-123".to_string(),
    };
    logger.log_llm_request(llm_request, 1);

    let execution_result = ExecutionResult {
        success: true,
        score: 0.85,
        total_time_ms: 5000,
        statistics: ExecutionStatistics {
            total_llm_calls: 1,
            total_tool_calls: 0,
            total_tokens: 100,
            tool_usage: HashMap::new(),
            max_depth: 1,
        },
    };

    // Complete the flow log
    logger.complete(execution_result).unwrap();

    // Check that YML file was created
    let mut files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();

    assert_eq!(files.len(), 1);
    assert!(files[0].starts_with("flow_test-benchmark_test-agent_"));
    assert!(files[0].ends_with(".yml"));

    // Verify YML content
    let yml_content = std::fs::read_to_string(temp_dir.path().join(&files[0])).unwrap();
    assert!(yml_content.contains("session_id:"));
    assert!(yml_content.contains("benchmark_id: test-benchmark"));
    assert!(yml_content.contains("agent_type: test-agent"));
    assert!(yml_content.contains("success: true"));
    assert!(yml_content.contains("score: 0.85"));
}

#[test]
fn test_website_export() {
    let temp_dir = TempDir::new().unwrap();
    let exporter = WebsiteExporter::new(temp_dir.path().to_path_buf());

    // Create mock flow data
    let flows = vec![]; // Empty for this test

    let result = exporter.export_for_website(&flows);
    assert!(result.is_ok());

    // Check that website data file was created
    let website_file = temp_dir.path().join("website_data.json");
    assert!(website_file.exists());

    // Verify JSON content
    let json_content = std::fs::read_to_string(&website_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    assert!(parsed.get("flows").is_some());
    assert!(parsed.get("flow_visualization").is_some());
    assert!(parsed.get("tool_usage_stats").is_some());
    assert!(parsed.get("performance_metrics").is_some());
    assert!(parsed.get("agent_behavior_analysis").is_some());
}

#[test]
fn test_serialization_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Log various events
    let llm_request = LlmRequestContent {
        prompt: "Test prompt".to_string(),
        context_tokens: 150,
        model: "test-model".to_string(),
        request_id: "req-456".to_string(),
    };
    logger.log_llm_request(llm_request, 1);

    let tool_call = ToolCallContent {
        tool_name: "test_tool".to_string(),
        tool_args: "{\"arg\": \"value\"}".to_string(),
        execution_time_ms: 2000,
        result_status: ToolResultStatus::Success,
        result_data: Some(serde_json::json!({"output": "success"})),
        error_message: None,
    };
    logger.log_tool_call(tool_call, 2);

    let execution_result = ExecutionResult {
        success: true,
        score: 0.9,
        total_time_ms: 8000,
        statistics: logger.get_current_statistics(),
    };

    // Complete and save
    logger.complete(execution_result).unwrap();

    // Read and deserialize back
    let files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();

    let yml_content = std::fs::read_to_string(temp_dir.path().join(&files[0])).unwrap();
    let deserialized: reev_lib::flow::FlowLog = serde_yaml::from_str(&yml_content).unwrap();

    assert_eq!(deserialized.benchmark_id, "test-benchmark");
    assert_eq!(deserialized.agent_type, "test-agent");
    assert_eq!(deserialized.events.len(), 2);
    assert!(deserialized.final_result.is_some());
    assert_eq!(deserialized.final_result.unwrap().score, 0.9);
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        PathBuf::from("/nonexistent/path"), // Invalid path
    );

    let execution_result = ExecutionResult {
        success: false,
        score: 0.0,
        total_time_ms: 0,
        statistics: ExecutionStatistics {
            total_llm_calls: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            tool_usage: HashMap::new(),
            max_depth: 0,
        },
    };

    // Should handle error gracefully
    let result = logger.complete(execution_result);
    assert!(result.is_err());
}

#[test]
fn test_multiple_flow_events() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "multi-step-test".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Simulate a complex flow with multiple steps
    for step in 1..=5 {
        // LLM request
        let llm_request = LlmRequestContent {
            prompt: format!("Step {} prompt", step),
            context_tokens: 100 * step,
            model: "test-model".to_string(),
            request_id: format!("req-{}", step),
        };
        logger.log_llm_request(llm_request, step);

        // Tool call
        let tool_call = ToolCallContent {
            tool_name: format!("tool_{}", step),
            tool_args: format!("{{\"step\": {}}}", step),
            execution_time_ms: 500 * step,
            result_status: ToolResultStatus::Success,
            result_data: Some(serde_json::json!({"step_result": step})),
            error_message: None,
        };
        logger.log_tool_call(tool_call, step);
    }

    assert_eq!(logger.events.len(), 10);

    let stats = logger.get_current_statistics();
    assert_eq!(stats.total_llm_calls, 5);
    assert_eq!(stats.total_tool_calls, 5);
    assert_eq!(stats.total_tokens, 1500); // Sum of 100, 200, 300, 400, 500
    assert_eq!(stats.max_depth, 5);
    assert_eq!(stats.tool_usage.len(), 5);

    // Complete the flow
    let execution_result = ExecutionResult {
        success: true,
        score: 1.0,
        total_time_ms: 15000,
        statistics: stats,
    };

    logger.complete(execution_result).unwrap();

    // Verify all events are in the YML
    let files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();

    let yml_content = std::fs::read_to_string(temp_dir.path().join(&files[0])).unwrap();
    assert_eq!(yml_content.matches("event_type:").count(), 10); // 10 events total
}
