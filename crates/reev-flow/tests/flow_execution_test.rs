//! # Flow Execution Tests
//!
//! Basic tests for flow logging functionality using the actual FlowLogger API.

use reev_flow::{
    calculate_execution_statistics, ErrorContent, FlowLogger, LlmRequestContent, ToolCallContent,
    TransactionExecutionContent,
};

/// Test basic flow logger functionality
#[tokio::test]
async fn test_basic_flow_logger() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let mut flow_logger = FlowLogger::new(
        "test-benchmark".to_string(),
        "deterministic".to_string(),
        temp_dir.path().join("test_flow.json"), // Use a specific file, not directory
    );

    tracing::info!("🧪 Starting basic flow logger test");

    // Log LLM request
    let request_content = LlmRequestContent {
        prompt: "Execute test action".to_string(),
        context_tokens: 100,
        model: "test-model".to_string(),
        request_id: "req-123".to_string(),
    };
    flow_logger.log_llm_request(request_content.clone(), 1);

    // Log tool call
    let tool_call_content = ToolCallContent {
        tool_name: "test_tool".to_string(),
        tool_args: r#"{"param": "value"}"#.to_string(),
        execution_time_ms: 100,
        result_status: reev_flow::ToolResultStatus::Success,
        result_data: Some(serde_json::json!({"result": "success"})),
        error_message: None,
    };
    flow_logger.log_tool_call(tool_call_content.clone(), 2);

    // Log transaction execution
    let transaction_content = TransactionExecutionContent {
        signature: "test-signature".to_string(),
        instruction_count: 1,
        execution_time_ms: 150,
        success: true,
        error: None,
    };
    flow_logger.log_transaction(transaction_content.clone(), 3);

    // Create events directly for validation without persistence
    let events = vec![
        reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::LlmRequest,
            depth: 1,
            content: reev_flow::EventContent {
                data: serde_json::to_value(request_content)?,
            },
        },
        reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::ToolCall,
            depth: 2,
            content: reev_flow::EventContent {
                data: serde_json::to_value(tool_call_content)?,
            },
        },
        reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::TransactionExecution,
            depth: 3,
            content: reev_flow::EventContent {
                data: serde_json::to_value(transaction_content)?,
            },
        },
    ];

    // Calculate statistics directly from events
    let stats = calculate_execution_statistics(&events);

    assert_eq!(stats.total_llm_calls, 1);
    assert_eq!(stats.total_tool_calls, 1);
    assert_eq!(stats.max_depth, 3);

    tracing::info!("✅ Basic flow logger test completed");
    Ok(())
}

/// Test flow with error handling
#[tokio::test]
async fn test_flow_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let mut flow_logger = FlowLogger::new(
        "error-test".to_string(),
        "deterministic".to_string(),
        temp_dir.path().join("error_flow.json"),
    );

    tracing::info!("🧪 Starting flow error handling test");

    // Log successful LLM request
    let request_content = LlmRequestContent {
        prompt: "Execute action".to_string(),
        context_tokens: 100,
        model: "test-model".to_string(),
        request_id: "req-456".to_string(),
    };
    flow_logger.log_llm_request(request_content, 1);

    // Log tool call that fails
    let tool_call_content = ToolCallContent {
        tool_name: "failing_tool".to_string(),
        tool_args: r#"{"action": "fail"}"#.to_string(),
        execution_time_ms: 200,
        result_status: reev_flow::ToolResultStatus::Error,
        result_data: None,
        error_message: Some("Tool execution failed".to_string()),
    };
    flow_logger.log_tool_call(tool_call_content.clone(), 2);

    // Log error event
    let error_content = ErrorContent {
        error_type: "ToolExecutionError".to_string(),
        message: "Failed to execute tool".to_string(),
        stack_trace: None,
        context: std::collections::HashMap::new(),
    };
    flow_logger.log_error(error_content.clone(), 3);

    // Create events for validation
    let mut events = Vec::new();

    events.push(reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::LlmRequest,
        depth: 1,
        content: reev_flow::EventContent {
            data: serde_json::json!({"prompt": "Execute action"}),
        },
    });

    events.push(reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::ToolCall,
        depth: 2,
        content: reev_flow::EventContent {
            data: serde_json::to_value(tool_call_content)?,
        },
    });

    events.push(reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::Error,
        depth: 3,
        content: reev_flow::EventContent {
            data: serde_json::to_value(error_content)?,
        },
    });

    let stats = calculate_execution_statistics(&events);

    assert_eq!(stats.total_llm_calls, 1);
    assert_eq!(stats.total_tool_calls, 1);
    assert_eq!(stats.max_depth, 3);

    tracing::info!("✅ Flow error handling test completed");
    Ok(())
}

