//! OpenTelemetry Tool Call Logging Demo
//!
//! This example demonstrates how the reev-agent uses OpenTelemetry to log
//! all tool calls made by the AI agent during execution. The logs are written
//! to `logs/tool_calls.log` and include detailed information about:
//!
//! - Agent execution spans
//! - Individual tool call spans
//! - Timing information
//! - Parameters and results
//! - Error handling
//!
//! Run this example with:
//! ```bash
//! cargo run --example otel_tool_logging_demo
//! ```

use anyhow::Result;
use reev_agent::enhanced::openai::OpenAIAgent;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 OpenTelemetry Tool Call Logging Demo");
    println!("==========================================");

    // Clean up any existing log file for this demo
    let log_path = "logs/tool_calls.log";
    if Path::new(log_path).exists() {
        fs::remove_file(log_path)?;
        println!("🧹 Cleaned up existing log file");
    }

    println!(
        "\n📋 This demo will run several agent scenarios to demonstrate OpenTelemetry logging:"
    );
    println!("   1. Account balance query");
    println!("   2. Jupiter swap operation");
    println!("   3. Multi-tool workflow");
    println!("   4. Error handling scenario");

    // Scenario 1: Account Balance Query
    println!("\n🔍 Scenario 1: Account Balance Query");
    println!("-------------------------------------");
    run_balance_query_scenario().await?;

    // Scenario 2: Jupiter Swap Operation
    println!("\n💱 Scenario 2: Jupiter Swap Operation");
    println!("--------------------------------------");
    run_swap_scenario().await?;

    // Scenario 3: Multi-tool Workflow
    println!("\n🔧 Scenario 3: Multi-tool Workflow");
    println!("-----------------------------------");
    run_multi_tool_scenario().await?;

    // Scenario 4: Error Handling
    println!("\n❌ Scenario 4: Error Handling");
    println!("-----------------------------");
    run_error_scenario().await?;

    // Display the final log content
    println!("\n📄 Final Log Content");
    println!("=====================");
    if Path::new(log_path).exists() {
        let log_content = fs::read_to_string(log_path)?;
        println!("Log file size: {} bytes", log_content.len());
        println!("Log content preview:\n");
        println!("{log_content}");
    } else {
        println!("❌ Log file was not created!");
    }

    println!("\n✅ Demo completed! Check {log_path} for detailed tool call logs.");
    Ok(())
}

async fn run_balance_query_scenario() -> Result<()> {
    println!("Running account balance query...");

    let payload = reev_agent::LlmRequest {
        id: "demo-balance-001".to_string(),
        prompt: "What is the SOL balance for USER_1?".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec!["get_account_balance".to_string()]),
    };

    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    match OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await {
        Ok(_result) => println!("✅ Balance query completed successfully"),
        Err(e) => println!("⚠️  Balance query failed (expected in demo): {e}"),
    }

    sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn run_swap_scenario() -> Result<()> {
    println!("Running Jupiter swap operation...");

    let payload = reev_agent::LlmRequest {
        id: "demo-swap-001".to_string(),
        prompt: "Swap 0.1 SOL to USDC for USER_1 with 3% slippage".to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec!["jupiter_swap".to_string()]),
    };

    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    match OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await {
        Ok(_result) => println!("✅ Swap operation completed successfully"),
        Err(e) => println!("⚠️  Swap operation failed (expected in demo): {e}"),
    }

    sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn run_multi_tool_scenario() -> Result<()> {
    println!("Running multi-tool workflow...");

    let payload = reev_agent::LlmRequest {
        id: "demo-multi-001".to_string(),
        prompt: "Check USER_1's SOL balance, then swap 0.05 SOL to USDC if sufficient funds"
            .to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec![
            "get_account_balance".to_string(),
            "jupiter_swap".to_string(),
        ]),
    };

    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    match OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await {
        Ok(_result) => println!("✅ Multi-tool workflow completed successfully"),
        Err(e) => println!("⚠️  Multi-tool workflow failed (expected in demo): {e}"),
    }

    sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn run_error_scenario() -> Result<()> {
    println!("Running error handling scenario...");

    let payload = reev_agent::LlmRequest {
        id: "demo-error-001".to_string(),
        prompt: "Swap 1000 SOL to USDC for USER_1 (should trigger insufficient funds error)"
            .to_string(),
        context_prompt: "".to_string(),
        model_name: "qwen3-vl-30b-a3b-instruct".to_string(),
        mock: false,
        initial_state: None,
        allowed_tools: Some(vec![
            "get_account_balance".to_string(),
            "jupiter_swap".to_string(),
        ]),
    };

    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_1".to_string(),
        "11111111111111111111111111111112".to_string(),
    );

    match OpenAIAgent::run("qwen3-vl-30b-a3b-instruct", payload, key_map).await {
        Ok(_result) => println!("✅ Error scenario completed (unexpected success)"),
        Err(e) => println!("✅ Error scenario handled correctly: {e}"),
    }

    sleep(Duration::from_millis(100)).await;
    Ok(())
}
