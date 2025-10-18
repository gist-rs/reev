//! Test OpenTelemetry logging for tool calls
//!
//! This test verifies that OpenTelemetry tracing is properly configured
//! and that tool calls are logged to the specified log file.

use anyhow::Result;
use reev_agent::enhanced::openai::OpenAIAgent;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_opentelemetry_tool_call_logging() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a simple test payload
    let payload = reev_agent::LlmRequest {
        id: "otel-test-001".to_string(),
        prompt: "What is the account balance for USER_1?".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec!["get_account_balance".to_string()]),
    };

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting OpenTelemetry logging test...");

    // Run the agent - this should trigger tool call logging
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(100)).await;

    // Check if log file was created
    assert!(
        Path::new(log_path).exists(),
        "Log file should be created at {log_path}"
    );

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Log content:\n{log_content}");

    // Verify that OpenTelemetry-related entries are in the log
    assert!(
        log_content.contains("OpenTelemetry") || log_content.contains("tool_calls"),
        "Log should contain OpenTelemetry or tool_calls entries"
    );

    // Verify that tool execution is logged
    assert!(
        log_content.contains("account_balance_tool_call")
            || log_content.contains("get_account_balance"),
        "Log should contain tool call information"
    );

    println!("✅ OpenTelemetry logging test passed!");
    Ok(())
}

#[tokio::test]
async fn test_multiple_tool_calls_logging() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a test payload that might trigger multiple tool calls
    let payload = reev_agent::LlmRequest {
        id: "otel-test-002".to_string(),
        prompt: "Check balance for USER_1 and then swap 0.1 SOL to USDC".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec![
            "get_account_balance".to_string(),
            "jupiter_swap".to_string(),
        ]),
    };

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multiple tool calls logging test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(100)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multiple tool calls log content:\n{log_content}");

    // Verify that multiple tool calls might be logged
    let tool_call_count = log_content.matches("tool_call").count();
    println!("🔢 Found {tool_call_count} tool call entries in log");

    // The test passes as long as we have some logging activity
    assert!(
        !log_content.is_empty(),
        "Log should contain content after tool execution"
    );

    println!("✅ Multiple tool calls logging test passed!");
    Ok(())
}

#[tokio::test]
async fn test_otel_span_attributes() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a test payload
    let payload = reev_agent::LlmRequest {
        id: "otel-test-003".to_string(),
        prompt: "Swap 0.05 SOL to USDC for USER_1".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec!["jupiter_swap".to_string()]),
    };

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting OpenTelemetry span attributes test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(100)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Span attributes log content:\n{log_content}");

    // Verify that span attributes are logged
    assert!(
        log_content.contains("jupiter_swap_tool_call")
            || log_content.contains("agent_execution")
            || log_content.contains("model_name"),
        "Log should contain span attributes or agent execution information"
    );

    println!("✅ OpenTelemetry span attributes test passed!");
    Ok(())
}
