//! Token Utilities Module
//!
//! This module provides utility functions for handling token symbols and amounts.

/// Convert mint address to token symbol
pub fn mint_to_symbol(mint: &str) -> &str {
    match mint {
        "So11111111111111111111111111111111111111112" => "SOL",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
        _ => {
            if mint.len() > 8 {
                "UNKNOWN"
            } else {
                mint
            }
        }
    }
}

/// Convert lamports to readable token amount based on mint
pub fn lamports_to_token_amount(lamports: u64, mint: &str) -> String {
    match mint {
        "So11111111111111111111111111111111111111112" => {
            // SOL: 9 decimal places
            format!("{:.3}", lamports as f64 / 1_000_000_000.0)
        }
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => {
            // USDC: 6 decimal places
            format!("{:.2}", lamports as f64 / 1_000_000.0)
        }
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => {
            // USDT: 6 decimal places
            format!("{:.2}", lamports as f64 / 1_000_000.0)
        }
        _ => {
            // Default: assume 6 decimal places
            format!("{:.6}", lamports as f64 / 1_000_000.0)
        }
    }
}
