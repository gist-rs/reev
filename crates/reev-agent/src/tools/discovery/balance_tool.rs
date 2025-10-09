//! Account Balance Discovery Tool
//!
//! This tool provides the LLM with the ability to query account balances
//! when context is insufficient, enabling prerequisite validation before operations.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// The arguments for the account balance tool
#[derive(Deserialize, Debug)]
pub struct AccountBalanceArgs {
    /// The public key of the account to query
    pub pubkey: String,
    /// Optional: The token mint to query specific token balance
    pub token_mint: Option<String>,
    /// Optional: The type of account (wallet, token_account, etc.)
    pub account_type: Option<String>,
}

/// Account balance information
#[derive(Serialize, Debug)]
pub struct AccountBalance {
    /// The account public key
    pub pubkey: String,
    /// Account type (wallet, token_account, etc.)
    pub account_type: String,
    /// SOL balance in lamports
    pub sol_balance: u64,
    /// Token balances if this is a token account
    pub token_balances: Vec<TokenBalance>,
    /// Whether the account exists
    pub exists: bool,
}

/// Token balance information
#[derive(Serialize, Debug)]
pub struct TokenBalance {
    /// Token mint address
    pub mint: String,
    /// Token balance in smallest units
    pub balance: u64,
    /// Token decimals
    pub decimals: u8,
    /// Symbol if known
    pub symbol: Option<String>,
}

/// A custom error type for the account balance tool
#[derive(Debug, Error)]
pub enum AccountBalanceError {
    #[error("Invalid account pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Failed to query balance: {0}")]
    QueryError(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Account balance discovery tool
#[derive(Deserialize, Serialize)]
pub struct AccountBalanceTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for AccountBalanceTool {
    const NAME: &'static str = "get_account_balance";
    type Error = AccountBalanceError;
    type Args = AccountBalanceArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Query account balance information including SOL and token balances. Use this when you need to verify if an account has sufficient funds before executing a transfer or operation. Returns SOL balance in lamports and token balances with mint addresses.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pubkey": {
                        "type": "string",
                        "description": "The public key of the account to query balance for"
                    },
                    "token_mint": {
                        "type": "string",
                        "description": "Optional: The token mint address to query specific token balance. If not provided, returns all token balances"
                    },
                    "account_type": {
                        "type": "string",
                        "description": "Optional: The type of account (wallet, token_account, etc.). Helps in parsing the account correctly"
                    }
                },
                "required": ["pubkey"]
            })
        }
    }

    /// Executes the tool to query account balance
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Handle placeholder pubkeys gracefully
        if args.pubkey.contains("USER_") || args.pubkey.contains("RECIPIENT_") {
            // For placeholder pubkeys, use simulated data based on the placeholder name
            return self.query_placeholder_balance(&args).await;
        }

        // Validate the pubkey format for real addresses
        if args.pubkey.len() != 44 && args.pubkey.len() != 43 {
            return Err(AccountBalanceError::InvalidPubkey(args.pubkey.clone()));
        }

        // For now, we'll simulate balance queries
        // In a real implementation, this would query the Solana RPC
        let balance_info = self.query_account_balance(&args).await?;

        // Convert to JSON response
        let response = json!({
            "account": balance_info,
            "query_params": {
                "pubkey": args.pubkey,
                "token_mint": args.token_mint,
                "account_type": args.account_type
            }
        });

        Ok(serde_json::to_string_pretty(&response)?)
    }
}

