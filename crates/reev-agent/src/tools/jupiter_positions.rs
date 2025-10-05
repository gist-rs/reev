use anyhow::Result;
use reqwest::Client;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// The arguments for the Jupiter positions tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterPositionsArgs {
    pub user_pubkey: String,
}

/// Jupiter token information from the API response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JupiterToken {
    pub id: u32,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "assetAddress")]
    pub asset_address: String,
    pub asset: JupiterAsset,
    #[serde(rename = "totalAssets")]
    pub total_assets: String,
    #[serde(rename = "totalSupply")]
    pub total_supply: String,
    #[serde(rename = "convertToShares")]
    pub convert_to_shares: String,
    #[serde(rename = "convertToAssets")]
    pub convert_to_assets: String,
    #[serde(rename = "rewardsRate")]
    pub rewards_rate: String,
    #[serde(rename = "supplyRate")]
    pub supply_rate: String,
    #[serde(rename = "totalRate")]
    pub total_rate: String,
    #[serde(rename = "rebalanceDifference")]
    pub rebalance_difference: String,
    #[serde(rename = "liquiditySupplyData")]
    pub liquidity_supply_data: LiquiditySupplyData,
    pub rewards: Vec<serde_json::Value>,
}

/// Asset information for a Jupiter token
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JupiterAsset {
    pub address: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    pub price: String,
    #[serde(rename = "coingeckoId")]
    pub coingecko_id: String,
}

/// Liquidity supply data for a Jupiter token
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LiquiditySupplyData {
    #[serde(rename = "modeWithInterest")]
    pub mode_with_interest: bool,
    pub supply: String,
    #[serde(rename = "withdrawalLimit")]
    pub withdrawal_limit: String,
    #[serde(rename = "lastUpdateTimestamp")]
    pub last_update_timestamp: String,
    #[serde(rename = "expandPercent")]
    pub expand_percent: serde_json::Value,
    #[serde(rename = "expandDuration")]
    pub expand_duration: serde_json::Value,
    #[serde(rename = "baseWithdrawalLimit")]
    pub base_withdrawal_limit: String,
    #[serde(rename = "withdrawableUntilLimit")]
    pub withdrawable_until_limit: String,
    pub withdrawable: String,
}

/// A single position from the Jupiter API response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JupiterPosition {
    pub token: JupiterToken,
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,
    pub shares: String,
    #[serde(rename = "underlyingAssets")]
    pub underlying_assets: String,
    #[serde(rename = "underlyingBalance")]
    pub underlying_balance: String,
    pub allowance: String,
}

/// A custom error type for the Jupiter positions tool.
#[derive(Debug, Error)]
pub enum JupiterPositionsError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
}

/// A `rig` tool for fetching Jupiter lending positions.
/// This tool queries the Jupiter API to get all lending positions for a user.
#[derive(Deserialize, Serialize)]
pub struct JupiterPositionsTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterPositionsTool {
    const NAME: &'static str = "jupiter_positions";
    type Error = JupiterPositionsError;
    type Args = JupiterPositionsArgs;
    type Output = String; // The tool will return a JSON string of positions data.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Fetch all Jupiter lending positions for a user. This returns detailed information about all lending positions including token details, balances, rates, and liquidity data.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet to fetch positions for."
                    }
                },
                "required": ["user_pubkey"],
            }),
        }
    }

    /// Executes the tool's logic: calls the Jupiter API to fetch positions.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = Client::builder()
            .build()
            .map_err(JupiterPositionsError::HttpError)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());

        let url = format!(
            "https://lite-api.jup.ag/lend/v1/earn/positions?users={}",
            args.user_pubkey
        );

        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(JupiterPositionsError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(JupiterPositionsError::ApiError(format!(
                "API returned status {status}: {error_text}"
            )));
        }

        let body = response
            .text()
            .await
            .map_err(JupiterPositionsError::HttpError)?;

        // Parse and validate the JSON response
        let positions: Vec<JupiterPosition> =
            serde_json::from_str(&body).map_err(JupiterPositionsError::JsonParse)?;

        // Create a summary response with key information
        let mut summary = Vec::new();
        for position in &positions {
            let underlying_balance_decimal =
                position.underlying_assets.parse::<f64>().unwrap_or(0.0)
                    / 10_f64.powi(position.token.decimals as i32);
            let asset_price = position.token.asset.price.parse::<f64>().unwrap_or(0.0);
            let usd_value = underlying_balance_decimal * asset_price;
            let supply_rate = position.token.supply_rate.parse::<f64>().unwrap_or(0.0) / 10000.0;
            let total_rate = position.token.total_rate.parse::<f64>().unwrap_or(0.0) / 10000.0;

            summary.push(json!({
                "token": {
                    "symbol": position.token.symbol,
                    "name": position.token.name,
                    "address": position.token.address,
                    "asset_address": position.token.asset_address,
                    "decimals": position.token.decimals
                },
                "asset": {
                    "symbol": position.token.asset.symbol,
                    "name": position.token.asset.name,
                    "price": position.token.asset.price,
                    "logo_url": position.token.asset.logo_url
                },
                "position": {
                    "shares": position.shares,
                    "underlying_assets": position.underlying_assets,
                    "underlying_balance": position.underlying_balance,
                    "underlying_balance_decimal": underlying_balance_decimal,
                    "usd_value": usd_value,
                    "allowance": position.allowance
                },
                "rates": {
                    "supply_rate_pct": supply_rate * 100.0,
                    "total_rate_pct": total_rate * 100.0,
                    "rewards_rate": position.token.rewards_rate
                },
                "liquidity": {
                    "total_assets": position.token.total_assets,
                    "withdrawable": position.token.liquidity_supply_data.withdrawable,
                    "withdrawal_limit": position.token.liquidity_supply_data.withdrawal_limit
                }
            }));
        }

        // Create final response with summary and raw data
        let response = json!({
            "user_pubkey": args.user_pubkey,
            "total_positions": positions.len(),
            "positions_with_balance": positions.iter().filter(|p| {
                p.underlying_balance.parse::<u64>().unwrap_or(0) > 0
            }).count(),
            "summary": summary,
            "raw_positions": positions
        });

        Ok(response.to_string())
    }
}
