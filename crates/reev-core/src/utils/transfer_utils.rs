//! Transfer utilities for calculating maximum transferable amounts
//!
//! This module provides utilities for calculating the maximum amount of tokens
//! that can be transferred while reserving enough for transaction fees.

/// Calculate the maximum amount that can be transferred while reserving gas fees
///
/// This function determines how much of a token balance can be transferred while
/// ensuring enough lamports remain for transaction fees and potential account
/// creation costs. It uses the standardized gas reserve calculations from the
/// gas_reserve module for consistency across the system.
///
/// # Parameters
///
/// * `mint_address` - Token mint address to identify token type (SOL vs SPL)
/// * `current_amount` - Current balance in lamports
/// * `gas_amount_lamport` - Amount of lamports to reserve for transaction fees
///
/// # Returns
///
/// The maximum amount in lamports that can be transferred
///
/// # Examples
///
/// ```
/// use reev_core::utils::transfer_utils::calculate_max_transferable_amount;
///
/// // For SOL transfer with 5 SOL balance and 0.001 SOL gas reserve
/// let current_amount = 5_000_000_000; // 5 SOL in lamports
/// let gas_reserve = 1_000_000; // 0.001 SOL in lamports
/// let max_amount = calculate_max_transferable_amount("", current_amount, gas_reserve);
/// assert_eq!(max_amount, 4_999_000_000); // 4.999 SOL in lamports
///
/// // For SOL transfer with insufficient balance
/// let current_amount = 500_000; // 0.0005 SOL in lamports
/// let gas_reserve = 1_000_000; // 0.001 SOL in lamports
/// let max_amount = calculate_max_transferable_amount("", current_amount, gas_reserve);
/// assert_eq!(max_amount, 0); // Can't transfer anything
/// ```
pub fn calculate_max_transferable_amount(
    mint_address: &str,
    current_amount: u64,
    gas_amount_lamport: u64,
) -> u64 {
    // Use the standardized gas reserve calculation for SOL
    if mint_address.is_empty() || mint_address == "So11111111111111111111111111111111112" {
        return crate::gas_reserve::calculate_max_sol_transferable(
            current_amount,
            Some(gas_amount_lamport),
        );
    }

    // Use the standardized gas reserve calculation for SPL tokens
    crate::gas_reserve::calculate_max_spl_transferable(current_amount, Some(gas_amount_lamport))
}
