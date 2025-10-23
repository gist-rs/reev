//! Context enhancement module for providing LLM with prerequisite account information
//!
//! This module parses benchmark YAML files to extract account balances and positions,
//! providing the LLM with necessary context to make intelligent decisions without
//! unnecessary tool calls.

use reev_lib::benchmark::InitialStateItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Failed to parse account data: {0}")]
    ParseError(String),
    #[error("Invalid token amount: {0}")]
    InvalidAmount(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Account context information extracted from benchmark initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountContext {
    /// User's wallet SOL balance
    pub sol_balance: Option<u64>,
    /// Token account balances
    pub token_balances: HashMap<String, TokenBalance>,
    /// Lending positions and shares
    pub lending_positions: HashMap<String, LendingPosition>,
    /// Formatted context string for LLM
    pub formatted_context: String,
}

/// Token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token mint address
    pub mint: String,
    /// Token amount in smallest units
    pub amount: u64,
    /// Token owner (wallet)
    pub owner: String,
    /// Formatted amount string (e.g., "50 USDC")
    pub formatted_amount: String,
}

/// Lending position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPosition {
    /// Position/token mint address
    pub mint: String,
    /// Number of shares/position tokens
    pub shares: u64,
    /// Position owner (wallet)
    pub owner: String,
    /// Position type (e.g., "jUSDC", "L-USDC")
    pub position_type: String,
    /// Formatted shares string
    pub formatted_shares: String,
}

/// Context builder for extracting account information from benchmark initial state
pub struct ContextBuilder {
    /// Token mint address to symbol mapping
    token_symbols: HashMap<String, String>,
    /// Token decimals for formatting
    token_decimals: HashMap<String, u8>,
}

