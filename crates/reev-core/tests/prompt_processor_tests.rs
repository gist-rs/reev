//! Tests for the PromptProcessor module
//!
//! These tests verify that the PromptProcessor correctly processes prompts
//! and handles various scenarios including the "all" keyword for transfers.
//! The tests focus on the prompt processing logic without making surfpool calls.

use anyhow::Result;
use reev_core::prompt_processor::PromptProcessor;
use rstest::*;
use serial_test::serial;
use std::env;
use tracing::info;

// Initialize tracing for all tests (runs only once)
#[serial]
fn init_tracing() {
    // Use serial_test to ensure this runs only once per test run
    let _ = tracing_subscriber::fmt::try_init();
}

// Load environment variables from .env file
#[serial]
fn setup_env() {
    // Use serial_test to ensure this runs only once per test run
    dotenvy::dotenv().ok();
}

/// Test basic prompt processing without any special keywords
#[rstest]
#[case("transfer 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("swap 0.5 SOL to USDC")]
#[tokio::test]
#[serial]
async fn test_basic_prompt_processing(#[case] prompt: &str) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Skip test if ZAI_API_KEY is not set
    if env::var("ZAI_API_KEY").is_err() {
        info!("Skipping test: ZAI_API_KEY not set");
        return Ok(());
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    let result = processor.process_prompt(prompt, test_address).await?;

    // Verify the prompt was processed
    assert!(
        !result.refined.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original, prompt,
        "Original prompt should match input"
    );

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    Ok(())
}

/// Test handling of the "all" keyword in transfer prompts
#[rstest]
#[case("transfer all SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[tokio::test]
#[serial]
async fn test_all_keyword_processing(#[case] prompt: &str) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Skip test if ZAI_API_KEY is not set
    if env::var("ZAI_API_KEY").is_err() {
        info!("Skipping test: ZAI_API_KEY not set");
        return Ok(());
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Starting prompt processing for: {}", prompt);
    let result = processor.process_prompt(prompt, test_address).await?;

    // Verify the prompt was processed
    assert!(
        !result.refined.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original, prompt,
        "Original prompt should match input"
    );

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    // Extract the transfer amount from the refined prompt
    // Expected pattern: "transfer {amount} SOL to {address}" or "send {amount} SOL to {address}"
    let transfer_amount = if let Some(start) = result.refined.find("transfer ") {
        // Find amount after "transfer "
        let after_transfer = &result.refined[start + "transfer ".len()..];
        if let Some(end) = after_transfer.find(" SOL") {
            after_transfer[..end].to_string()
        } else {
            String::new()
        }
    } else if let Some(start) = result.refined.find("send ") {
        // Find amount after "send "
        let after_send = &result.refined[start + "send ".len()..];
        if let Some(end) = after_send.find(" SOL") {
            after_send[..end].to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    info!("Extracted transfer amount: {}", transfer_amount);

    // Parse the extracted amount to float for comparison
    let parsed_amount = transfer_amount.parse::<f64>().unwrap_or(0.0);

    // Check if the refined prompt contains a reasonable amount (close to expected 1.004691919)
    // We'll check if it's close to 1.004 with some tolerance for floating point rounding
    let expected_amount = 1.004;
    let is_amount_correct = (parsed_amount - expected_amount).abs() < 0.01; // Allow 0.01 tolerance

    info!(
        "Is transfer amount correct ({} ~ {}): {}",
        parsed_amount, expected_amount, is_amount_correct
    );

    // Log full refined prompt for debugging
    info!("Full refined prompt: {}", result.refined);

    // Check if refined prompt properly replaced "all" with actual amount
    assert!(
        is_amount_correct,
        "Refined prompt should replace 'all' keyword with actual transferable amount (~{}), got {}",
        expected_amount, parsed_amount
    );

    Ok(())
}

/// Test handling of typos in prompts
#[rstest]
#[case("trasnfer 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("swp 0.5 SOL to USDC")]
#[tokio::test]
#[serial]
async fn test_typo_correction(#[case] prompt: &str) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Skip test if ZAI_API_KEY is not set
    if env::var("ZAI_API_KEY").is_err() {
        info!("Skipping test: ZAI_API_KEY not set");
        return Ok(());
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    let result = processor.process_prompt(prompt, test_address).await?;

    // Verify the prompt was processed
    assert!(
        !result.refined.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original, prompt,
        "Original prompt should match input"
    );

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    // For typo correction, changes may or may not be detected depending on LLM behavior
    // We just verify the prompt was processed without errors
    info!(
        "Typo correction test completed, changes detected: {}",
        result.changes_detected
    );

    Ok(())
}
