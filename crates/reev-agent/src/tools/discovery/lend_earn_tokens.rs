//! Jupiter Lend/Earn Tokens Discovery Tool
//!
//! This tool provides the LLM with access to Jupiter lending token information
//! including prices, APYs, and liquidity data for informed decision making.

use reqwest::Client;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// The arguments for the lend/earn tokens tool
#[derive(Deserialize, Debug)]
pub struct LendEarnTokensArgs {
    /// Optional: Filter by specific token symbol (e.g., "USDC", "SOL")
    pub symbol: Option<String>,
    /// Optional: Filter by token mint address
    pub mint_address: Option<String>,
    /// Optional: Filter by Jupiter token address
    pub jupiter_address: Option<String>,
}

/// Jupiter lend token information
#[derive(Deserialize, Debug, Serialize)]
pub struct LendEarnToken {
    /// Token ID
    pub id: u32,
    /// Jupiter token address
    pub address: String,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
    /// Underlying asset address
    pub asset_address: String,
    /// Asset information including price
    pub asset: AssetInfo,
    /// Total assets in the pool
    pub total_assets: String,
    /// Total supply of tokens
    pub total_supply: String,
    /// Conversion rate to shares
    pub convert_to_shares: String,
    /// Conversion rate to assets
    pub convert_to_assets: String,
    /// Rewards rate
    pub rewards_rate: String,
    /// Supply rate (APY)
    pub supply_rate: String,
    /// Total rate (rewards + supply)
    pub total_rate: String,
    /// Liquidity supply data
    pub liquidity_supply_data: LiquiditySupplyData,
    /// Rewards information
    pub rewards: Vec<serde_json::Value>,
}

/// Asset information with price data
#[derive(Deserialize, Debug, Serialize)]
pub struct AssetInfo {
    /// Asset address
    pub address: String,
    /// Chain ID
    pub chain_id: String,
    /// Asset name
    pub name: String,
    /// Asset symbol
    pub symbol: String,
    /// Asset decimals
    pub decimals: u8,
    /// Logo URL
    pub logo_url: String,
    /// Current price in USD
    pub price: String,
    /// Coingecko ID
    pub coingecko_id: String,
}

/// Liquidity supply data
#[derive(Deserialize, Debug, Serialize)]
pub struct LiquiditySupplyData {
    /// Whether mode with interest is enabled
    pub mode_with_interest: bool,
    /// Total supply
    pub supply: String,
    /// Withdrawal limit
    pub withdrawal_limit: String,
    /// Last update timestamp
    pub last_update_timestamp: String,
    /// Expand percent
    pub expand_percent: u32,
    /// Expand duration
    pub expand_duration: u32,
    /// Base withdrawal limit
    pub base_withdrawal_limit: String,
    /// Withdrawable until limit
    pub withdrawable_until_limit: String,
    /// Currently withdrawable amount
    pub withdrawable: String,
}

/// A custom error type for the lend/earn tokens tool
#[derive(Debug, Error)]
pub enum LendEarnTokensError {
    #[error("Failed to fetch tokens from Jupiter API: {0}")]
    ApiError(String),
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("No tokens found matching criteria")]
    NoTokensFound,
}

/// Jupiter lend/earn tokens discovery tool
#[derive(Deserialize, Serialize)]
pub struct LendEarnTokensTool {
    pub key_map: HashMap<String, String>,
    #[serde(skip)]
    client: Client,
}

impl Tool for LendEarnTokensTool {
    const NAME: &'static str = "get_lend_earn_tokens";
    type Error = LendEarnTokensError;
    type Args = LendEarnTokensArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Query Jupiter lending and earning tokens with real-time prices, APYs, and liquidity information. Use this to get current token prices, lending rates, and available tokens for Jupiter operations. Returns comprehensive token data including current USD prices, supply rates, and liquidity limits.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Optional: Filter by specific token symbol (e.g., 'USDC', 'SOL', 'USDT')"
                    },
                    "mint_address": {
                        "type": "string",
                        "description": "Optional: Filter by underlying asset mint address"
                    },
                    "jupiter_address": {
                        "type": "string",
                        "description": "Optional: Filter by Jupiter token address"
                    }
                }
            })
        }
    }

    /// Executes the tool to query lend/earn tokens
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Fetch tokens from Jupiter API
        let tokens = self.fetch_tokens().await?;

        // Apply filters if provided
        let filtered_tokens = self.apply_filters(tokens, &args);

        if filtered_tokens.is_empty() {
            return Err(LendEarnTokensError::NoTokensFound);
        }

        // Convert to JSON response
        let response = json!({
            "tokens": filtered_tokens,
            "query_params": {
                "symbol": args.symbol,
                "mint_address": args.mint_address,
                "jupiter_address": args.jupiter_address
            },
            "total_tokens": filtered_tokens.len(),
            "api_source": "https://lite-api.jup.ag/lend/v1/earn/tokens"
        });

        Ok(serde_json::to_string_pretty(&response)?)
    }
}

impl LendEarnTokensTool {
    /// Create a new lend/earn tokens tool
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self {
            key_map,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Fetch tokens from Jupiter API
    async fn fetch_tokens(&self) -> Result<Vec<LendEarnToken>, LendEarnTokensError> {
        let url = "https://lite-api.jup.ag/lend/v1/earn/tokens";

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LendEarnTokensError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let tokens: Vec<LendEarnToken> = response.json().await?;
        Ok(tokens)
    }

    /// Apply filters to the tokens list
    fn apply_filters(
        &self,
        tokens: Vec<LendEarnToken>,
        args: &LendEarnTokensArgs,
    ) -> Vec<LendEarnToken> {
        tokens
            .into_iter()
            .filter(|token| {
                // Filter by symbol
                if let Some(symbol) = &args.symbol {
                    if !token.symbol.to_lowercase().contains(&symbol.to_lowercase()) {
                        return false;
                    }
                }

                // Filter by mint address
                if let Some(mint_address) = &args.mint_address {
                    if token.asset_address.to_lowercase() != mint_address.to_lowercase() {
                        return false;
                    }
                }

                // Filter by Jupiter address
                if let Some(jupiter_address) = &args.jupiter_address {
                    if token.address.to_lowercase() != jupiter_address.to_lowercase() {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}

impl Default for LendEarnTokensTool {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
