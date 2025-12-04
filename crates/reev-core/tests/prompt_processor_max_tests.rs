//! Comprehensive tests for max amount calculation system

use anyhow::Result;
use reev_core::prompt_processor::max_amount_calculator::{MaxAmounts, TokenAmounts};
use reev_core::prompt_processor::{
    MaxAmountCalculator, PromptAction, PromptParameters, StructuredProcessor,
    StructuredRefineRequest, StructuredRefinedPrompt,
};
use rstest::*;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use tracing::info;

// Initialize tracing for all tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt::try_init();
}

// Load environment variables from .env file
fn setup_env() {
    dotenvy::dotenv().ok();
}

#[test]
fn test_max_amounts_serialization() {
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
}

#[test]
fn test_get_max_amount_for_action_token() {
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
}

#[test]
fn test_validate_amount_for_action() {
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
}

#[test]
fn test_max_amount_calculator_fees() {
    let calculator = MaxAmountCalculator::new();

    // Check default fees
    assert_eq!(calculator.get_fee(&PromptAction::Transfer), 1_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Swap), 5_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Lend), 2_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Borrow), 2_000_000);
    assert_eq!(calculator.get_fee(&PromptAction::Earn), 2_000_000);
}

#[test]
fn test_max_amount_calculator_token_mints() {
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
}

#[test]
fn test_format_as_yml_prompt() {
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
}

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
    init_tracing();
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        // Skip test if API key not available
        return Ok(());
    }

    let processor = StructuredProcessor::new();
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

    // Log results for inspection
    info!("Original: {}", result.original_prompt);
    info!("Refined: {}", result.refined_prompt);
    info!("Action: {:?}", result.action);
    info!("Usable amount: {:?}", result.usable_amount);

    Ok(())
}

#[rstest]
#[case("transfer all SOL", "transfer", "SOL")]
#[case("swap all USDC for SOL", "swap", "USDC")]
#[case("lend all SOL", "lend", "SOL")]
#[tokio::test]
#[serial]
async fn test_correct_fee_application(
    #[case] prompt: &str,
    #[case] action: &str,
    #[case] token: &str,
) -> Result<()> {
    init_tracing();
    setup_env();

    // Check for ZAI_API_KEY
    if env::var("ZAI_API_KEY").is_err() {
        // Skip test if API key not available
        return Ok(());
    }

    let processor = StructuredProcessor::new();
    let test_address = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

    info!("Testing fee application for prompt: {}", prompt);
    let result = processor
        .process_prompt_structured(prompt, test_address)
        .await?;

    // Get max amounts
    let max_amounts = MaxAmountCalculator::calculate_all_max_amounts(test_address).await?;
    let max_amounts_yml = MaxAmountCalculator::format_as_yml_prompt(&max_amounts)?;
    let parsed_max_amounts = MaxAmountCalculator::parse_from_yml(&max_amounts_yml)?;

    // Get the max amount for this action and token
    let max_amount =
        MaxAmountCalculator::get_max_amount_for_action_token(&parsed_max_amounts, action, token)
            .unwrap_or(0.0);

    // Verify usable amount is within expected range
    if let Some(usable_amount) = result.usable_amount {
        assert!(usable_amount > 0.0, "usable_amount should be positive");
        assert!(
            usable_amount <= max_amount,
            "usable_amount should not exceed max_amount"
        );

        // Log values for inspection
        info!(
            "Action: {}, Token: {}, Usable: {}, Max: {}",
            action, token, usable_amount, max_amount
        );
    }

    Ok(())
}

#[test]
fn test_structured_refine_request_serialization() {
    let request = StructuredRefineRequest {
        prompt: "swap all SOL for USDC".to_string(),
        owner_wallet_address: Some("gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string()),
        max_amount: Some(10.5),
        max_amounts_yml: Some("max_amounts:\n  swap:\n    SOL: 10.3\n    USDC: 1000.0".to_string()),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&request).unwrap();

    // Parse back from JSON
    let parsed: StructuredRefineRequest = serde_json::from_str(&json_str).unwrap();

    assert_eq!(request.prompt, parsed.prompt);
    assert_eq!(request.owner_wallet_address, parsed.owner_wallet_address);
    assert_eq!(request.max_amount, parsed.max_amount);
    assert_eq!(request.max_amounts_yml, parsed.max_amounts_yml);
}

#[test]
fn test_structured_refined_prompt_builder() {
    let prompt = StructuredRefinedPrompt::builder()
        .refined_prompt("transfer 5.0 SOL to address".to_string())
        .action(PromptAction::Transfer)
        .subject_pubkey(Some(
            "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string(),
        ))
        .target_pubkey(Some("11111111111111111111111111111112".to_string()))
        .parameters(PromptParameters {
            amount: Some("5.0".to_string()),
            input_mint: Some("So11111111111111111111111111111111111111112".to_string()),
            output_mint: None,
            additional: HashMap::new(),
        })
        .confidence(0.9)
        .original_prompt("send all SOL to address".to_string())
        .usable_amount(Some(5.0))
        .build();

    assert_eq!(prompt.refined_prompt, "transfer 5.0 SOL to address");
    assert_eq!(prompt.action, PromptAction::Transfer);
    assert_eq!(
        prompt.subject_pubkey,
        Some("gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq".to_string())
    );
    assert_eq!(
        prompt.target_pubkey,
        Some("11111111111111111111111111111112".to_string())
    );
    assert_eq!(prompt.parameters.amount, Some("5.0".to_string()));
    assert_eq!(prompt.confidence, 0.9);
    assert_eq!(prompt.original_prompt, "send all SOL to address");
    assert_eq!(prompt.usable_amount, Some(5.0));
}
