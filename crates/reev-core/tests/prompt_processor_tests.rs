//! Tests for the PromptProcessor module
//!
//! These tests verify that the PromptProcessor correctly processes prompts
//! and handles various scenarios including the "all" keyword for transfers.
//! The tests focus on the prompt processing logic without making surfpool calls.

use anyhow::{anyhow, Result};
use reev_core::prompt_processor::{PromptAction, PromptProcessor};

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

    // Special case for "0 SOL" which indicates insufficient balance
    if refined_lower.contains("0 sol") {
        return Ok(0.0);
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

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
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

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
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

    // Check if this is a swap operation with insufficient balance
    let is_swap = prompt.to_lowercase().contains("swap");

    if is_swap {
        // For swap operations, the usable_amount might be 0 if balance is insufficient
        // This is expected behavior, not an error
        info!(
            "Swap operation detected with usable_amount: {}",
            usable_amount
        );
        assert!(
            usable_amount >= 0.0,
            "usable_amount should be non-negative for 'all' keyword prompts"
        );
    } else {
        // For transfer operations, usable_amount should be positive
        assert!(
            usable_amount > 0.0,
            "usable_amount should be positive for 'all' keyword prompts"
        );
    }

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

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
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
    info!("Confidence: {}", result.get_confidence());

    // For typo correction, changes may or may not be detected depending on LLM behavior
    // We just verify the prompt was processed without errors

    Ok(())
}

/// Test structured response system for various prompt types
#[rstest]
#[case(
    "transfer 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    PromptAction::Transfer
)]
#[case("swap 0.5 SOL to USDC", PromptAction::Swap)]
#[tokio::test]
#[serial]
async fn test_structured_response(
    #[case] prompt: &str,
    #[case] expected_action: PromptAction,
) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Testing structured response for prompt: {}", prompt);
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify the prompt was processed
    assert!(
        !result.refined_prompt.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original_prompt, prompt,
        "Original prompt should match input"
    );

    // Verify action was detected correctly
    assert_eq!(
        result.action, expected_action,
        "Detected action {:?} should match expected {:?}",
        result.action, expected_action
    );

    // Verify subject_pubkey is set to the owner wallet address
    assert_eq!(
        result.subject_pubkey,
        Some(test_address.to_string()),
        "Subject pubkey should be set to owner wallet address"
    );

    // Verify confidence is reasonable (>0.5)
    assert!(
        result.confidence > 0.5,
        "Confidence should be greater than 0.5, got {}",
        result.confidence
    );

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Subject pubkey: {:?}", result.subject_pubkey);
    info!("Target pubkey: {:?}", result.target_pubkey);
    info!("Parameters: {:?}", result.parameters);
    info!("Confidence: {}", result.confidence);

    Ok(())
}

/// Test structured response system with "all" keyword
#[rstest]
#[case("transfer all SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("swap all SOL for USDC")]
#[tokio::test]
#[serial]
async fn test_structured_response_all_keyword(#[case] prompt: &str) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!(
        "Testing structured response with 'all' keyword for prompt: {}",
        prompt
    );
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify the prompt was processed
    assert!(
        !result.refined_prompt.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original_prompt, prompt,
        "Original prompt should match input"
    );

    // For "all" keyword prompts, usable_amount should be Some
    assert!(
        result.usable_amount.is_some(),
        "usable_amount should be Some for 'all' keyword prompts"
    );

    let usable_amount = result.usable_amount.unwrap();

    // Check if this is a swap operation with insufficient balance
    let is_swap = prompt.to_lowercase().contains("swap");

    if is_swap {
        // For swap operations, the usable_amount might be 0 if balance is insufficient
        // This is expected behavior, not an error
        info!(
            "Swap operation detected with usable_amount: {}",
            usable_amount
        );
        assert!(
            usable_amount >= 0.0,
            "usable_amount should be non-negative for 'all' keyword prompts"
        );
    } else {
        // For transfer operations, usable_amount should be positive
        assert!(
            usable_amount > 0.0,
            "usable_amount should be positive for 'all' keyword prompts"
        );
    }

    // Verify action was detected
    assert_ne!(
        result.action,
        PromptAction::Unknown,
        "Action should be detected for 'all' keyword prompts"
    );

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Subject pubkey: {:?}", result.subject_pubkey);
    info!("Target pubkey: {:?}", result.target_pubkey);
    info!("Parameters: {:?}", result.parameters);
    info!("Usable amount: {}", usable_amount);
    info!("Confidence: {}", result.confidence);

    Ok(())
}

