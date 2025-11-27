//! Shared Balance Validation Utilities
//!
//! This module provides common balance validation functionality that can be used
//! across multiple tools to prevent insufficient funds errors and ensure
//! proper balance checking before token operations.
//!
//! This utility queries REAL account data from surfpool RPC, not simulated values.

use reev_types::TokenBalance;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur during balance validation
#[derive(Debug, Error)]
pub enum BalanceValidationError {
    #[error("Insufficient funds: requested {requested}, available {available}")]
    InsufficientFunds { requested: u64, available: u64 },
    #[error("Account not found: {account}")]
    AccountNotFound { account: String },
    #[error("Invalid amount: {amount}")]
    InvalidAmount { amount: String },
    #[error("Could not determine balance for mint: {mint}")]
    BalanceUnavailable { mint: String },
    #[error("RPC error: {0}")]
    RpcError(#[from] Box<solana_client::client_error::ClientError>),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
}

impl From<solana_client::client_error::ClientError> for BalanceValidationError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        Self::RpcError(Box::new(err))
    }
}

pub type BalanceValidationResult<T> = Result<T, BalanceValidationError>;

// TokenBalance now imported from reev-types

/// Shared balance validation utilities that query REAL surfpool data
pub struct BalanceValidator {
    /// Key map containing account information from context (for resolving placeholders)
    pub key_map: HashMap<String, String>,
    /// RPC client for querying surfpool
    rpc_client: RpcClient,
}

