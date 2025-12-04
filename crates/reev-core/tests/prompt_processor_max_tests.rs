//! Tests for Max Amount Calculator module
//!
//! These tests verify that MaxAmountCalculator correctly calculates max amounts
//! for all action types and handles structured prompts with "all" keyword.
//! The tests focus on max amount calculation logic without making surfpool calls.

use anyhow::{anyhow, Result};
use reev_core::prompt_processor::max_amount_calculator::{MaxAmounts, TokenAmounts};
use reev_core::prompt_processor::{
    MaxAmountCalculator, PromptAction, PromptParameters, StructuredProcessor,
    StructuredRefineRequest, StructuredRefineResponse,
};
use rstest::*;
use serial_test::serial;
use std::collections::HashMap;
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
    // Try to find any number followed by a space and then a token name
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)\s+(SOL|USDC|USDT)")?;
    if let Some(captures) = re.captures(refined) {
        if let Some(amount_str) = captures.get(1) {
            return Ok(amount_str.as_str().parse::<f64>()?);
        }
    }

    // Special case for "0 SOL", "0 USDC", or "0 USDT" which indicates insufficient balance
    let refined_lower = refined.to_lowercase();
    if refined_lower.contains("0 sol")
        || refined_lower.contains("0 usdc")
        || refined_lower.contains("0 usdt")
    {
        return Ok(0.0);
    }

    Err(anyhow!(
        "Could not extract amount from refined prompt: {refined}"
    ))
}

/// Test max amounts serialization and deserialization
#[test]
fn test_max_amounts_serialization() -> Result<()> {
    // Create test max amounts
    let mut max_amounts_map = HashMap::new();
    let mut sol_amounts = HashMap::new();
    sol_amounts.insert("SOL".to_string(), 10.5);
    sol_amounts.insert("USDC".to_string(), 1000.0);

    max_amounts_map.insert(
        "transfer".to_string(),
        TokenAmounts {
            amounts: sol_amounts,
        },
    );

    let max_amounts = MaxAmounts {
        max_amounts: max_amounts_map,
    };

    // Serialize to YAML
    let yml_str = serde_yaml::to_string(&max_amounts).unwrap();

    // Parse back from YAML
    let parsed = serde_yaml::from_str::<MaxAmounts>(&yml_str).unwrap();

    assert_eq!(max_amounts, parsed);

    Ok(())
}

/// Test getting max amount for a specific action and token
#[test]
fn test_get_max_amount_for_action_token() -> Result<()> {
    let mut max_amounts_map = HashMap::new();
    let mut sol_amounts = HashMap::new();
    sol_amounts.insert("SOL".to_string(), 10.5);
    sol_amounts.insert("USDC".to_string(), 1000.0);

    max_amounts_map.insert(
        "transfer".to_string(),
        TokenAmounts {
            amounts: sol_amounts,
        },
    );

    let max_amounts = MaxAmounts {
        max_amounts: max_amounts_map,
    };

    assert_eq!(
        MaxAmountCalculator::get_max_amount_for_action_token(&max_amounts, "transfer", "SOL"),
        Some(10.5)
    );
    assert_eq!(
        MaxAmountCalculator::get_max_amount_for_action_token(&max_amounts, "transfer", "USDC"),
        Some(1000.0)
    );
    assert_eq!(
        MaxAmountCalculator::get_max_amount_for_action_token(&max_amounts, "swap", "SOL"),
        None
    );

    Ok(())
}

/// Test validation of amounts against max limits
#[test]
fn test_validate_amount_for_action() -> Result<()> {
    let mut max_amounts_map = HashMap::new();
    let mut sol_amounts = HashMap::new();
    sol_amounts.insert("SOL".to_string(), 10.5);

    max_amounts_map.insert(
        "transfer".to_string(),
        TokenAmounts {
            amounts: sol_amounts,
        },
    );

    let max_amounts = MaxAmounts {
        max_amounts: max_amounts_map,
    };

    // Valid amount should pass
    assert!(
        MaxAmountCalculator::validate_amount_for_action(&max_amounts, "transfer", "SOL", 5.0)
            .is_ok()
    );

    // Amount equal to max should pass
    assert!(
        MaxAmountCalculator::validate_amount_for_action(&max_amounts, "transfer", "SOL", 10.5)
            .is_ok()
    );

    // Amount exceeding max should fail
    assert!(
        MaxAmountCalculator::validate_amount_for_action(&max_amounts, "transfer", "SOL", 15.0)
            .is_err()
    );

    // Non-existent token should fail
    assert!(
        MaxAmountCalculator::validate_amount_for_action(&max_amounts, "transfer", "USDC", 5.0)
            .is_err()
    );

    Ok(())
}