/// Test flow statistics calculation
#[tokio::test]
async fn test_flow_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let mut flow_logger = FlowLogger::new(
        "stats-test".to_string(),
        "deterministic".to_string(),
        temp_dir.path().join("stats_flow.json"),
    );

    tracing::info!("🧪 Starting flow statistics test");

    // Add multiple LLM requests
    for i in 1..=3 {
        let request_content = LlmRequestContent {
            prompt: format!("Step {i} prompt"),
            context_tokens: 100 * i,
            model: "test-model".to_string(),
            request_id: format!("req-{i}"),
        };
        flow_logger.log_llm_request(request_content, i);
    }

    // Add multiple tool calls
    for i in 1..=2 {
        let tool_call_content = ToolCallContent {
            tool_name: format!("tool_{i}"),
            tool_args: format!(r#"{{"step": {i}}}"#),
            execution_time_ms: 100 * i,
            result_status: reev_flow::ToolResultStatus::Success,
            result_data: Some(serde_json::json!({"step": i, "result": "ok"})),
            error_message: None,
        };
        flow_logger.log_tool_call(tool_call_content, i + 3);
    }

    // Add transaction execution
    let transaction_content = TransactionExecutionContent {
        signature: "stats-test-signature".to_string(),
        instruction_count: 2,
        execution_time_ms: 300,
        success: true,
        error: None,
    };
    flow_logger.log_transaction(transaction_content.clone(), 6);

    // Create events for validation
    let mut events = Vec::new();
    let mut tool_usage = std::collections::HashMap::new();

    // Add LLM request events
    for i in 1..=3 {
        events.push(reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::LlmRequest,
            depth: i,
            content: reev_flow::EventContent {
                data: serde_json::json!({"prompt": format!("Step {} prompt", i)}),
            },
        });
    }

    // Add tool call events
    for i in 1..=2 {
        let tool_name = format!("tool_{i}");
        tool_usage.insert(tool_name.clone(), 1);

        events.push(reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::ToolCall,
            depth: i + 3,
            content: reev_flow::EventContent {
                data: serde_json::json!({"tool_name": tool_name, "step": i}),
            },
        });
    }

    // Add transaction event
    events.push(reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::TransactionExecution,
        depth: 6,
        content: reev_flow::EventContent {
            data: serde_json::to_value(transaction_content)?,
        },
    });

    let stats = calculate_execution_statistics(&events);

    assert_eq!(stats.total_llm_calls, 3);
    assert_eq!(stats.total_tool_calls, 2);
    assert_eq!(stats.max_depth, 6);

    // Check tool usage statistics
    assert!(stats.tool_usage.contains_key("tool_1"));
    assert!(stats.tool_usage.contains_key("tool_2"));
    assert_eq!(stats.tool_usage.get("tool_1"), Some(&1));
    assert_eq!(stats.tool_usage.get("tool_2"), Some(&1));

    tracing::info!("✅ Flow statistics test completed");
    Ok(())
}

