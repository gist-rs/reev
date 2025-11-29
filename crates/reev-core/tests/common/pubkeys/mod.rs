//! Common pubkeys used in tests
//!
//! This module provides commonly used pubkeys across all e2e tests
//! to avoid hardcoding and ensure consistency.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub const TARGET: &str = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";
pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const JUSDC: &str = "jupsoL7By9suyDaGK735BLahFzhWd8vFjYUjdnFnJsw";

/// Get the target pubkey for transfer tests
pub fn target() -> Pubkey {
    Pubkey::from_str(TARGET).expect("Invalid target pubkey")
}

/// Get the USDC mint pubkey
pub fn usdc() -> Pubkey {
    Pubkey::from_str(USDC).expect("Invalid USDC pubkey")
}

/// Get the Jupiter USDC (jUSDC) mint pubkey
pub fn jusdc() -> Pubkey {
    Pubkey::from_str(JUSDC).expect("Invalid JUSDC pubkey")
}