impl BalanceValidator {
    /// Create a new balance validator with the given key map and surfpool RPC
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self {
            key_map,
            rpc_client: RpcClient::new("http://127.0.0.1:8899".to_string()),
        }
    }

    /// Create a new balance validator with custom RPC endpoint
    pub fn new_with_rpc(key_map: HashMap<String, String>, rpc_url: String) -> Self {
        Self {
            key_map,
            rpc_client: RpcClient::new(rpc_url),
        }
    }

    /// Validate that the requested amount does not exceed available balance
    /// for a specific token mint and owner. Returns error if insufficient funds.
    pub fn validate_token_balance(
        &self,
        mint: &str,
        owner: &str,
        requested_amount: u64,
    ) -> BalanceValidationResult<()> {
        if requested_amount == 0 {
            return Err(BalanceValidationError::InvalidAmount {
                amount: requested_amount.to_string(),
            });
        }

        // For native SOL, check the lamports balance directly in the wallet account
        if mint == "So11111111111111111111111111111111111111112" {
            match self.get_sol_balance(owner) {
                Ok(available) => {
                    if requested_amount > available {
                        Err(BalanceValidationError::InsufficientFunds {
                            requested: requested_amount,
                            available,
                        })
                    } else {
                        Ok(())
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            // For other tokens, check the token account balance
            match self.get_token_balance(mint, owner) {
                Ok(available) => {
                    if requested_amount > available {
                        Err(BalanceValidationError::InsufficientFunds {
                            requested: requested_amount,
                            available,
                        })
                    } else {
                        Ok(())
                    }
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Get the available balance for a specific token mint and owner from surfpool RPC
    pub fn get_token_balance(&self, mint: &str, owner: &str) -> BalanceValidationResult<u64> {
        // Resolve owner pubkey if it's a placeholder
        let resolved_owner = if owner.contains("USER_") || owner.contains("RECIPIENT_") {
            if let Some(resolved) = self.key_map.get(owner) {
                resolved.clone()
            } else {
                return Err(BalanceValidationError::AccountNotFound {
                    account: owner.to_string(),
                });
            }
        } else {
            owner.to_string()
        };

        let owner_pubkey = Pubkey::from_str(&resolved_owner)
            .map_err(|_| BalanceValidationError::InvalidPubkey(resolved_owner))?;
        let mint_pubkey = Pubkey::from_str(mint)
            .map_err(|_| BalanceValidationError::InvalidPubkey(mint.to_string()))?;

        // Calculate the ATA for this token
        let ata = get_associated_token_address(&owner_pubkey, &mint_pubkey);

        // Query the token account from surfpool
        match self.rpc_client.get_account(&ata) {
            Ok(account) => {
                if account.owner == spl_token::ID {
                    // Parse the token account
                    match spl_token::state::Account::unpack(&account.data) {
                        Ok(token_account) => {
                            tracing::info!(
                                "DEBUG: BalanceValidator.get_token_balance - Account {} for mint {} has balance: {}",
                                ata, mint, token_account.amount
                            );
                            Ok(token_account.amount)
                        }
                        Err(_) => Err(BalanceValidationError::BalanceUnavailable {
                            mint: mint.to_string(),
                        }),
                    }
                } else {
                    Err(BalanceValidationError::BalanceUnavailable {
                        mint: mint.to_string(),
                    })
                }
            }
            Err(_) => {
                tracing::info!(
                    "DEBUG: BalanceValidator.get_token_balance - Account {} not found for mint {}",
                    ata,
                    mint
                );
                Err(BalanceValidationError::AccountNotFound {
                    account: ata.to_string(),
                })
            }
        }
    }

    /// Get the maximum swappable SOL amount after reserving fees
    /// This is useful for "swap all SOL" operations where we need to
    /// reserve some SOL for transaction fees
    pub fn get_max_swappable_sol(
        &self,
        owner: &str,
        fee_reserve: u64,
    ) -> BalanceValidationResult<u64> {
        let available_balance = self.get_sol_balance(owner)?;
        let max_swappable = available_balance.saturating_sub(fee_reserve);

        // Ensure we don't return zero if the user has some balance but less than the fee reserve
        if available_balance > 0 && max_swappable == 0 {
            return Err(BalanceValidationError::InsufficientFunds {
                requested: fee_reserve,
                available: available_balance,
            });
        }

        Ok(max_swappable)
    }

    /// Get token balance information including decimals for a specific mint and owner
    pub fn get_token_balance_info(
        &self,
        mint: &str,
        owner: &str,
    ) -> BalanceValidationResult<TokenBalance> {
        let balance = if mint == "So11111111111111111111111111111111111111112" {
            self.get_sol_balance(owner)?
        } else {
            self.get_token_balance(mint, owner)?
        };
        let decimals = self.get_token_decimals(mint);

        Ok(TokenBalance {
            mint: mint.to_string(),
            balance,
            decimals: Some(decimals),
            formatted_amount: None,
            owner: None,
            symbol: None,
        })
    }

    /// Get all token balances available in the key_map (legacy method for compatibility)
    pub fn get_all_token_balances(&self) -> Vec<TokenBalance> {
        // This method is deprecated - use get_token_balance_info with specific owner instead
        Vec::new()
    }

    /// Get token decimals for common tokens
    fn get_token_decimals(&self, mint: &str) -> u8 {
        match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => 6, // USDC
            "So11111111111111111111111111111111111111112" => 9,  // SOL/WSOL
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => 6, // USDT
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => 5, // Bonk
            _ => 0, // Default to 0 decimals for unknown tokens
        }
    }

    /// Check if an account has sufficient balance for multiple token operations
    pub fn validate_multiple_balances(
        &self,
        requirements: &[(String, String, u64)], // (mint, owner, requested_amount)
    ) -> BalanceValidationResult<()> {
        for (mint, owner, amount) in requirements {
            self.validate_token_balance(mint, owner, *amount)?;
        }
        Ok(())
    }

    /// Get the maximum available balance for a token (useful for "deposit all" operations)
    pub fn get_max_available_balance(
        &self,
        mint: &str,
        owner: &str,
    ) -> BalanceValidationResult<u64> {
        if mint == "So11111111111111111111111111111111111111112" {
            self.get_sol_balance(owner)
        } else {
            self.get_token_balance(mint, owner)
        }
    }

    // Private helper methods

    /// Get identifier patterns for a token mint
    #[allow(unused)]
    fn get_token_identifiers(&self, mint: &str) -> Vec<String> {
        match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => {
                vec!["USDC".to_string(), "usdc".to_string(), mint.to_string()]
            }
            "So11111111111111111111111111111111111111112" => {
                vec![
                    "SOL".to_string(),
                    "sol".to_string(),
                    "WSOL".to_string(),
                    "wsol".to_string(),
                    mint.to_string(),
                ]
            }
            _ => vec![mint.to_string()],
        }
    }

    /// Get SOL balance for an account from surfpool
    pub fn get_sol_balance(&self, owner: &str) -> BalanceValidationResult<u64> {
        // Resolve owner pubkey if it's a placeholder
        let resolved_owner = if owner.contains("USER_") || owner.contains("RECIPIENT_") {
            if let Some(resolved) = self.key_map.get(owner) {
                resolved.clone()
            } else {
                return Err(BalanceValidationError::AccountNotFound {
                    account: owner.to_string(),
                });
            }
        } else {
            owner.to_string()
        };

        let owner_pubkey = Pubkey::from_str(&resolved_owner.clone())
            .map_err(|_| BalanceValidationError::InvalidPubkey(resolved_owner.clone()))?;

        // Query the account from surfpool
        match self.rpc_client.get_account(&owner_pubkey) {
            Ok(account) => Ok(account.lamports),
            Err(_) => Err(BalanceValidationError::AccountNotFound {
                account: resolved_owner,
            }),
        }
    }

    /// Check if a given SOL amount can be swapped after accounting for fees
    /// Returns the actual swappable amount after reserving fees
    pub fn get_swappable_amount_after_fees(
        &self,
        owner: &str,
        requested_amount: u64,
        fee_reserve: u64,
    ) -> BalanceValidationResult<u64> {
        let available_balance = self.get_sol_balance(owner)?;

        // If the requested amount is more than available, use the max swappable
        if requested_amount >= available_balance {
            return self.get_max_swappable_sol(owner, fee_reserve);
        }

        // For specific amounts, ensure we have enough for the amount plus fees
        if requested_amount + fee_reserve > available_balance {
            return Err(BalanceValidationError::InsufficientFunds {
                requested: requested_amount + fee_reserve,
                available: available_balance,
            });
        }

        Ok(requested_amount)
    }
}

/// Convenience function to create a balance validator from key_map
pub fn create_balance_validator(key_map: HashMap<String, String>) -> BalanceValidator {
    BalanceValidator::new(key_map)
}

/// Convenience function to validate a single token balance
pub fn validate_balance(
    key_map: &HashMap<String, String>,
    mint: &str,
    owner: &str,
    requested_amount: u64,
) -> BalanceValidationResult<()> {
    let validator = BalanceValidator::new(key_map.clone());
    validator.validate_token_balance(mint, owner, requested_amount)
}

/// Convenience function to get token balance
pub fn get_token_balance(
    key_map: &HashMap<String, String>,
    mint: &str,
    owner: &str,
) -> BalanceValidationResult<u64> {
    let validator = BalanceValidator::new(key_map.clone());
    validator.get_token_balance(mint, owner)
}
