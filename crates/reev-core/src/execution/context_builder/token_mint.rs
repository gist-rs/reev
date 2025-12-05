//! Typed wrapper for token mints with validation and common constants

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use thiserror::Error;

/// Error types for token mint operations
#[derive(Debug, Error)]
pub enum MintError {
    #[error("Invalid mint address: {0}")]
    InvalidAddress(String),
    #[error("Mint not found: {0}")]
    NotFound(String),
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Typed wrapper for token mints with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    /// Mint address as string
    pub address: String,
    /// Token symbol if available
    pub symbol: Option<String>,
    /// Token decimals if available
    pub decimals: Option<u8>,
    /// Token price in USD if available
    pub price_usd: Option<f64>,
}

impl TokenMint {
    /// Create a new TokenMint with just an address
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            symbol: None,
            decimals: None,
            price_usd: None,
        }
    }

    /// Create a TokenMint with address and symbol
    pub fn with_symbol(address: impl Into<String>, symbol: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            symbol: Some(symbol.into()),
            decimals: None,
            price_usd: None,
        }
    }

    /// Create a TokenMint with full metadata
    pub fn with_metadata(
        address: impl Into<String>,
        symbol: Option<String>,
        decimals: Option<u8>,
        price_usd: Option<f64>,
    ) -> Self {
        Self {
            address: address.into(),
            symbol,
            decimals,
            price_usd,
        }
    }

    /// Convert to Pubkey with validation
    pub fn to_pubkey(&self) -> Result<Pubkey, MintError> {
        Pubkey::from_str(&self.address).map_err(|e| MintError::ConversionError(e.to_string()))
    }

    /// Check if this is a SOL mint
    pub fn is_sol(&self) -> bool {
        self.address == mints::SOL
    }

    /// Check if this is a USDC mint
    pub fn is_usdc(&self) -> bool {
        self.address == mints::USDC
    }

    /// Check if this is a USDT mint
    pub fn is_usdt(&self) -> bool {
        self.address == mints::USDT
    }
}

impl From<Pubkey> for TokenMint {
    fn from(pubkey: Pubkey) -> Self {
        Self::new(pubkey.to_string())
    }
}

impl TryFrom<&str> for TokenMint {
    type Error = MintError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for TokenMint {
    type Error = MintError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for TokenMint {
    type Err = MintError;

    fn from_str(mint_str: &str) -> Result<Self, Self::Err> {
        // Validate the string is a valid Pubkey
        if Pubkey::from_str(mint_str).is_err() {
            return Err(MintError::InvalidAddress(mint_str.to_string()));
        }

        Ok(Self::new(mint_str))
    }
}

/// Common token mint addresses
pub mod mints {
    /// SOL mint address
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    /// USDC mint address
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    /// USDT mint address
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    /// RAY mint address
    pub const RAY: &str = "4k3Dyjzvz8SLJJ5aB6qQmYCzpcLQGQdip8JFkQkqP5Cv";

    /// Get a TokenMint for SOL
    pub fn sol() -> super::TokenMint {
        super::TokenMint::with_symbol(SOL, "SOL")
    }

    /// Get a TokenMint for USDC
    pub fn usdc() -> super::TokenMint {
        super::TokenMint::with_symbol(USDC, "USDC")
    }

    /// Get a TokenMint for USDT
    pub fn usdt() -> super::TokenMint {
        super::TokenMint::with_symbol(USDT, "USDT")
    }

    /// Get a TokenMint for RAY
    pub fn ray() -> super::TokenMint {
        super::TokenMint::with_symbol(RAY, "RAY")
    }
}

/// Helper functions for working with token mints
pub mod helpers {
    use super::*;

    /// Convert a string to a valid mint address, defaulting to SOL if invalid
    pub fn resolve_mint_or_default(
        mint_str: &str,
        key_map: &std::collections::HashMap<String, String>,
    ) -> String {
        // Check if it's a key that needs to be resolved
        if mint_str.starts_with("USER_") || mint_str.starts_with("RECIPIENT_") {
            if let Some(resolved_mint) = key_map.get(mint_str) {
                return resolved_mint.clone();
            }
        }

        // Validate the mint address
        if Pubkey::from_str(mint_str).is_ok() {
            return mint_str.to_string();
        }

        // Default to SOL
        mints::SOL.to_string()
    }

    /// Get the display symbol for a mint address
    pub fn get_mint_symbol(mint_address: &str) -> &'static str {
        match mint_address {
            mints::SOL => "SOL",
            mints::USDC => "USDC",
            mints::USDT => "USDT",
            mints::RAY => "RAY",
            _ => "Unknown",
        }
    }
}
