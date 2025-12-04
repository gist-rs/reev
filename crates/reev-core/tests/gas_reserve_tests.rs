//! Comprehensive tests for the gas reserve module
//!
//! This module tests the gas reserve calculation functionality to ensure
//! consistent behavior across all components of the Reev system.

use reev_core::gas_reserve::*;
use reev_core::prompt_processor::types::PromptAction;
use reev_types::flow::TokenBalance;
use std::collections::HashMap;

#[tokio::test]
async fn test_gas_reserve_constants() {
    // Verify gas reserve constants are correctly defined
    assert_eq!(TRANSFER_GAS_RESERVE, 1_000_000); // 0.001 SOL
    assert_eq!(SWAP_GAS_RESERVE, 5_000_000); // 0.005 SOL
    assert_eq!(DEFAULT_GAS_RESERVE, 2_000_000); // 0.002 SOL
}

#[tokio::test]
async fn test_get_gas_reserve_for_action() {
    // Test all action types return correct gas reserve values
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Transfer),
        TRANSFER_GAS_RESERVE
    );
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Swap),
        SWAP_GAS_RESERVE
    );
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Lend),
        DEFAULT_GAS_RESERVE
    );
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Borrow),
        DEFAULT_GAS_RESERVE
    );
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Earn),
        DEFAULT_GAS_RESERVE
    );
    assert_eq!(
        get_gas_reserve_for_action(PromptAction::Unknown),
        DEFAULT_GAS_RESERVE
    );
}

#[tokio::test]
async fn test_get_gas_reserve_for_action_human() {
    // Test human-readable gas reserve values
    assert_eq!(
        get_gas_reserve_for_action_human(PromptAction::Transfer),
        "0.001 SOL".to_string()
    );
    assert_eq!(
        get_gas_reserve_for_action_human(PromptAction::Swap),
        "0.005 SOL".to_string()
    );
    assert_eq!(
        get_gas_reserve_for_action_human(PromptAction::Lend),
        "0.002 SOL".to_string()
    );
}

#[tokio::test]
async fn test_calculate_max_sol_transferable() {
    // Normal case with sufficient balance
    let balance = 10_000_000_000; // 10 SOL
    let gas_reserve = 1_000_000; // 0.001 SOL
    assert_eq!(
        calculate_max_sol_transferable(balance, Some(gas_reserve)),
        9_999_000_000
    );

    // Balance less than gas reserve
    let balance = 500_000; // 0.0005 SOL
    let gas_reserve = 1_000_000; // 0.001 SOL
    assert_eq!(
        calculate_max_sol_transferable(balance, Some(gas_reserve)),
        0
    );

    // Balance exactly equal to gas reserve
    let balance = 1_000_000; // 0.001 SOL
    let gas_reserve = 1_000_000; // 0.001 SOL
    assert_eq!(
        calculate_max_sol_transferable(balance, Some(gas_reserve)),
        0
    );

    // Default gas reserve (None)
    let balance = 10_000_000_000; // 10 SOL
    assert_eq!(calculate_max_sol_transferable(balance, None), 9_999_000_000);

    // Very large balance
    let balance = 1_000_000_000_000; // 1000 SOL
    let gas_reserve = 5_000_000; // 0.005 SOL
    assert_eq!(
        calculate_max_sol_transferable(balance, Some(gas_reserve)),
        999_995_000_000
    );
}

#[tokio::test]
async fn test_calculate_max_spl_transferable() {
    // SPL tokens can transfer entire balance
    let balance = 10_000_000; // 10 USDC
    assert_eq!(
        calculate_max_spl_transferable(balance, Some(1_000_000)),
        10_000_000
    );

    // Zero balance
    let balance = 0;
    assert_eq!(calculate_max_spl_transferable(balance, Some(1_000_000)), 0);

    // Different gas reserve values shouldn't affect SPL token transfer
    let balance = 5_000_000; // 5 USDC
    assert_eq!(
        calculate_max_spl_transferable(balance, Some(5_000_000)),
        5_000_000
    );
}

