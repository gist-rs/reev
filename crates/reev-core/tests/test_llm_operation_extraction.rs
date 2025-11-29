//! Focused test for LLM-based operation extraction
//!
//! This test directly tests the `extract_operations_from_prompt` function in YmlGenerator
//! to verify that multi-step operations are correctly parsed by the LLM.
//! This helps isolate issues with multi-step operation splitting without running full e2e tests.

use anyhow::Result;
use reev_core::llm::glm_client::init_glm_client;
use reev_core::yml_generator::YmlGenerator;
use std::sync::Arc;
use tracing::info;

/// Test GLM client with multi-step prompt
#[tokio::test]
#[serial_test::serial]
async fn test_glm_client_multi_step() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    println!("DEBUG: ZAI_API_KEY length: {}", zai_api_key.len());

    // Initialize GLM client
    let llm_client = init_glm_client()?;

    println!("DEBUG: GLM client initialized successfully");

    // Test with a multi-step prompt
    let multi_step_prompt = "swap 0.1 SOL to USDC then lend 10 USDC";
    println!("DEBUG: Sending multi-step prompt: {multi_step_prompt}");

    let response = llm_client.generate_flow(multi_step_prompt).await?;
    println!("DEBUG: Multi-step prompt response: {response}");

    Ok(())
}

/// Simple test to verify GLM client is working
#[tokio::test]
#[serial_test::serial]
async fn test_glm_client_simple() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    println!("DEBUG: ZAI_API_KEY length: {}", zai_api_key.len());

    // Initialize GLM client
    let llm_client = init_glm_client()?;

    println!("DEBUG: GLM client initialized successfully");

    // Test with a very simple prompt
    let simple_prompt = "swap";
    println!("DEBUG: Sending very simple prompt: {simple_prompt}");

    let response = llm_client.generate_flow(simple_prompt).await?;
    println!("DEBUG: Simple prompt response: {response}");

    Ok(())
}

/// Test operation extraction for simple single-step operations
#[tokio::test]
#[serial_test::serial]
async fn test_simple_operation_extraction() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    println!("DEBUG: GLM client initialized successfully");

    // Create YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client_arc);

    println!("DEBUG: YmlGenerator created with LLM client");

    // Test single-step operation
    let prompt = "swap 0.1 SOL to USDC";
    info!("Testing operation extraction for prompt: {}", prompt);
    println!(
        "DEBUG: About to test operation extraction for prompt: {prompt}"
    );

    // Call extract_operations_from_prompt directly
    let operations = reev_core::yml_generator::extract_operations_from_prompt(
        prompt,
        yml_generator.llm_client(),
    )
    .await;

    info!("Extracted operations: {:?}", operations);
    println!("DEBUG: Extracted {} operations", operations.len());
    for (i, op) in operations.iter().enumerate() {
        println!("DEBUG: Operation {}: {}", i + 1, op);
    }

    // Verify that we get exactly one operation
    if operations.len() != 1 {
        println!(
            "ERROR: Expected 1 operation but got {}. Failing test.",
            operations.len()
        );
    }
    assert_eq!(
        operations.len(),
        1,
        "Expected 1 operation for simple prompt"
    );
    assert!(
        operations[0].contains("swap"),
        "Operation should contain 'swap'"
    );
    assert!(
        operations[0].contains("0.1"),
        "Operation should contain amount"
    );
    assert!(
        operations[0].contains("SOL"),
        "Operation should contain source token"
    );
    assert!(
        operations[0].contains("USDC"),
        "Operation should contain target token"
    );

    Ok(())
}

/// Test operation extraction for multi-step operations
#[tokio::test]
#[serial_test::serial]
async fn test_multi_step_operation_extraction() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    // Create YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client_arc);

    // Test multi-step operation with "then"
    let prompt = "swap 0.1 SOL to USDC then lend 10 USDC";
    info!("Testing operation extraction for prompt: {}", prompt);

    // Call extract_operations_from_prompt directly
    let operations = reev_core::yml_generator::extract_operations_from_prompt(
        prompt,
        yml_generator.llm_client(),
    )
    .await;

    info!("Extracted operations: {:?}", operations);

    // Verify that we get exactly two operations
    assert_eq!(
        operations.len(),
        2,
        "Expected 2 operations for multi-step prompt"
    );

    // Check first operation (swap)
    assert!(
        operations[0].contains("swap"),
        "First operation should contain 'swap'"
    );
    assert!(
        operations[0].contains("0.1"),
        "First operation should contain amount"
    );
    assert!(
        operations[0].contains("SOL"),
        "First operation should contain source token"
    );
    assert!(
        operations[0].contains("USDC"),
        "First operation should contain target token"
    );

    // Check second operation (lend)
    assert!(
        operations[1].contains("lend"),
        "Second operation should contain 'lend'"
    );
    assert!(
        operations[1].contains("10"),
        "Second operation should contain amount"
    );
    assert!(
        operations[1].contains("USDC"),
        "Second operation should contain token"
    );

    Ok(())
}

