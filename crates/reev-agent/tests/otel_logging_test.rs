//! Test OpenTelemetry logging for tool calls
//!
//! This test verifies that OpenTelemetry tracing is properly configured
//! and that tool calls are logged to the specified log file.

use anyhow::Result;
use rig::tool::Tool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test(flavor = "multi_thread")]
async fn test_opentelemetry_tool_call_logging() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    println!("🧪 Starting OpenTelemetry logging test...");

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    // Test direct tool call to account balance tool
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    // Call the tool directly - this should trigger logging
    println!("🔧 About to call balance tool...");
    let result = balance_tool.call(balance_args).await;
    match &result {
        Ok(output) => println!("✅ Balance tool succeeded: {output}"),
        Err(e) => println!("❌ Balance tool failed: {e}"),
    }
    let _result = result;

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

    // Verify that tool logging session was started
    assert!(
        log_content.contains("Tool logging session started"),
        "Log should contain session start message"
    );

    // Verify that tool execution is logged
    assert!(
        log_content.contains("get_account_balance") || log_content.contains("account_balance"),
        "Log should contain tool call information"
    );

    println!("✅ OpenTelemetry logging test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_tool_calls_logging() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multiple tool calls logging test...");

    // Test multiple tool calls directly
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    // Call balance tool
    let _balance_result = balance_tool.call(balance_args).await;

    // Test jupiter swap tool (this will likely fail due to network issues but should still log)
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 1000000, // 0.001 SOL
        slippage_bps: Some(100),
    };

    // Call jupiter tool (may fail but should still log the attempt)
    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multiple tool calls log content:\n{log_content}");

    // Verify that multiple tool calls are logged
    let tool_call_count = log_content.matches("Calling tool").count();
    println!("🔢 Found {tool_call_count} tool call entries in log");

    // The test passes as long as we have some logging activity
    assert!(
        !log_content.is_empty(),
        "Log should contain content after tool execution"
    );

    assert!(
        tool_call_count >= 1,
        "Should have at least one tool call logged"
    );

    println!("✅ Multiple tool calls logging test passed!");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_otel_span_attributes() -> Result<()> {
    // Clean up existing log file
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
    }

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting OpenTelemetry span attributes test...");

    // Test jupiter swap tool for span attributes logging
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 500000, // 0.0005 SOL
        slippage_bps: Some(100),
    };

    // Call jupiter tool (may fail but should still log the attempt)
    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(100)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Span attributes log content:\n{log_content}");

    // Verify that span attributes are logged
    assert!(
        log_content.contains("jupiter_swap") || log_content.contains("JupiterSwapTool"),
        "Log should contain jupiter swap tool information"
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

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step balance check then swap test...");

    // Step 1: Check balance
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    let _balance_result = balance_tool.call(balance_args).await;

    // Step 2: Attempt swap
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 1000000, // 0.001 SOL
        slippage_bps: Some(100),
    };

    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step balance check then swap log content:\n{log_content}");

    // Verify that we have multiple tool interactions
    let balance_checks = log_content.matches("get_account_balance").count();
    let swap_attempts = log_content.matches("jupiter_swap").count();

    println!("🔊 Found {balance_checks} balance checks and {swap_attempts} swap attempts");

    assert!(
        !log_content.is_empty(),
        "Log should contain content after multi-step execution"
    );

    assert!(
        balance_checks >= 1 && swap_attempts >= 1,
        "Should have at least one balance check and one swap attempt"
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

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step swap then balance verification test...");

    // Step 1: Attempt swap
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 2000000, // 0.002 SOL
        slippage_bps: Some(100),
    };

    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    // Step 2: Verify balance
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    let _balance_result = balance_tool.call(balance_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(200)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step swap then verify log content:\n{log_content}");

    // Verify sequence of operations
    let swap_attempts = log_content.matches("jupiter_swap").count();
    let balance_checks = log_content.matches("get_account_balance").count();

    assert!(
        swap_attempts >= 1 && balance_checks >= 1,
        "Log should contain evidence of both swap and balance verification operations"
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

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step three operation sequence test...");

    // Step 1: Check initial SOL balance
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    let _initial_balance = balance_tool.call(balance_args).await;

    // Step 2: Swap SOL to USDC
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 100000, // 0.0001 SOL
        slippage_bps: Some(100),
    };

    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    // Step 3: Check final USDC balance
    let usdc_balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        account_type: None,
    };

    let _final_balance = balance_tool.call(usdc_balance_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(300)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step three operation sequence log content:\n{log_content}");

    // Count different types of operations
    let balance_operations = log_content.matches("get_account_balance").count();
    let swap_operations = log_content.matches("jupiter_swap").count();

    println!(
        "🔊 Found {balance_operations} balance operations and {swap_operations} swap operations"
    );

    assert!(
        balance_operations >= 2 && swap_operations >= 1,
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

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step conditional flow test...");

    // Step 1: Check balance
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    let balance_result = balance_tool.call(balance_args).await;

    // Step 2: Check if balance is sufficient and conditionally swap
    // For testing purposes, we'll always attempt the swap to test logging
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let jupiter_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 100000000, // 0.1 SOL
        slippage_bps: Some(100),
    };

    let _jupiter_result = jupiter_tool.call(jupiter_args).await;

    println!("🔍 Balance result: {}", balance_result.unwrap_or_default());

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

    // Initialize tool logging
    reev_agent::enhanced::openai::init_tool_logging()?;

    // Create a simple key_map
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    println!("🧪 Starting multi-step error recovery flow test...");

    // Step 1: Attempt large swap that should fail
    let jupiter_tool = reev_agent::tools::jupiter_swap::JupiterSwapTool {
        key_map: key_map.clone(),
    };

    let large_swap_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 1000000000000, // 1000 SOL - should fail
        slippage_bps: Some(100),
    };

    let _large_swap_result = jupiter_tool.call(large_swap_args).await;

    // Step 2: Check balance
    let balance_tool = reev_agent::tools::discovery::balance_tool::AccountBalanceTool {
        key_map: key_map.clone(),
    };

    let balance_args = reev_agent::tools::discovery::balance_tool::AccountBalanceArgs {
        pubkey: "USER_1".to_string(),
        token_mint: None,
        account_type: None,
    };

    let _balance_result = balance_tool.call(balance_args).await;

    // Step 3: Attempt smaller swap
    let small_swap_args = reev_agent::tools::jupiter_swap::JupiterSwapArgs {
        user_pubkey: "USER_1".to_string(),
        input_mint: "So11111111111111111111111111111111111111112".to_string(),
        output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        amount: 10000000, // 0.01 SOL
        slippage_bps: Some(100),
    };

    let _small_swap_result = jupiter_tool.call(small_swap_args).await;

    // Wait a moment for logs to be written
    sleep(Duration::from_millis(300)).await;

    // Read and verify log content
    let log_content = fs::read_to_string(log_path)?;
    println!("📝 Multi-step error recovery flow log content:\n{log_content}");

    // Verify that we have some activity (might include errors)
    let balance_checks = log_content.matches("get_account_balance").count();
    let swap_attempts = log_content.matches("jupiter_swap").count();

    assert!(
        balance_checks >= 1 && swap_attempts >= 2,
        "Log should contain evidence of balance check and multiple swap attempts"
    );

    println!("✅ Multi-step error recovery flow test passed!");
    Ok(())
}