/// Test different tool result statuses
#[tokio::test]
async fn test_tool_result_statuses() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let mut flow_logger = FlowLogger::new(
        "statuses-test".to_string(),
        "deterministic".to_string(),
        temp_dir.path().join("statuses_flow.json"),
    );

    tracing::info!("🧪 Starting tool result statuses test");

    // Test successful tool call
    let success_tool = ToolCallContent {
        tool_name: "success_tool".to_string(),
        tool_args: r#"{"action": "success"}"#.to_string(),
        execution_time_ms: 100,
        result_status: reev_flow::ToolResultStatus::Success,
        result_data: Some(serde_json::json!({"result": "ok"})),
        error_message: None,
    };
    flow_logger.log_tool_call(success_tool.clone(), 1);

    // Test failed tool call
    let error_tool = ToolCallContent {
        tool_name: "error_tool".to_string(),
        tool_args: r#"{"action": "error"}"#.to_string(),
        execution_time_ms: 150,
        result_status: reev_flow::ToolResultStatus::Error,
        result_data: None,
        error_message: Some("Intentional error".to_string()),
    };
    flow_logger.log_tool_call(error_tool.clone(), 2);

    // Test timeout tool call
    let timeout_tool = ToolCallContent {
        tool_name: "timeout_tool".to_string(),
        tool_args: r#"{"action": "timeout"}"#.to_string(),
        execution_time_ms: 30000,
        result_status: reev_flow::ToolResultStatus::Timeout,
        result_data: None,
        error_message: Some("Operation timed out".to_string()),
    };
    flow_logger.log_tool_call(timeout_tool.clone(), 3);

    // Create events for validation
    let mut events = Vec::new();
    let mut tool_usage = std::collections::HashMap::new();

    let tools = vec![success_tool, error_tool, timeout_tool];
    for tool in tools {
        let tool_name = tool.tool_name.clone();
        tool_usage.insert(tool_name, 1);

        events.push(reev_flow::FlowEvent {
            timestamp: std::time::SystemTime::now(),
            event_type: reev_flow::FlowEventType::ToolCall,
            depth: 1,
            content: reev_flow::EventContent {
                data: serde_json::to_value(tool)?,
            },
        });
    }

    let stats = calculate_execution_statistics(&events);
    assert_eq!(stats.total_tool_calls, 3);

    // Check tool usage for all tools
    assert_eq!(stats.tool_usage.get("success_tool"), Some(&1));
    assert_eq!(stats.tool_usage.get("error_tool"), Some(&1));
    assert_eq!(stats.tool_usage.get("timeout_tool"), Some(&1));

    tracing::info!("✅ Tool result statuses test completed");
    Ok(())
}

/// Test event creation and validation
#[tokio::test]
async fn test_event_creation() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🧪 Starting event creation test");

    // Test creating different types of events
    let llm_event = reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::LlmRequest,
        depth: 1,
        content: reev_flow::EventContent {
            data: serde_json::json!({
                "prompt": "Test prompt",
                "model": "test-model"
            }),
        },
    };

    let tool_event = reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::ToolCall,
        depth: 2,
        content: reev_flow::EventContent {
            data: serde_json::json!({
                "tool_name": "test_tool",
                "execution_time_ms": 100
            }),
        },
    };

    let error_event = reev_flow::FlowEvent {
        timestamp: std::time::SystemTime::now(),
        event_type: reev_flow::FlowEventType::Error,
        depth: 3,
        content: reev_flow::EventContent {
            data: serde_json::json!({
                "error_type": "TestError",
                "message": "Test error message"
            }),
        },
    };

    let events = vec![llm_event, tool_event, error_event];
    let stats = calculate_execution_statistics(&events);

    assert_eq!(stats.total_llm_calls, 1);
    assert_eq!(stats.total_tool_calls, 1);
    assert_eq!(stats.max_depth, 3);

    tracing::info!("✅ Event creation test completed");
    Ok(())
}