impl AccountBalanceTool {
    /// Create a new account balance tool
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self { key_map }
    }

    /// Query account balance from the blockchain or simulated data
    async fn query_account_balance(
        &self,
        args: &AccountBalanceArgs,
    ) -> Result<AccountBalance, AccountBalanceError> {
        // Check if this is a known account from key_map
        let account_info = if let Some(resolved_pubkey) = self.key_map.get(&args.pubkey) {
            // This is a placeholder account, simulate balance
            self.simulate_account_balance(resolved_pubkey, args).await?
        } else {
            // This might be a real pubkey, simulate basic wallet
            AccountBalance {
                pubkey: args.pubkey.clone(),
                account_type: args
                    .account_type
                    .clone()
                    .unwrap_or_else(|| "wallet".to_string()),
                sol_balance: 1000000000, // 1 SOL default
                token_balances: vec![],
                exists: true,
            }
        };

        Ok(account_info)
    }

    /// Simulate account balance for placeholder accounts
    async fn simulate_account_balance(
        &self,
        pubkey: &str,
        args: &AccountBalanceArgs,
    ) -> Result<AccountBalance, AccountBalanceError> {
        let account_type = args.account_type.clone().unwrap_or_else(|| {
            // Determine account type from pubkey name
            if pubkey.contains("WALLET") {
                "wallet".to_string()
            } else if pubkey.contains("ATA") {
                "token_account".to_string()
            } else {
                "unknown".to_string()
            }
        });

        match account_type.as_str() {
            "wallet" => Ok(AccountBalance {
                pubkey: pubkey.to_string(),
                account_type: "wallet".to_string(),
                sol_balance: 2000000000, // 2 SOL
                token_balances: vec![],
                exists: true,
            }),
            "token_account" => {
                // Determine token type from pubkey name
                let token_balance = if pubkey.contains("USDC") {
                    vec![TokenBalance {
                        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                        balance: 100000000, // 100 USDC (6 decimals)
                        decimals: 6,
                        symbol: Some("USDC".to_string()),
                    }]
                } else if pubkey.contains("L_USDC") {
                    vec![TokenBalance {
                        mint: "D23a1LgEa5SyWUJZnkqde1qRQzYhGdM6kY".to_string(), // Example L-USDC mint
                        balance: 50000000,                                      // 50 L-USDC shares
                        decimals: 6,
                        symbol: Some("L-USDC".to_string()),
                    }]
                } else {
                    vec![]
                };

                Ok(AccountBalance {
                    pubkey: pubkey.to_string(),
                    account_type: "token_account".to_string(),
                    sol_balance: 2039280, // Rent exemption
                    token_balances: token_balance,
                    exists: true,
                })
            }
            _ => Ok(AccountBalance {
                pubkey: pubkey.to_string(),
                account_type,
                sol_balance: 0,
                token_balances: vec![],
                exists: false,
            }),
        }
    }

    /// Query balance for placeholder pubkeys (simulation)
    async fn query_placeholder_balance(
        &self,
        args: &AccountBalanceArgs,
    ) -> Result<String, AccountBalanceError> {
        let account_type = args.account_type.clone().unwrap_or_else(|| {
            // Determine account type from pubkey name
            if args.pubkey.contains("WALLET") {
                "wallet".to_string()
            } else if args.pubkey.contains("ATA") {
                "token_account".to_string()
            } else {
                "unknown".to_string()
            }
        });

        let balance_info = match account_type.as_str() {
            "wallet" => AccountBalance {
                pubkey: args.pubkey.clone(),
                account_type: "wallet".to_string(),
                sol_balance: 2000000000, // 2 SOL
                token_balances: vec![],
                exists: true,
            },
            "token_account" => {
                // Determine token type from pubkey name
                let token_balance = if let Some(token_mint) = &args.token_mint {
                    match token_mint.as_str() {
                        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => vec![TokenBalance {
                            mint: token_mint.clone(),
                            balance: 100000000, // 100 USDC (6 decimals)
                            decimals: 6,
                            symbol: Some("USDC".to_string()),
                        }],
                        "So11111111111111111111111111111111111111112" => vec![TokenBalance {
                            mint: token_mint.clone(),
                            balance: 1000000000, // 1 SOL (9 decimals)
                            decimals: 9,
                            symbol: Some("SOL".to_string()),
                        }],
                        _ => vec![],
                    }
                } else if args.pubkey.contains("USDC") {
                    vec![TokenBalance {
                        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                        balance: 100000000, // 100 USDC (6 decimals)
                        decimals: 6,
                        symbol: Some("USDC".to_string()),
                    }]
                } else if args.pubkey.contains("L_USDC") {
                    vec![TokenBalance {
                        mint: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(), // jlUSDC mint
                        balance: 98721000, // ~98.7 jlUSDC shares
                        decimals: 6,
                        symbol: Some("jlUSDC".to_string()),
                    }]
                } else {
                    vec![]
                };

                AccountBalance {
                    pubkey: args.pubkey.clone(),
                    account_type: "token_account".to_string(),
                    sol_balance: 2039280, // Rent exemption
                    token_balances: token_balance,
                    exists: true,
                }
            }
            _ => AccountBalance {
                pubkey: args.pubkey.clone(),
                account_type,
                sol_balance: 0,
                token_balances: vec![],
                exists: false,
            },
        };

        // Convert to JSON response
        let response = json!({
            "account": balance_info,
            "query_params": {
                "pubkey": args.pubkey,
                "token_mint": args.token_mint,
                "account_type": args.account_type
            },
            "note": "This is simulated data for placeholder pubkey. In a real scenario, use the resolved address from context.",
            "placeholder_detected": args.pubkey
        });

        Ok(serde_json::to_string_pretty(&response)?)
    }
}
