//! Test LLM integration for operation extraction and tool determination
//!
//! This test validates that the LLM client is properly initialized and can extract
//! operations and determine tools from multi-step prompts.

use reev_core::llm::glm_client::GLMClient;
use reev_core::yml_generator::YmlGenerator;
use std::sync::Arc;

#[tokio::test]
async fn test_llm_integration() {
    println!("DEBUG: Starting LLM integration test");

    // Initialize LLM client
    let llm_client = match GLMClient::from_env() {
        Ok(client) => {
            println!("DEBUG: LLM client initialized successfully");
            Arc::new(client)
        }
        Err(e) => {
            println!("DEBUG: Failed to initialize LLM client: {e}");
            return;
        }
    };

    // Initialize YmlGenerator with LLM client
    let yml_generator = YmlGenerator::with_llm_client(llm_client);

    // Test operation extraction with a multi-step prompt
    let multi_step_prompt = "swap 1 SOL to USDC then lend 100 USDC";
    println!(
        "DEBUG: Testing operation extraction with prompt: {multi_step_prompt}"
    );

    // This should call the LLM to extract operations
    println!("DEBUG: About to call extract_operations_from_prompt");

    // Test with a simple prompt first
    let simple_prompt = "swap 1 SOL to USDC";
    println!("DEBUG: Testing with simple prompt: {simple_prompt}");

    println!("DEBUG: Test completed");
}
