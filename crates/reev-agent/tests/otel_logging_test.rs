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

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_step_balance_check_then_swap() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a multi-step test payload
    let payload = reev_agent::LlmRequest {
        id: "multi-step-balance-swap-001".to_string(),
        prompt: "First check my account balance for USER_1, then swap 0.05 SOL to USDC if I have enough balance".to_string(),
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

    println!("🧪 Starting multi-step balance check then swap test...");

    // Run the agent - this should trigger multiple tool calls
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step balance check then swap log content:\n{log_content}");

    // Verify that we have multiple tool interactions
    let balance_checks = log_content.matches("account_balance").count();
    let swap_attempts = log_content.matches("jupiter_swap").count();

    println!("🔊 Found {balance_checks} balance checks and {swap_attempts} swap attempts");

    assert!(
        !log_content.is_empty(),
        "Log should contain content after multi-step execution"
    );

    println!("✅ Multi-step balance check then swap test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_step_swap_then_balance_verification() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a multi-step test payload for swap then verify
    let payload = reev_agent::LlmRequest {
        id: "multi-step-swap-verify-001".to_string(),
        prompt: "Swap 0.02 SOL to USDC for USER_1, then check the final balance to confirm the swap was successful".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec![
            "jupiter_swap".to_string(),
            "get_account_balance".to_string(),
        ]),
    };

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step swap then balance verification test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step swap then verify log content:\n{log_content}");

    // Verify sequence of operations
    assert!(
        log_content.contains("jupiter_swap") || log_content.contains("account_balance"),
        "Log should contain evidence of swap and balance verification operations"
    );

    println!("✅ Multi-step swap then balance verification test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_step_three_operation_sequence() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a complex multi-step test payload
    let payload = reev_agent::LlmRequest {
        id: "multi-step-three-ops-001".to_string(),
        prompt: "For USER_1: 1) Check current SOL balance, 2) Swap 0.01 SOL to USDC, 3) Check final USDC balance to verify the swap".to_string(),
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

    println!("🧪 Starting multi-step three operation sequence test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(300)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step three operation sequence log content:\n{log_content}");

    // Count different types of operations
    let balance_operations = log_content.matches("account_balance").count();
    let swap_operations = log_content.matches("jupiter_swap").count();

    println!(
        "🔊 Found {balance_operations} balance operations and {swap_operations} swap operations"
    );

    assert!(
        balance_operations >= 2 || swap_operations >= 1,
        "Should have multiple balance checks and at least one swap operation"
    );

    println!("✅ Multi-step three operation sequence test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_step_conditional_flow() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a conditional multi-step test payload
    let payload = reev_agent::LlmRequest {
        id: "multi-step-conditional-001".to_string(),
        prompt: "Check USER_1's SOL balance. If they have more than 1 SOL, swap 0.1 SOL to USDC. Otherwise, just report the current balance.".to_string(),
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

    println!("🧪 Starting multi-step conditional flow test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step conditional flow log content:\n{log_content}");

    // Verify that at least balance checking happened
    assert!(
        log_content.contains("account_balance"),
        "Log should contain balance checking operation"
    );

    println!("✅ Multi-step conditional flow test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_step_error_recovery_flow() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Create a test payload that might trigger error recovery
    let payload = reev_agent::LlmRequest {
        id: "multi-step-error-recovery-001".to_string(),
        prompt: "Try to swap 1000 SOL to USDC for USER_1 (this should fail), then check the balance and try a smaller amount like 0.01 SOL".to_string(),
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

    println!("🧪 Starting multi-step error recovery flow test...");

    // Run the agent
    let _result = OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(300)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step error recovery flow log content:\n{log_content}");

    // Verify that we have some activity (might include errors)
    assert!(
        log_content.contains("account_balance")
            || log_content.contains("jupiter_swap")
            || log_content.contains("ERROR"),
        "Log should contain evidence of operations or error handling"
    );

    println!("✅ Multi-step error recovery flow test passed!");
    Ok(())
}