impl ContextBuilder {
    /// Create a new context builder with known token mappings
    pub fn new() -> Self {
        let mut token_symbols = HashMap::new();
        let mut token_decimals = HashMap::new();

        // Common Solana tokens
        token_symbols.insert(
            "So11111111111111111111111111111111111111112".to_string(),
            "SOL".to_string(),
        );
        token_decimals.insert("So11111111111111111111111111111111111111112".to_string(), 9);

        token_symbols.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            "USDC".to_string(),
        );
        token_decimals.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            6,
        );

        token_symbols.insert(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            "USDT".to_string(),
        );
        token_decimals.insert(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
            6,
        );

        // Jupiter lending tokens
        token_symbols.insert(
            "DHqzaSm8w9X2zprx7FLabfU1JhKFmQzuQsWzJ2hJgKq".to_string(),
            "jSOL".to_string(),
        );
        token_decimals.insert("DHqzaSm8w9X2zprx7FLabfU1JhKFmQzuQsWzJ2hJgKq".to_string(), 9);

        token_symbols.insert(
            "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7KgCKB".to_string(),
            "jUSDC".to_string(),
        );
        token_decimals.insert(
            "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7KgCKB".to_string(),
            6,
        );

        // Solend tokens
        token_symbols.insert(
            "2uQsyo1fXXQkDtcpXnLofWy88PxcvnfH2L8FPSE62FVU".to_string(),
            "L-SOL".to_string(),
        );
        token_decimals.insert(
            "2uQsyo1fXXQkDtcpXnLofWy88PxcvnfH2L8FPSE62FVU".to_string(),
            9,
        );

        token_symbols.insert(
            "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            "L-USDC".to_string(),
        );
        token_decimals.insert(
            "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            6,
        );

        Self {
            token_symbols,
            token_decimals,
        }
    }

    /// Extract account context from initial state items and key mapping
    pub fn build_context(
        &self,
        initial_state: &[InitialStateItem],
        key_map: &HashMap<String, String>,
    ) -> Result<AccountContext, ContextError> {
        let mut sol_balance = None;
        let mut token_balances = HashMap::new();
        let mut lending_positions = HashMap::new();

        for item in initial_state {
            match &item.data {
                Some(data) => {
                    // This is a token account
                    let mint = &data.mint;
                    let amount_str = &data.amount;
                    let owner = &data.owner;

                    let amount = amount_str
                        .parse::<u64>()
                        .map_err(|e| ContextError::InvalidAmount(e.to_string()))?;

                    let token_symbol = self
                        .token_symbols
                        .get(mint)
                        .cloned()
                        .unwrap_or_else(|| format!("TOKEN_{}", &mint[..8]));

                    let decimals = self.token_decimals.get(mint).copied().unwrap_or(0);
                    let formatted_amount =
                        self.format_token_amount(amount, decimals, &token_symbol);

                    // Check if this is a lending position token
                    if self.is_lending_token(mint) {
                        let position_type = if mint.contains("jupiter") || mint.contains("Jupiter")
                        {
                            format!("j{token_symbol}")
                        } else {
                            format!("L-{token_symbol}")
                        };

                        let position_type_clone = position_type.clone();
                        lending_positions.insert(
                            item.pubkey.clone(),
                            LendingPosition {
                                mint: mint.clone(),
                                shares: amount,
                                owner: owner.clone(),
                                position_type,
                                formatted_shares: self.format_token_amount(
                                    amount,
                                    decimals,
                                    &position_type_clone,
                                ),
                            },
                        );
                    } else {
                        token_balances.insert(
                            item.pubkey.clone(),
                            TokenBalance {
                                mint: mint.clone(),
                                amount,
                                owner: owner.clone(),
                                formatted_amount,
                            },
                        );
                    }
                }
                None => {
                    // This might be a SOL account
                    if item.owner == "11111111111111111111111111111111" {
                        sol_balance = Some(item.lamports);
                    }
                }
            }
        }

        // Build formatted context string
        let formatted_context = self.build_formatted_context(
            &sol_balance,
            &token_balances,
            &lending_positions,
            key_map,
        );

        Ok(AccountContext {
            sol_balance,
            token_balances,
            lending_positions,
            formatted_context,
        })
    }

    /// Check if a mint represents a lending position token
    fn is_lending_token(&self, mint: &str) -> bool {
        mint.contains("jupiter") ||
        mint.contains("Jupiter") ||
        mint.contains("9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D") || // L-USDC
        mint.contains("2uQsyo1fXXQkDtcpXnLofWy88PxcvnfH2L8FPSE62FVU") || // L-SOL
        mint.contains("DHqzaSm8w9X2zprx7FLabfU1JhKFmQzuQsWzJ2hJgKq") || // jSOL
        mint.contains("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7KgCKB") // jUSDC
    }

    /// Format token amount for human-readable display
    fn format_token_amount(&self, amount: u64, decimals: u8, symbol: &str) -> String {
        if decimals == 0 {
            format!("{amount} {symbol}")
        } else {
            let divisor = 10u64.pow(decimals as u32);
            let whole = amount / divisor;
            let fractional = amount % divisor;
            if fractional == 0 {
                format!("{whole} {symbol}")
            } else {
                format!(
                    "{}.{:0width$} {}",
                    whole,
                    fractional,
                    symbol,
                    width = decimals as usize
                )
            }
        }
    }

    /// Build formatted context string for LLM
    fn build_formatted_context(
        &self,
        sol_balance: &Option<u64>,
        token_balances: &HashMap<String, TokenBalance>,
        lending_positions: &HashMap<String, LendingPosition>,
        key_map: &HashMap<String, String>,
    ) -> String {
        let mut context = String::from("ACCOUNT BALANCES AND POSITIONS:\n\n");

        // SOL balance
        if let Some(sol) = sol_balance {
            let sol_amount = *sol as f64 / 1_000_000_000.0;
            context.push_str(&format!("💰 SOL Balance: {sol_amount:.4} SOL\n"));
        }

        // Token balances
        if !token_balances.is_empty() {
            context.push_str("\n💎 Token Balances:\n");
            for (pubkey, balance) in token_balances {
                let account_name = self.get_account_name(pubkey, key_map);
                context.push_str(&format!(
                    "  • {}: {}\n",
                    account_name, balance.formatted_amount
                ));
            }
        }

        // Lending positions
        if !lending_positions.is_empty() {
            context.push_str("\n🏦 Lending Positions:\n");
            for (pubkey, position) in lending_positions {
                let account_name = self.get_account_name(pubkey, key_map);
                context.push_str(&format!(
                    "  • {}: {} shares\n",
                    account_name, position.formatted_shares
                ));
            }
        }

        context.push_str("\n💡 You have sufficient account information above. Use this context to make decisions without unnecessary balance checks.");
        context
    }

    /// Get human-readable account name from key mapping
    fn get_account_name(&self, pubkey: &str, key_map: &HashMap<String, String>) -> String {
        // Find the key name for this pubkey
        for (key_name, key_value) in key_map {
            if key_value == pubkey {
                return key_name.clone();
            }
        }

        // Fallback to shortened pubkey
        if pubkey.len() > 12 {
            format!("{}...{}", &pubkey[..6], &pubkey[pubkey.len() - 6..])
        } else {
            pubkey.to_string()
        }
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ContextBuilder;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_build_context_from_observation() {
        let builder = ContextBuilder::new();

        // Create mock account states simulating surfpool observation
        let mut account_states = HashMap::new();

        // Add a SOL account (like USER_WALLET_PUBKEY)
        account_states.insert(
            "USER_WALLET_PUBKEY".to_string(),
            json!({
                "lamports": 1000000000, // 1 SOL
                "owner": "11111111111111111111111111111111", // System Program
                "executable": false,
                "data_len": 0
            }),
        );

        // Add a USDC token account
        account_states.insert(
            "USER_USDC_ATA".to_string(),
            json!({
                "lamports": 2039280,
                "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token Program
                "executable": false,
                "data_len": 165,
                "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "token_account_owner": "USER_WALLET_PUBKEY_RESOLVED",
                "amount": "50000000" // 50 USDC
            }),
        );

        // Create key map
        let mut key_map = HashMap::new();
        key_map.insert(
            "USER_WALLET_PUBKEY".to_string(),
            "USER_WALLET_PUBKEY_RESOLVED".to_string(),
        );
        key_map.insert(
            "USER_USDC_ATA".to_string(),
            "USER_USDC_ATA_RESOLVED".to_string(),
        );

        // Build context from observation (NEW METHOD)
        let context = builder
            .build_context_from_observation(&account_states, &key_map, "test-benchmark")
            .unwrap();

        // Debug: Show actual context
        println!("Actual context: {}", context.formatted_context);

        // Verify the context contains real balances
        assert!(context.formatted_context.contains("1.0000 SOL"));
        assert!(context.formatted_context.contains("50 USDC"));

        // Verify SOL balance is correct
        assert_eq!(context.sol_balance, Some(1000000000));

        // Verify token balance is correct
        assert_eq!(context.token_balances.len(), 1);
        let usdc_balance = context.token_balances.get("USER_USDC_ATA").unwrap();
        assert_eq!(usdc_balance.amount, 50000000);
        assert_eq!(
            usdc_balance.mint,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );

        println!("✅ SUCCESS: Context shows real balances from observation!");
        println!("Context: {}", context.formatted_context);
    }
}

pub mod builder;
pub mod integration;
