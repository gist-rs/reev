//! Jupiter Lend/Earn Tokens Discovery Tool
//!
//! This tool provides the LLM with access to Jupiter lending token information
//! including prices, APYs, and liquidity data for informed decision making.

use reqwest::Client;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, instrument};

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
    #[serde(rename = "assetAddress")]
    pub asset_address: String,
    /// Asset information including price
    pub asset: AssetInfo,
    /// Total assets in the pool
    #[serde(default)]
    pub total_assets: Option<String>,
    /// Total supply of tokens
    #[serde(default)]
    pub total_supply: Option<String>,
    /// Conversion rate to shares
    #[serde(default)]
    pub convert_to_shares: Option<String>,
    /// Conversion rate to assets
    #[serde(default)]
    pub convert_to_assets: Option<String>,
    /// Rewards rate
    #[serde(default)]
    pub rewards_rate: Option<String>,
    /// Supply rate (APY)
    #[serde(default)]
    pub supply_rate: Option<String>,
    /// Total rate (rewards + supply)
    #[serde(default)]
    pub total_rate: Option<String>,
    /// Liquidity supply data
    #[serde(default)]
    pub liquidity_supply_data: Option<LiquiditySupplyData>,
    /// Rewards information
    #[serde(default)]
    pub rewards: Option<Vec<serde_json::Value>>,
}

/// Asset information with price data
#[derive(Deserialize, Debug, Serialize)]
pub struct AssetInfo {
    /// Asset address
    pub address: String,
    /// Chain ID
    #[serde(rename = "chainId")]
    pub chain_id: String,
    /// Asset name
    pub name: String,
    /// Asset symbol
    pub symbol: String,
    /// Asset decimals
    pub decimals: u8,
    /// Logo URL
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    /// Current price in USD
    pub price: String,
    /// Coingecko ID
    #[serde(rename = "coingeckoId")]
    pub coingecko_id: String,
}

/// Liquidity supply data
#[derive(Deserialize, Debug, Serialize)]
pub struct LiquiditySupplyData {
    /// Whether mode with interest is enabled
    #[serde(default)]
    pub mode_with_interest: Option<bool>,
    /// Total supply
    #[serde(default)]
    pub supply: Option<String>,
    /// Withdrawal limit
    #[serde(default)]
    pub withdrawal_limit: Option<String>,
    /// Last update timestamp
    #[serde(default)]
    pub last_update_timestamp: Option<String>,
    /// Expand percent
    #[serde(default)]
    pub expand_percent: Option<u32>,
    /// Expand duration
    #[serde(default)]
    pub expand_duration: Option<String>,
    /// Base withdrawal limit
    #[serde(default)]
    pub base_withdrawal_limit: Option<String>,
    /// Withdrawable until limit
    #[serde(default)]
    pub withdrawable_until_limit: Option<String>,
    /// Currently withdrawable amount
    #[serde(default)]
    pub withdrawable: Option<String>,
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
    #[instrument(
        name = "lend_earn_tokens_tool_call",
        skip(self),
        fields(
            tool_name = "lend_earn_tokens",
            symbol = ?args.symbol,
            mint_address = ?args.mint_address
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[LendEarnTokensTool] Starting tool execution with OpenTelemetry tracing");
        let start_time = Instant::now();
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

        let total_execution_time = start_time.elapsed().as_millis() as u32;
        info!(
            "[LendEarnTokensTool] Tool execution completed - total_time: {}ms, tokens_found: {}",
            total_execution_time,
            filtered_tokens.len()
        );

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
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LendEarnTokensError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        // Check content length to detect potential truncation
        if let Some(content_length) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    tracing::debug!(
                        "[LendEarnTokensTool] Response content length: {} bytes",
                        length
                    );
                    if length > 100_000 {
                        tracing::warn!(
                            "[LendEarnTokensTool] Large response detected ({} bytes), potential truncation risk",
                            length
                        );
                    }
                }
            }
        }

        // Get response as text first for debugging
        let response_text = response.text().await?;
        tracing::debug!(
            "[LendEarnTokensTool] Raw API response (first 500 chars): {}",
            &response_text[..500.min(response_text.len())]
        );

        // Try to parse the JSON normally
        let tokens: Vec<LendEarnToken> = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!("[LendEarnTokensTool] Failed to parse JSON: {}", e);
            tracing::error!(
                "[LendEarnTokensTool] Response was: {}",
                &response_text[..1000.min(response_text.len())]
            );
            LendEarnTokensError::JsonError(e)
        })?;

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
                // Filter by symbol with special handling for jUSDC/jlUSDC
                if let Some(symbol) = &args.symbol {
                    let search_symbol = symbol.to_lowercase();
                    let token_symbol = token.symbol.to_lowercase();

                    // Handle common variations for Jupiter lending tokens
                    let normalized_search = if search_symbol == "jusdc" {
                        "jlusdc" // Convert jUSDC to jlUSDC
                    } else {
                        &search_symbol
                    };

                    if !token_symbol.contains(normalized_search) {
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
