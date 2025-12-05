//! Standardized Gas Reserve Module
//!
//! This module provides a centralized location for gas reserve calculations
//! across all components of the Reev system. It ensures consistent gas
//! reserve values and handling for different operation types.
//!
//! This addresses the issue where gas reserve calculations were implemented
//! inconsistently across multiple components, leading to potential user
//! experience issues and difficult maintenance.

use crate::prompt_processor::types::PromptAction;
use reev_types::flow::TokenBalance;
use std::collections::HashMap;

/// Gas reserve constants based on actual transaction costs
pub const TRANSFER_GAS_RESERVE: u64 = 1_000_000; // 0.001 SOL for transfers
pub const SWAP_GAS_RESERVE: u64 = 5_000_000; // 0.005 SOL for Jupiter swaps
pub const DEFAULT_GAS_RESERVE: u64 = 2_000_000; // 0.002 SOL for other operations

/// Get gas reserve for a specific action type
pub fn get_gas_reserve_for_action(action: PromptAction) -> u64 {
    match action {
        PromptAction::Transfer => TRANSFER_GAS_RESERVE,
        PromptAction::Swap => SWAP_GAS_RESERVE,
        PromptAction::Lend | PromptAction::Borrow | PromptAction::Earn => DEFAULT_GAS_RESERVE,
        PromptAction::Unknown => DEFAULT_GAS_RESERVE,
    }
}

/// Calculate max transferable amount for SOL
pub fn calculate_max_sol_transferable(balance: u64, gas_reserve: Option<u64>) -> u64 {
    let gas_reserve = gas_reserve.unwrap_or(TRANSFER_GAS_RESERVE);

    // Return 0 if balance is less than or equal to gas reserve
    if balance <= gas_reserve {
        return 0;
    }

    // Return balance minus gas reserve
    balance - gas_reserve
}

/// Calculate max transferable amount for SPL tokens
pub fn calculate_max_spl_transferable(balance: u64, _gas_reserve: Option<u64>) -> u64 {
    // For SPL tokens, gas is paid in SOL, not the token itself
    // So we can transfer the entire balance (except for minimum rent exemption if needed)
    // For now, we'll just return the current amount as is
    // TODO: Consider minimum balance for token accounts if needed
    balance
}

/// Calculate max amount for "all" keyword handling
pub fn calculate_amount_for_all_keyword(
    action: PromptAction,
    sol_balance: u64,
    token_balances: &HashMap<String, TokenBalance>,
) -> HashMap<String, f64> {
    let mut result = HashMap::new();

    // Calculate max SOL amount
    let gas_reserve = get_gas_reserve_for_action(action);
    let max_sol = calculate_max_sol_transferable(sol_balance, Some(gas_reserve));
    result.insert("SOL".to_string(), max_sol as f64 / 1_000_000_000.0);

    // Calculate max amounts for SPL tokens
    for (mint_address, token_balance) in token_balances {
        let max_amount = calculate_max_spl_transferable(token_balance.balance, None);
        result.insert(mint_address.clone(), max_amount as f64 / 1_000_000.0);
    }

    result
}

/// Standardized error message for insufficient balance
pub fn insufficient_balance_error(balance: u64, gas_reserve: u64) -> String {
    format!(
        "Insufficient balance for transfer. Balance: {} SOL, required reserve: {} SOL",
        balance as f64 / 1_000_000_000.0,
        gas_reserve as f64 / 1_000_000_000.0
    )
}

/// Get gas reserve for a specific action type as a human-readable string
pub fn get_gas_reserve_for_action_human(action: PromptAction) -> String {
    let reserve = get_gas_reserve_for_action(action);
    format!("{} SOL", reserve as f64 / 1_000_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt_processor::types::PromptAction;

    #[test]
    fn test_get_gas_reserve_for_action() {
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

    #[test]
    fn test_calculate_max_sol_transferable() {
        // Normal case
        assert_eq!(
            calculate_max_sol_transferable(5_000_000_000, Some(1_000_000)),
            4_999_000_000
        );

        // Balance less than gas reserve
        assert_eq!(calculate_max_sol_transferable(500_000, Some(1_000_000)), 0);

        // Balance exactly equal to gas reserve
        assert_eq!(
            calculate_max_sol_transferable(1_000_000, Some(1_000_000)),
            0
        );

        // Default gas reserve
        assert_eq!(
            calculate_max_sol_transferable(5_000_000_000, None),
            4_999_000_000
        );
    }

    #[test]
    fn test_calculate_max_spl_transferable() {
        // SPL tokens can transfer entire balance
        assert_eq!(
            calculate_max_spl_transferable(1_000_000, Some(1_000_000)),
            1_000_000
        );

        assert_eq!(calculate_max_spl_transferable(0, Some(1_000_000)), 0);
    }

    #[test]
    fn test_insufficient_balance_error() {
        let error = insufficient_balance_error(1_000_000, 1_000_000);
        assert!(error.contains("Insufficient balance"));
        assert!(error.contains("0.001 SOL"));
        assert!(error.contains("0.001 SOL"));
    }

    #[test]
    fn test_calculate_amount_for_all_keyword() {
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

        let result = calculate_amount_for_all_keyword(
            PromptAction::Transfer,
            5_000_000_000, // 5 SOL
            &token_balances,
        );

        assert_eq!(result.get("SOL"), Some(&4.999));
        assert_eq!(
            result.get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            Some(&10.0)
        );
    }
}