/// Test max amount calculator fees
#[test]
fn test_max_amount_calculator_fees() -> Result<()> {
    let calculator = MaxAmountCalculator::new();

    // Check default fees
    assert_eq!(calculator.get_fee(&PromptAction::Transfer), 1_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Swap), 5_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Lend), 2_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Borrow), 2_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Earn), 2_000_000);

    Ok(())
}

/// Test max amount calculator token mints
#[test]
fn test_max_amount_calculator_token_mints() -> Result<()> {
    let calculator = MaxAmountCalculator::new();

    // Check token mints
    assert_eq!(
        calculator.get_token_mint("SOL"),
        Some(&"So11111111111111111111111111111111111111112".to_string())
    );
    assert_eq!(
        calculator.get_token_mint("USDC"),
        Some(&"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string())
    );
    assert_eq!(
        calculator.get_token_mint("USDT"),
        Some(&"Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string())
    );
    assert_eq!(calculator.get_token_mint("UNKNOWN"), None);

    Ok(())
}

/// Test formatting max amounts as YML prompt
#[test]
fn test_format_as_yml_prompt() -> Result<()> {
    let mut max_amounts = HashMap::new();

    // Add transfer max amounts
    let mut transfer_amounts = HashMap::new();
    transfer_amounts.insert("SOL".to_string(), 10.5);
    transfer_amounts.insert("USDC".to_string(), 1000.0);

    max_amounts.insert(PromptAction::Transfer, transfer_amounts);

    // Add swap max amounts
    let mut swap_amounts = HashMap::new();
    swap_amounts.insert("SOL".to_string(), 10.3);
    swap_amounts.insert("USDC".to_string(), 1000.0);

    max_amounts.insert(PromptAction::Swap, swap_amounts);

    // Convert to calculations
    let calculations = max_amounts
        .into_iter()
        .map(
            |(action, max_amounts)| reev_core::prompt_processor::MaxAmountCalculation {
                action,
                max_amounts,
            },
        )
        .collect::<Vec<_>>();

    // Format as YML
    let yml_str = MaxAmountCalculator::format_as_yml_prompt(&calculations).unwrap();

    // Verify structure
    assert!(yml_str.contains("max_amounts:"));
    assert!(yml_str.contains("transfer:"));
    assert!(yml_str.contains("swap:"));
    assert!(yml_str.contains("SOL: 10.5"));
    assert!(yml_str.contains("USDC: 1000.0"));

    // Parse back to verify correctness
    let parsed = MaxAmountCalculator::parse_from_yml(&yml_str).unwrap();
    assert!(parsed.max_amounts.contains_key("transfer"));
    assert!(parsed.max_amounts.contains_key("swap"));

    let transfer = parsed.max_amounts.get("transfer").unwrap();
    assert_eq!(transfer.amounts.get("SOL"), Some(&10.5));
    assert_eq!(transfer.amounts.get("USDC"), Some(&1000.0));

    let swap = parsed.max_amounts.get("swap").unwrap();
    assert_eq!(swap.amounts.get("SOL"), Some(&10.3));
    assert_eq!(swap.amounts.get("USDC"), Some(&1000.0));

    Ok(())
}

/// Test structured refined prompt builder
#[test]
fn test_structured_refined_prompt_builder() -> Result<()> {
    let parameters = PromptParameters {
        amount: Some("10.5".to_string()),
        input_mint: Some("So11111111111111111111111111111111111111112".to_string()),
        output_mint: None,
        additional: HashMap::new(),
    };

    let response = StructuredRefineResponse {
        refined_prompt: "transfer 10.5 SOL to address".to_string(),
        action: "transfer".to_string(),
        subject_pubkey: None,
        target_pubkey: Some("gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()),
        parameters,
        confidence: 0.95,
    };

    let structured_prompt = response
        .to_structured_prompt("transfer all SOL to address".to_string(), Some(10.5))
        .unwrap();

    assert_eq!(
        structured_prompt.original_prompt,
        "transfer all SOL to address"
    );
    assert_eq!(
        structured_prompt.refined_prompt,
        "transfer 10.5 SOL to address"
    );
    assert_eq!(structured_prompt.action, PromptAction::Transfer);
    assert_eq!(
        structured_prompt.target_pubkey,
        Some("gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string())
    );
    assert_eq!(structured_prompt.usable_amount, Some(10.5));
    assert_eq!(structured_prompt.confidence, 0.95);

    Ok(())
}

/// Test structured refine request serialization
#[test]
fn test_structured_refine_request_serialization() -> Result<()> {
    let request = StructuredRefineRequest {
        prompt: "transfer all SOL to address".to_string(),
        owner_wallet_address: Some("gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()),
        max_amount: None,
        max_amounts_yml: Some(
            "max_amounts:\n  transfer:\n    SOL: 10.5\n    USDC: 1000.0".to_string(),
        ),
    };

    let json = serde_json::to_string(&request)?;
    let parsed: StructuredRefineRequest = serde_json::from_str(&json)?;

    assert_eq!(request.prompt, parsed.prompt);
    assert_eq!(request.owner_wallet_address, parsed.owner_wallet_address);
    assert_eq!(request.max_amount, parsed.max_amount);
    assert_eq!(request.max_amounts_yml, parsed.max_amounts_yml);

    Ok(())
}

/// Test structured max amounts for different action types
#[rstest]
#[case("transfer all SOL", PromptAction::Transfer)]
#[case("swap all SOL for USDC", PromptAction::Swap)]
#[case("lend all USDC", PromptAction::Lend)]
#[case("borrow all USDT", PromptAction::Borrow)]
#[tokio::test]
#[serial]
async fn test_structured_max_amounts_by_action(
    #[case] prompt: &str,
    #[case] expected_action: PromptAction,
) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    let api_key = match env::var("ZAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            // Skip test if API key not available
            println!("Skipping test - ZAI_API_KEY not available");
            return Ok(());
        }
    };

    let processor = StructuredProcessor::with_config(api_key, "glm-4.6-coding".to_string());
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Testing structured response for prompt: {}", prompt);
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify action was detected correctly
    assert_eq!(result.action, expected_action);

    // Verify usable amount is set for "all" keyword
    assert!(
        result.usable_amount.is_some(),
        "usable_amount should be set for 'all' keyword"
    );

    // Extract amount from refined prompt for verification
    let refined_amount = extract_amount_from_refined_prompt(&result.refined_prompt)?;

    // Verify amount in refined prompt matches usable_amount
    if let Some(usable_amount) = result.usable_amount {
        assert!(
            (refined_amount - usable_amount).abs() < 0.000001,
            "Amount in refined prompt ({refined_amount}) should match usable_amount ({usable_amount})"
        );
    }

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Usable amount: {:?}", result.usable_amount);
    info!("Refined amount: {}", refined_amount);

    Ok(())
}

/// Test correct fee application for different actions
#[rstest]
#[case("transfer all SOL", "transfer", "SOL")]
#[case("swap all USDC for SOL", "swap", "USDC")]
#[case("lend all SOL", "lend", "SOL")]
#[tokio::test]
#[serial]
async fn test_correct_fee_application(
    #[case] prompt: &str,
    #[case] action_str: &str,
    #[case] token: &str,
) -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load environment variables
    setup_env();

    // Check for ZAI_API_KEY
    let api_key = match env::var("ZAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            // Skip test if API key not available
            println!("Skipping test - ZAI_API_KEY not available");
            return Ok(());
        }
    };

    let processor = StructuredProcessor::with_config(api_key, "glm-4.6-coding".to_string());
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Testing fee application for prompt: {}", prompt);
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Verify action was detected correctly
    let expected_action = match action_str {
        "transfer" => PromptAction::Transfer,
        "swap" => PromptAction::Swap,
        "lend" => PromptAction::Lend,
        "borrow" => PromptAction::Borrow,
        "earn" => PromptAction::Earn,
        _ => PromptAction::Unknown,
    };
    assert_eq!(result.action, expected_action);

    // Verify usable amount is set for "all" keyword
    assert!(
        result.usable_amount.is_some(),
        "usable_amount should be set for 'all' keyword"
    );

    // Calculate expected max amount based on action type and fee
    let calculator = MaxAmountCalculator::new();
    let fee = calculator.get_fee(&result.action);

    let expected_max = match token {
        "SOL" => {
            // For SOL, subtract fee from balance (1 SOL = 1_000_000_000 lamports)
            let balance = 1_000_000_000; // 1 SOL test balance
            (balance - fee) as f64 / 1_000_000_000.0
        }
        "USDC" | "USDT" => {
            // For USDC/USDT, use default test balance (no fee for tokens)
            100.0
        }
        _ => 0.0,
    };

    // Extract amount from refined prompt for verification
    let refined_amount = extract_amount_from_refined_prompt(&result.refined_prompt)?;

    // Verify amount in refined prompt matches usable_amount
    if let Some(usable_amount) = result.usable_amount {
        assert!(
            (refined_amount - usable_amount).abs() < 0.000001,
            "Amount in refined prompt ({refined_amount}) should match usable_amount ({usable_amount})"
        );
    }

    // Verify amount is reasonable (within 10% of expected)
    let actual_amount = result.usable_amount.unwrap_or(0.0);
    let tolerance = expected_max * 0.1;
    assert!(
        (actual_amount - expected_max).abs() < tolerance,
        "Amount {actual_amount} is not close to expected {expected_max} for {action_str} {token}"
    );

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Expected max: {}", expected_max);
    info!("Actual amount: {}", actual_amount);
    info!("Refined amount: {}", refined_amount);
    info!("Fee: {}", fee);

    Ok(())
}
