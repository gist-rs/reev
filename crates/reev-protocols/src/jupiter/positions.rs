//! Jupiter positions API handler
//!
//! This module provides functions to fetch Jupiter lending positions
//! for a given user wallet address.

use crate::{get_jupiter_config, jupiter::parse_json_response};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Fetch Jupiter lending positions for a user
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
///
/// # Returns
/// A vector of Jupiter positions with detailed information
pub async fn get_positions(user_pubkey: String) -> Result<Vec<JupiterPosition>> {
    let config = get_jupiter_config();
    let client = config.create_client()?;
    let url = format!("{}?users={}", config.positions_url(), user_pubkey);

    let request = client.get(&url).header("Accept", "application/json");

    let response = crate::jupiter::execute_request(request, config.max_retries).await?;

    let json_value = parse_json_response(response).await?;
    let positions: Vec<JupiterPosition> = serde_json::from_value(json_value)?;

    Ok(positions)
}

/// Get positions with formatted summary for easier consumption
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
///
/// # Returns
/// A summary response with positions formatted for display
pub async fn get_positions_summary(user_pubkey: String) -> Result<serde_json::Value> {
    let positions = get_positions(user_pubkey.clone()).await?;

    // Create a summary response with key information
    let mut summary = Vec::new();
    let total_positions = positions.len();
    let mut positions_with_balance = 0;

    for position in &positions {
        let underlying_balance_decimal = position.underlying_assets.parse::<f64>().unwrap_or(0.0)
            / 10_f64.powi(position.token.decimals as i32);
        let asset_price = position.token.asset.price.parse::<f64>().unwrap_or(0.0);
        let usd_value = underlying_balance_decimal * asset_price;
        let supply_rate = position.token.supply_rate.parse::<f64>().unwrap_or(0.0) / 10000.0;
        let total_rate = position.token.total_rate.parse::<f64>().unwrap_or(0.0) / 10000.0;

        if underlying_balance_decimal > 0.0 {
            positions_with_balance += 1;
        }

        summary.push(serde_json::json!({
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
    let response = serde_json::json!({
        "user_pubkey": user_pubkey,
        "total_positions": total_positions,
        "positions_with_balance": positions_with_balance,
        "summary": summary,
        "raw_positions": positions
    });

    Ok(response)
}

/// Get positions for multiple users
///
/// # Arguments
/// * `user_pubkeys` - Vector of user wallet public keys
///
/// # Returns
/// A hashmap mapping user pubkeys to their positions
pub async fn get_multiple_positions(
    user_pubkeys: Vec<String>,
) -> Result<HashMap<String, Vec<JupiterPosition>>> {
    let config = get_jupiter_config();
    let client = config.create_client()?;
    let users_param = user_pubkeys.join(",");
    let url = format!("{}?users={}", config.positions_url(), users_param);

    let request = client.get(&url).header("Accept", "application/json");

    let response = crate::jupiter::execute_request(request, config.max_retries).await?;

    let json_value = parse_json_response(response).await?;
    let positions: Vec<JupiterPosition> = serde_json::from_value(json_value)?;

    // Group positions by user
    let mut user_positions: HashMap<String, Vec<JupiterPosition>> = HashMap::new();
    for position in positions {
        let user = position.owner_address.clone();
        user_positions.entry(user).or_default().push(position);
    }

    Ok(user_positions)
}
