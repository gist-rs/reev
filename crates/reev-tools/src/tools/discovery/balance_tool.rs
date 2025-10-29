//! Account Balance Discovery Tool
//!
//! This tool provides the LLM with the ability to query account balances
//! from the real surfpool testnet. This enables proper validation before operations
//! by accessing actual on-chain state, not simulated data.

use reev_lib::constants::{sol_mint, usdc_mint};
use reev_types::TokenBalance;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use tracing::{info, instrument};

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

// TokenBalance now imported from reev-types

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
    #[error("RPC client error: {0}")]
    RpcError(#[from] Box<solana_client::client_error::ClientError>),
}

impl From<solana_client::client_error::ClientError> for AccountBalanceError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        Self::RpcError(Box::new(err))
    }
}

/// Account balance discovery tool that queries real surfpool state
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
            description: "Query REAL account balance information from surfpool testnet. Returns actual SOL and token balances. Use this to verify sufficient funds before transfers, swaps, or deposits. This queries the live surfpool RPC endpoint for real account data.".to_string(),
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
                        "description": "Optional: The type of account (wallet, token_account, etc.). Helps in determining how to interpret the account"
                    }
                },
                "required": ["pubkey"]
            })
        }
    }

    /// Executes the tool to query REAL account balance from surfpool
    #[instrument(
        name = "account_balance_tool_call",
        skip(self),
        fields(
            tool_name = "get_account_balance",
            pubkey = %args.pubkey,
            token_mint = ?args.token_mint,
            account_type = ?args.account_type
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[AccountBalanceTool] Starting tool execution with OpenTelemetry tracing");

        // Prepare tool args for logging
        let _tool_args = json!({
            "pubkey": args.pubkey,
            "token_mint": args.token_mint,
            "account_type": args.account_type
        })
        .to_string();

        // Execute the tool
        let result = self.execute_balance_query_internal(&args).await;

        // Convert result to JSON response
        match result {
            Ok(balance_info) => {
                let resolved_pubkey =
                    if args.pubkey.contains("USER_") || args.pubkey.contains("RECIPIENT_") {
                        self.key_map
                            .get(&args.pubkey)
                            .cloned()
                            .unwrap_or_else(|| args.pubkey.clone())
                    } else {
                        args.pubkey.clone()
                    };

                let response = json!({
                    "account": balance_info,
                    "query_params": {
                        "pubkey": args.pubkey,
                        "resolved_pubkey": resolved_pubkey,
                        "token_mint": args.token_mint,
                        "account_type": args.account_type
                    },
                    "data_source": "surfpool_rpc",
                    "note": "This is REAL account data from surfpool testnet, not simulated values"
                });

                Ok(serde_json::to_string_pretty(&response)?)
            }
            Err(e) => Err(e),
        }
    }
}

impl AccountBalanceTool {
    /// Execute the actual balance query (internal helper function)
    async fn execute_balance_query_internal(
        &self,
        args: &AccountBalanceArgs,
    ) -> Result<AccountBalance, AccountBalanceError> {
        // Handle placeholder pubkeys by resolving them from key_map
        let resolved_pubkey = if args.pubkey.contains("USER_") || args.pubkey.contains("RECIPIENT_")
        {
            if let Some(resolved) = self.key_map.get(&args.pubkey) {
                resolved.clone()
            } else {
                return Err(AccountBalanceError::AccountNotFound(args.pubkey.clone()));
            }
        } else {
            args.pubkey.clone()
        };

        // Validate the pubkey format
        let pubkey = Pubkey::from_str(&resolved_pubkey)
            .map_err(|_| AccountBalanceError::InvalidPubkey(resolved_pubkey.clone()))?;

        // Connect to surfpool RPC endpoint
        let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());

        // Query the account balance from surfpool
        self.query_real_account_balance(&rpc_client, &pubkey, args)
            .await
    }
}

impl AccountBalanceTool {
    /// Create a new account balance tool
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self { key_map }
    }

    /// Query REAL account balance from surfpool RPC
    async fn query_real_account_balance(
        &self,
        rpc_client: &RpcClient,
        pubkey: &Pubkey,
        args: &AccountBalanceArgs,
    ) -> Result<AccountBalance, AccountBalanceError> {
        // Query account info from surfpool
        let account = rpc_client.get_account(pubkey)?;

        let mut token_balances = Vec::new();
        let account_type = args.account_type.clone().unwrap_or_else(|| {
            // Determine account type based on owner
            if account.owner == solana_sdk::system_program::ID {
                "wallet".to_string()
            } else if account.owner == spl_token::ID {
                "token_account".to_string()
            } else {
                "unknown".to_string()
            }
        });

        // If this is a token account, parse the token balance
        if account.owner == spl_token::ID {
            if let Ok(token_account) = spl_token::state::Account::unpack(&account.data) {
                let decimals = self.get_token_decimals(&token_account.mint.to_string());
                let symbol = self.get_token_symbol(&token_account.mint.to_string());

                token_balances.push(TokenBalance {
                    mint: token_account.mint.to_string(),
                    balance: token_account.amount,
                    decimals: Some(decimals),
                    symbol,
                    formatted_amount: None,
                    owner: None,
                });
            }
        }

        // If a specific token mint was requested, query that token account
        if let Some(token_mint) = &args.token_mint {
            let token_mint_pubkey = Pubkey::from_str(token_mint)
                .map_err(|_| AccountBalanceError::InvalidPubkey(token_mint.clone()))?;

            // Calculate the ATA for this token
            let ata = get_associated_token_address(pubkey, &token_mint_pubkey);

            // Try to query the token account
            if let Ok(token_account) = rpc_client.get_account(&ata) {
                if token_account.owner == spl_token::ID {
                    if let Ok(token_state) = spl_token::state::Account::unpack(&token_account.data)
                    {
                        let decimals = self.get_token_decimals(token_mint);
                        let symbol = self.get_token_symbol(token_mint);

                        // Only add if not already present (to avoid duplicates)
                        if !token_balances.iter().any(|tb| tb.mint == *token_mint) {
                            token_balances.push(TokenBalance {
                                mint: token_mint.clone(),
                                balance: token_state.amount,
                                decimals: Some(decimals),
                                symbol,
                                formatted_amount: None,
                                owner: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(AccountBalance {
            pubkey: pubkey.to_string(),
            account_type,
            sol_balance: account.lamports,
            token_balances,
            exists: true,
        })
    }

    /// Get token decimals for common tokens
    fn get_token_decimals(&self, mint: &str) -> u8 {
        match mint {
            _ if mint == usdc_mint().to_string() => 6, // USDC
            _ if mint == sol_mint().to_string() => 9,  // SOL/WSOL
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => 6, // USDT
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => 5, // Bonk
            _ => 0,                                    // Default to 0 decimals for unknown tokens
        }
    }

    /// Get token symbol for common tokens
    fn get_token_symbol(&self, mint: &str) -> Option<String> {
        match mint {
            _ if mint == usdc_mint().to_string() => Some("USDC".to_string()),
            _ if mint == sol_mint().to_string() => Some("SOL".to_string()),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some("USDT".to_string()),
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => Some("BONK".to_string()),
            _ => None,
        }
    }
}