#[tokio::test]
async fn test_insufficient_balance_error() {
    // Test error message formatting
    let error = insufficient_balance_error(1_000_000_000, 1_000_000);
    assert!(error.contains("Insufficient balance"));
    assert!(error.contains("1 SOL")); // Changed from "1.0 SOL" to "1 SOL"
    assert!(error.contains("0.001 SOL"));

    let error = insufficient_balance_error(500_000_000, 1_000_000);
    assert!(error.contains("0.5 SOL"));
    assert!(error.contains("0.001 SOL"));
}

#[tokio::test]
async fn test_calculate_amount_for_all_keyword() {
    // Create mock token balances
    let mut token_balances = HashMap::new();
    token_balances.insert(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        TokenBalance {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            balance: 10_000_000, // 10 USDC
            decimals: Some(6),
            symbol: Some("USDC".to_string()),
            formatted_amount: Some("10.0 USDC".to_string()),
            owner: None,
        },
    );
    token_balances.insert(
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
        TokenBalance {
            mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            balance: 5_000_000, // 5 USDT
            decimals: Some(6),
            symbol: Some("USDT".to_string()),
            formatted_amount: Some("5.0 USDT".to_string()),
            owner: None,
        },
    );

    // Test with transfer action (uses TRANSFER_GAS_RESERVE)
    let sol_balance = 5_000_000_000; // 5 SOL
    let result =
        calculate_amount_for_all_keyword(PromptAction::Transfer, sol_balance, &token_balances);

    assert_eq!(result.get("SOL"), Some(&4.999));
    assert_eq!(
        result.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        Some(&10.0)
    );
    assert_eq!(
        result.get("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        Some(&5.0)
    );

    // Test with swap action (uses SWAP_GAS_RESERVE)
    let sol_balance = 5_000_000_000; // 5 SOL
    let result = calculate_amount_for_all_keyword(PromptAction::Swap, sol_balance, &token_balances);

    assert_eq!(result.get("SOL"), Some(&4.995)); // 5 SOL - 0.005 SOL

    // Test with insufficient SOL balance
    let sol_balance = 500_000; // 0.0005 SOL (less than TRANSFER_GAS_RESERVE)
    let result =
        calculate_amount_for_all_keyword(PromptAction::Transfer, sol_balance, &token_balances);

    assert_eq!(result.get("SOL"), Some(&0.0));
}

#[tokio::test]
async fn test_edge_cases() {
    // Test with zero balance
    assert_eq!(calculate_max_sol_transferable(0, Some(1_000_000)), 0);

    // Test with very small gas reserve
    assert_eq!(
        calculate_max_sol_transferable(1_000_000, Some(100)),
        999_900
    );

    // Test with gas reserve of zero
    assert_eq!(
        calculate_max_sol_transferable(1_000_000, Some(0)),
        1_000_000
    );
}

#[tokio::test]
async fn test_compatibility_with_existing_code() {
    // Test that our new module works with existing MaxAmountCalculator
    let calculator = reev_core::prompt_processor::MaxAmountCalculator::new();

    // Verify fee values match our constants
    assert_eq!(
        calculator.get_fee(&PromptAction::Transfer),
        TRANSFER_GAS_RESERVE
    );
    assert_eq!(calculator.get_fee(&PromptAction::Swap), SWAP_GAS_RESERVE);
    assert_eq!(calculator.get_fee(&PromptAction::Lend), DEFAULT_GAS_RESERVE);

    // Test transfer utils integration
    let balance = 5_000_000_000; // 5 SOL
    let gas_reserve = 1_000_000; // 0.001 SOL
    assert_eq!(
        reev_core::utils::transfer_utils::calculate_max_transferable_amount(
            "",
            balance,
            gas_reserve
        ),
        4_999_000_000
    );
}