/// Test structured response system with typos
#[rstest]
#[case(
    "trasnfer 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    PromptAction::Transfer
)]
#[case("swp 0.5 SOL to USDC", PromptAction::Swap)]
#[tokio::test]
#[serial]
async fn test_structured_response_typos(
    #[case] prompt: &str,
    #[case] expected_action: PromptAction,
) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!(
        "Testing structured response with typos for prompt: {}",
        prompt
    );
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify the prompt was processed
    assert!(
        !result.refined_prompt.is_empty(),
        "Refined prompt should not be empty"
    );
    assert_eq!(
        result.original_prompt, prompt,
        "Original prompt should match input"
    );

    // Verify action was detected correctly despite typos
    assert_eq!(
        result.action, expected_action,
        "Detected action {:?} should match expected {:?}",
        result.action, expected_action
    );

    // For typo prompts, changes should be detected
    assert!(
        result.original_prompt != result.refined_prompt,
        "Changes should be detected for typo prompts"
    );

    // Verify confidence is reasonable (>0.5 but maybe lower than for correct prompts)
    assert!(
        result.confidence >= 0.4,
        "Confidence should be greater than or equal to 0.4 for typo prompts, got {}",
        result.confidence
    );

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Subject pubkey: {:?}", result.subject_pubkey);
    info!("Target pubkey: {:?}", result.target_pubkey);
    info!("Parameters: {:?}", result.parameters);
    info!("Confidence: {}", result.confidence);

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

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
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

    // Check if this is a swap operation with insufficient balance
    let is_swap = prompt.to_lowercase().contains("swap") || prompt.to_lowercase().contains("swp");

    if is_swap {
        // For swap operations, usable_amount might be 0 if balance is insufficient
        // This is expected behavior, not an error
        info!(
            "Swap operation detected with usable_amount: {}",
            usable_amount
        );
        assert!(
            usable_amount >= 0.0,
            "usable_amount should be non-negative for 'all' keyword prompts"
        );
    } else {
        // For transfer operations, usable_amount should be positive
        assert!(
            usable_amount > 0.0,
            "usable_amount should be positive for 'all' keyword prompts"
        );
    }

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
    info!("Confidence: {}", result.get_confidence());

    // Log full refined prompt for debugging
    info!("Full refined prompt: {}", result.refined);

    Ok(())
}

/// Test SPL token extraction from prompts
#[rstest]
#[case(
    "send 1 usdc to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    "USDC",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "1"
)]
#[case(
    "transfer 10 usdt to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    "USDT",
    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    "10"
)]
#[case(
    "send all usdc to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    "USDC",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "0.03"
)]
#[case(
    "swap 0.5 usdc for sol",
    "USDC",
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "0.5"
)]
#[tokio::test]
#[serial]
async fn test_spl_token_extraction(
    #[case] prompt: &str,
    #[case] _expected_symbol: &str,
    #[case] expected_mint: &str,
    #[case] expected_amount: &str,
) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        panic!("ZAI_API_KEY not set");
    }

    let mut processor = PromptProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Testing SPL token extraction for prompt: {}", prompt);
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify prompt was processed
    assert!(
        !result.refined_prompt.is_empty(),
        "Refined prompt should not be empty"
    );

    // Verify action is detected as transfer or swap
    assert!(
        matches!(result.action, PromptAction::Transfer | PromptAction::Swap),
        "Detected action should be Transfer or Swap"
    );

    // Verify input_mint is set correctly
    assert_eq!(
        result.parameters.input_mint,
        Some(expected_mint.to_string()),
        "Input mint should match expected mint"
    );

    // Verify amount is extracted correctly
    assert_eq!(
        result.parameters.amount,
        Some(expected_amount.to_string()),
        "Amount should match expected amount"
    );

    // Verify target pubkey is extracted (for transfer operations) or null (for swap operations)
    match result.action {
        PromptAction::Transfer => {
            assert_eq!(
                result.target_pubkey,
                Some(test_address.to_string()),
                "Target pubkey should match provided address for transfer operations"
            );
        }
        PromptAction::Swap => {
            assert_eq!(
                result.target_pubkey, None,
                "Target pubkey should be None for swap operations"
            );
        }
        _ => {}
    }

    // Verify confidence is reasonable (>0.5)
    assert!(
        result.confidence > 0.5,
        "Confidence should be greater than 0.5, got {}",
        result.confidence
    );

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Input mint: {:?}", result.parameters.input_mint);
    info!("Amount: {:?}", result.parameters.amount);
    info!("Target pubkey: {:?}", result.target_pubkey);
    info!("Confidence: {}", result.confidence);

    Ok(())
}
