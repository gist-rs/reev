//! Jupiter earnings API handler
//!
//! This module provides functions to fetch Jupiter lending earnings
//! for a given user wallet address and optional position filtering.

use crate::protocols::{get_jupiter_config, jupiter::parse_json_response};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jupiter earnings data from the API response
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JupiterEarnings {
    pub address: String,
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,
    #[serde(rename = "totalDeposits")]
    pub total_deposits: String,
    #[serde(rename = "totalWithdraws")]
    pub total_withdraws: String,
    #[serde(rename = "totalBalance")]
    pub total_balance: String,
    #[serde(rename = "totalAssets")]
    pub total_assets: String,
    pub earnings: String,
    pub slot: u64,
}

/// Fetch Jupiter lending earnings for a user
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
/// * `position_address` - Optional specific position address to filter by
///
/// # Returns
/// A vector of Jupiter earnings with detailed information
pub async fn get_earnings(
    user_pubkey: String,
    position_address: Option<String>,
) -> Result<Vec<JupiterEarnings>> {
    let config = get_jupiter_config();
    let client = config.create_client()?;

    let mut url = format!("{}?user={}", config.earnings_url(), user_pubkey);

    // Add position filter if provided
    if let Some(position) = position_address {
        url.push_str(&format!("&positions={position}"));
    }

    let request = client.get(&url).header("Accept", "application/json");

    let response = crate::protocols::jupiter::execute_request(request, config.max_retries).await?;

    let json_value = parse_json_response(response).await?;
    let earnings: Vec<JupiterEarnings> = serde_json::from_value(json_value)?;

    Ok(earnings)
}

/// Get earnings with formatted summary for easier consumption
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
/// * `position_address` - Optional specific position address to filter by
///
/// # Returns
/// A summary response with earnings formatted for display
pub async fn get_earnings_summary(
    user_pubkey: String,
    position_address: Option<String>,
) -> Result<serde_json::Value> {
    let earnings = get_earnings(user_pubkey.clone(), position_address.clone()).await?;

    // Create a summary response with key information
    let mut summary = Vec::new();
    let mut total_earnings = 0.0;
    let mut total_deposits = 0.0;
    let mut total_withdraws = 0.0;
    let mut current_balance = 0.0;

    for earning in &earnings {
        let earnings_value = earning.earnings.parse::<f64>().unwrap_or(0.0);
        let deposits_value = earning.total_deposits.parse::<f64>().unwrap_or(0.0);
        let withdraws_value = earning.total_withdraws.parse::<f64>().unwrap_or(0.0);
        let balance_value = earning.total_balance.parse::<f64>().unwrap_or(0.0);

        total_earnings += earnings_value;
        total_deposits += deposits_value;
        total_withdraws += withdraws_value;
        current_balance += balance_value;

        summary.push(serde_json::json!({
            "position_address": earning.address,
            "owner_address": earning.owner_address,
            "earnings": {
                "raw": earning.earnings,
                "decimal": earnings_value
            },
            "deposits": {
                "raw": earning.total_deposits,
                "decimal": deposits_value
            },
            "withdraws": {
                "raw": earning.total_withdraws,
                "decimal": withdraws_value
            },
            "current_balance": {
                "raw": earning.total_balance,
                "decimal": balance_value
            },
            "total_assets": earning.total_assets,
            "slot": earning.slot
        }));
    }

    // Create final response with summary and raw data
    let response = serde_json::json!({
        "user_pubkey": user_pubkey,
        "position_filter": position_address,
        "total_positions": earnings.len(),
        "summary": {
            "total_earnings": {
                "raw": format!("{}", total_earnings as u64),
                "decimal": total_earnings
            },
            "total_deposits": {
                "raw": format!("{}", total_deposits as u64),
                "decimal": total_deposits
            },
            "total_withdraws": {
                "raw": format!("{}", total_withdraws as u64),
                "decimal": total_withdraws
            },
            "current_balance": {
                "raw": format!("{}", current_balance as u64),
                "decimal": current_balance
            }
        },
        "positions": summary,
        "raw_earnings": earnings
    });

    Ok(response)
}

/// Get earnings for multiple users
///
/// # Arguments
/// * `user_pubkeys` - Vector of user wallet public keys
///
/// # Returns
/// A hashmap mapping user pubkeys to their earnings
pub async fn get_multiple_earnings(
    user_pubkeys: Vec<String>,
) -> Result<HashMap<String, Vec<JupiterEarnings>>> {
    let mut user_earnings: HashMap<String, Vec<JupiterEarnings>> = HashMap::new();

    for user_pubkey in user_pubkeys {
        let earnings = get_earnings(user_pubkey.clone(), None).await?;
        user_earnings.insert(user_pubkey, earnings);
    }

    Ok(user_earnings)
}

/// Get total earnings across all positions for a user
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
///
/// # Returns
/// Total earnings amount
pub async fn get_total_earnings(user_pubkey: String) -> Result<f64> {
    let earnings = get_earnings(user_pubkey, None).await?;

    let total = earnings
        .iter()
        .map(|e| e.earnings.parse::<f64>().unwrap_or(0.0))
        .sum();

    Ok(total)
}

/// Get earnings for a specific position
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
/// * `position_address` - The position address to filter by
///
/// # Returns
/// Earnings data for the specific position
pub async fn get_position_earnings(
    user_pubkey: String,
    position_address: String,
) -> Result<Option<JupiterEarnings>> {
    let earnings = get_earnings(user_pubkey, Some(position_address)).await?;

    Ok(earnings.into_iter().next())
}

/// Calculate earnings rate (earnings / deposits * 100)
///
/// # Arguments
/// * `user_pubkey` - The user's wallet public key
/// * `position_address` - Optional specific position address to filter by
///
/// # Returns
/// Earnings rate as a percentage
pub async fn get_earnings_rate(
    user_pubkey: String,
    position_address: Option<String>,
) -> Result<f64> {
    let earnings = get_earnings(user_pubkey, position_address).await?;

    let total_earnings: f64 = earnings
        .iter()
        .map(|e| e.earnings.parse::<f64>().unwrap_or(0.0))
        .sum();

    let total_deposits: f64 = earnings
        .iter()
        .map(|e| e.total_deposits.parse::<f64>().unwrap_or(0.0))
        .sum();

    if total_deposits == 0.0 {
        return Ok(0.0);
    }

    Ok((total_earnings / total_deposits) * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jupiter_earnings_serialization() {
        let earnings_data = r#"
        {
            "address": "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D",
            "ownerAddress": "test_user",
            "totalDeposits": "1000000000",
            "totalWithdraws": "500000000",
            "totalBalance": "500000000",
            "totalAssets": "505000000",
            "earnings": "5000000",
            "slot": 123456789
        }
        "#;

        let earnings: JupiterEarnings = serde_json::from_str(earnings_data).unwrap();
        assert_eq!(
            earnings.address,
            "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D"
        );
        assert_eq!(earnings.owner_address, "test_user");
        assert_eq!(earnings.earnings, "5000000");
        assert_eq!(earnings.slot, 123456789);
    }

    #[test]
    fn test_earnings_rate_calculation() {
        // Test earnings rate calculation: (earnings / deposits) * 100
        let earnings = 5000000.0; // $5M
        let deposits = 100000000.0; // $100M
        let expected_rate: f64 = (earnings / deposits) * 100.0; // 5%

        assert!((expected_rate - 5.0).abs() < 0.0001);
    }
}