/// Test operation extraction for complex multi-step operations
#[tokio::test]
#[serial_test::serial]
async fn test_complex_multi_step_operation_extraction() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    // Create YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client_arc);

    // Test complex multi-step operation with "and then"
    let prompt = "I want to swap 0.1 SOL to USDC and then lend 10 USDC to Jupiter. This is a two-step process that must be executed in sequence.";
    info!("Testing operation extraction for prompt: {}", prompt);

    // Call extract_operations_from_prompt directly
    let operations = reev_core::yml_generator::extract_operations_from_prompt(
        prompt,
        yml_generator.llm_client(),
    )
    .await;

    info!("Extracted operations: {:?}", operations);

    // Verify that we get exactly two operations
    assert_eq!(
        operations.len(),
        2,
        "Expected 2 operations for complex multi-step prompt"
    );

    // Check first operation (swap)
    assert!(
        operations[0].contains("swap"),
        "First operation should contain 'swap'"
    );
    assert!(
        operations[0].contains("0.1"),
        "First operation should contain amount"
    );
    assert!(
        operations[0].contains("SOL"),
        "First operation should contain source token"
    );
    assert!(
        operations[0].contains("USDC"),
        "First operation should contain target token"
    );

    // Check second operation (lend)
    assert!(
        operations[1].contains("lend"),
        "Second operation should contain 'lend'"
    );
    assert!(
        operations[1].contains("10"),
        "Second operation should contain amount"
    );
    assert!(
        operations[1].contains("USDC"),
        "Second operation should contain token"
    );

    Ok(())
}

/// Test tool determination for single-step operations
#[tokio::test]
#[serial_test::serial]
async fn test_single_step_tool_determination() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    // Create YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client_arc);

    // Test tool determination for swap
    let prompt = "swap 0.1 SOL to USDC";
    info!("Testing tool determination for prompt: {}", prompt);

    // Call determine_expected_tools directly
    let tools =
        reev_core::yml_generator::determine_expected_tools(prompt, yml_generator.llm_client())
            .await;

    info!("Determined tools: {:?}", tools);

    // Verify that we get the expected tool
    assert!(tools.is_some(), "Should determine tools for swap operation");
    let tools = tools.unwrap();
    assert_eq!(tools.len(), 1, "Should determine exactly 1 tool");
    assert_eq!(
        tools[0],
        reev_types::tools::ToolName::JupiterSwap,
        "Should determine JupiterSwap tool"
    );

    Ok(())
}

/// Test what GLM client returns directly for simple prompts
#[tokio::test]
#[serial_test::serial]
async fn test_glm_client_direct_response() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    println!("DEBUG: ZAI_API_KEY length: {}", zai_api_key.len());
    println!(
        "DEBUG: ZAI_API_KEY prefix: {}...",
        &zai_api_key[..std::cmp::min(8, zai_api_key.len())]
    );

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    println!("DEBUG: Testing GLM client directly");

    // Test with simple prompt
    let simple_prompt = "swap 0.1 SOL to USDC";
    println!("DEBUG: Sending simple prompt: {simple_prompt}");

    let simple_response = llm_client_arc.generate_flow(simple_prompt).await?;
    println!("DEBUG: Simple prompt response: {simple_response}");

    // Test with multi-step prompt
    let multi_step_prompt = "swap 0.1 SOL to USDC then lend 10 USDC";
    println!("DEBUG: Sending multi-step prompt: {multi_step_prompt}");

    let multi_step_response = llm_client_arc.generate_flow(multi_step_prompt).await?;
    println!("DEBUG: Multi-step prompt response: {multi_step_response}");

    // Test with complex prompt
    let complex_prompt = r#"Analyze this DeFi request and break it down into individual operations that should be executed sequentially:
"swap 0.1 SOL to USDC then lend 10 USDC"

Rules:
1. Each operation should be a single, actionable DeFi task (swap, lend, borrow, transfer, etc.)
2. Operations must be executed in the order they appear
3. Keep each operation concise but complete with all necessary context
4. Do NOT split a single operation into multiple steps
5. Return ONLY a valid JSON array of operation strings

Example:
Input: "Swap 1 SOL to USDC and then lend 500 USDC"
Output: ["Swap 1 SOL to USDC", "Lend 500 USDC"]

CRITICAL: Respond with ONLY a valid JSON array. No explanations, no markdown formatting, no extra text."#;

    println!(
        "DEBUG: Sending complex prompt (length: {})",
        complex_prompt.len()
    );

    let complex_response = llm_client_arc.generate_flow(complex_prompt).await?;
    println!("DEBUG: Complex prompt response: {complex_response}");

    Ok(())
}

/// Test tool determination for multi-step operations
#[tokio::test]
#[serial_test::serial]
async fn test_multi_step_tool_determination() -> Result<()> {
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = std::env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    // Initialize GLM client
    let llm_client = init_glm_client()?;
    let llm_client_arc: Arc<dyn reev_core::planner::LlmClient> = Arc::from(llm_client);

    // Create YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client_arc);

    // Test tool determination for multi-step
    let prompt = "swap 0.1 SOL to USDC then lend 10 USDC";
    info!("Testing tool determination for prompt: {}", prompt);

    // Call determine_expected_tools directly
    let tools =
        reev_core::yml_generator::determine_expected_tools(prompt, yml_generator.llm_client())
            .await;

    info!("Determined tools: {:?}", tools);

    // Verify that we get the expected tools
    assert!(
        tools.is_some(),
        "Should determine tools for multi-step operation"
    );
    let tools = tools.unwrap();
    assert_eq!(tools.len(), 2, "Should determine exactly 2 tools");

    // Check that both JupiterSwap and GetJupiterLendEarnPosition are included
    let has_swap = tools
        .iter()
        .any(|t| matches!(t, reev_types::tools::ToolName::JupiterSwap));
    let has_lend = tools
        .iter()
        .any(|t| matches!(t, reev_types::tools::ToolName::GetJupiterLendEarnPosition));

    assert!(has_swap, "Should include JupiterSwap tool");
    assert!(has_lend, "Should include GetJupiterLendEarnPosition tool");

    Ok(())
}
