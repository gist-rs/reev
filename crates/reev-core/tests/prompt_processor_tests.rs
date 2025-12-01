//! Tests for the PromptProcessor module
//!
//! These tests verify that the PromptProcessor correctly processes prompts
//! and handles various scenarios including the "all" keyword for transfers.
//! The tests focus on the prompt processing logic without making surfpool calls.

use anyhow::{anyhow, Result};
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

/// Extract amount from a refined prompt for verification
fn extract_amount_from_refined_prompt(refined: &str) -> Result<f64> {
    // Convert to lowercase for case-insensitive matching
    let refined_lower = refined.to_lowercase();

    // Try various patterns for extracting amount from refined prompt
    // Expected patterns: "transfer {amount} SOL", "send {amount} SOL", "swap {amount} SOL", etc.
    let patterns = ["transfer ", "send ", "swap "];

    for pattern in &patterns {
        if let Some(start) = refined_lower.find(pattern) {
            // Find amount after the pattern
            let after_pattern = &refined_lower[start + pattern.len()..];
            if let Some(end) = after_pattern.find(" sol") {
                match after_pattern[..end].parse::<f64>() {
                    Ok(amount) => return Ok(amount),
                    Err(_) => continue, // Try next pattern if parsing fails
                }
            }
        }
    }

    // If standard patterns don't work, try to find any number followed by SOL (case-insensitive)
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s+[Ss][Oo][Ll]")?;
    if let Some(captures) = re.captures(refined) {
        if let Some(amount_str) = captures.get(1) {
            return Ok(amount_str.as_str().parse::<f64>()?);
        }
    }

    Err(anyhow!(
        "Could not extract amount from refined prompt: {refined}"
    ))
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

    // For prompts without "all", usable_amount should be None
    assert!(
        result.usable_amount.is_none(),
        "usable_amount should be None for non-'all' keyword prompts"
    );

    // Extract the amount from the refined prompt to verify it's present
    let refined_amount = extract_amount_from_refined_prompt(&result.refined)?;
    assert!(refined_amount > 0.0, "Refined amount should be positive");

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Refined amount: {}", refined_amount);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    Ok(())
}

/// Test handling of the "all" keyword in transfer prompts
#[rstest]
#[case("transfer all SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("swap all SOL for USDC")]
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

    // For "all" keyword prompts, usable_amount should be Some
    assert!(
        result.usable_amount.is_some(),
        "usable_amount should be Some for 'all' keyword prompts"
    );

    let usable_amount = result.usable_amount.unwrap();
    assert!(
        usable_amount > 0.0,
        "usable_amount should be positive for 'all' keyword prompts"
    );

    // Extract amount from refined prompt for comparison
    let refined_amount = extract_amount_from_refined_prompt(&result.refined)?;

    // Verify that the amount in the refined prompt matches usable_amount (within tolerance)
    assert!(
        (refined_amount - usable_amount).abs() < 0.000001,
        "Amount in refined prompt ({refined_amount}) should match usable_amount ({usable_amount})"
    );

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Usable amount: {}", usable_amount);
    info!("Refined amount: {}", refined_amount);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    // Log full refined prompt for debugging
    info!("Full refined prompt: {}", result.refined);

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

    // For typo prompts without "all", usable_amount should be None
    assert!(
        result.usable_amount.is_none(),
        "usable_amount should be None for non-'all' keyword prompts with typos"
    );

    // Extract the amount from the refined prompt to verify it's present
    let refined_amount = extract_amount_from_refined_prompt(&result.refined)?;
    assert!(refined_amount > 0.0, "Refined amount should be positive");

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Refined amount: {}", refined_amount);
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

/// Test handling of "all" keyword with typos
#[rstest]
#[case("trasnfer all SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("swp all SOL for USDC")]
#[tokio::test]
#[serial]
async fn test_all_keyword_with_typos(#[case] prompt: &str) -> Result<()> {
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

    info!(
        "Starting prompt processing for typo with 'all' keyword: {}",
        prompt
    );
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

    // For "all" keyword prompts with typos, usable_amount should be Some
    assert!(
        result.usable_amount.is_some(),
        "usable_amount should be Some for 'all' keyword prompts even with typos"
    );

    let usable_amount = result.usable_amount.unwrap();
    assert!(
        usable_amount > 0.0,
        "usable_amount should be positive for 'all' keyword prompts"
    );

    // Extract amount from refined prompt for comparison
    let refined_amount = extract_amount_from_refined_prompt(&result.refined)?;

    // Verify that the amount in the refined prompt matches usable_amount (within tolerance)
    assert!(
        (refined_amount - usable_amount).abs() < 0.000001,
        "Amount in refined prompt ({refined_amount}) should match usable_amount ({usable_amount})"
    );

    // Log results for inspection
    info!("Original: {}", result.original);
    info!("Refined: {}", result.refined);
    info!("Usable amount: {}", usable_amount);
    info!("Refined amount: {}", refined_amount);
    info!("Changes detected: {}", result.changes_detected);
    info!("Confidence: {}", result.get_confidence());

    // Log full refined prompt for debugging
    info!("Full refined prompt: {}", result.refined);

    Ok(())
}
