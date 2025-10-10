use reev_lib::flow::{
    ErrorContent, ExecutionResult, ExecutionStatistics, FlowEvent, FlowEventType, FlowLogger,
    LlmRequestContent, ToolCallContent, ToolResultStatus,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;

#[test]
fn test_basic_flow_logging() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Log LLM request
    let llm_request = LlmRequestContent {
        prompt: "Test prompt".to_string(),
        context_tokens: 100,
        model: "test-model".to_string(),
        request_id: "req-123".to_string(),
    };
    logger.log_llm_request(llm_request, 1);

    // Log tool call
    let tool_call = ToolCallContent {
        tool_name: "jupiter_swap".to_string(),
        tool_args: "{\"input_token\": \"USDC\"}".to_string(),
        execution_time_ms: 1500,
        result_status: ToolResultStatus::Success,
        result_data: Some(serde_json::json!({"result": "success"})),
        error_message: None,
    };
    logger.log_tool_call(tool_call, 2);

    // Complete the flow
    let execution_result = ExecutionResult {
        success: true,
        score: 0.85,
        total_time_ms: 5000,
        statistics: ExecutionStatistics {
            total_llm_calls: 1,
            total_tool_calls: 1,
            total_tokens: 100,
            tool_usage: HashMap::from([("jupiter_swap".to_string(), 1)]),
            max_depth: 2,
        },
    };

    logger.complete(execution_result).unwrap();

    // Verify YML file was created
    let files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();

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
    assert!(yml_content.contains("llm_request"));
    assert!(yml_content.contains("tool_call"));
}

#[test]
fn test_statistics_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "stats-test".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    // Log multiple events
    for i in 0..3 {
        let llm_request = LlmRequestContent {
            prompt: format!("Prompt {}", i),
            context_tokens: 100 * (i + 1),
            model: "test-model".to_string(),
            request_id: format!("req-{}", i),
        };
        logger.log_llm_request(llm_request, i + 1);

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
        logger.log_tool_call(tool_call, i + 1);
    }

    let stats = logger.get_current_statistics();
    assert_eq!(stats.total_llm_calls, 3);
    assert_eq!(stats.total_tool_calls, 3);
    assert_eq!(stats.total_tokens, 600); // 100 + 200 + 300
    assert_eq!(stats.max_depth, 3);
    assert_eq!(stats.tool_usage.get("jupiter_swap"), Some(&1));
    assert_eq!(stats.tool_usage.get("jupiter_lend"), Some(&2));
}

#[test]
fn test_error_logging() {
    let temp_dir = TempDir::new().unwrap();
    let mut logger = FlowLogger::new(
        "error-test".to_string(),
        "test-agent".to_string(),
        temp_dir.path().to_path_buf(),
    );

    let error_content = ErrorContent {
        error_type: "TestError".to_string(),
        message: "Test error message".to_string(),
        stack_trace: Some("stack trace".to_string()),
        context: HashMap::from([("key".to_string(), "value".to_string())]),
    };

    logger.log_error(error_content, 3);

    let execution_result = ExecutionResult {
        success: false,
        score: 0.0,
        total_time_ms: 1000,
        statistics: logger.get_current_statistics(),
    };

    logger.complete(execution_result).unwrap();

    // Verify error is in YML
    let files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();

    let yml_content = std::fs::read_to_string(temp_dir.path().join(&files[0])).unwrap();
    assert!(yml_content.contains("error"));
    assert!(yml_content.contains("TestError"));
    assert!(yml_content.contains("Test error message"));
}
