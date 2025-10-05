use anyhow::Result;
use reqwest::Client;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// The arguments for the Jupiter earnings tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterEarningsArgs {
    pub user_pubkey: String,
    pub position_address: Option<String>,
}

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

/// A custom error type for the Jupiter earnings tool.
#[derive(Debug, Error)]
pub enum JupiterEarningsError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
}

/// A `rig` tool for fetching Jupiter lending earnings.
/// This tool queries the Jupiter API to get earnings data for a user's lending positions.
#[derive(Deserialize, Serialize)]
pub struct JupiterEarningsTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterEarningsTool {
    const NAME: &'static str = "jupiter_earnings";
    type Error = JupiterEarningsError;
    type Args = JupiterEarningsArgs;
    type Output = String; // The tool will return a JSON string of earnings data.

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Fetch Jupiter lending earnings data for a user. This returns detailed information about earnings from lending positions including total deposits, withdrawals, current balance, and total earnings.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet to fetch earnings for."
                    },
                    "position_address": {
                        "type": "string",
                        "description": "Optional: The specific position address to fetch earnings for. If not provided, returns earnings for all positions."
                    }
                },
                "required": ["user_pubkey"],
            }),
        }
    }

    /// Executes the tool's logic: calls the Jupiter API to fetch earnings.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = Client::builder()
            .build()
            .map_err(JupiterEarningsError::HttpError)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());

        // Build URL with user parameter
        let mut url = format!(
            "https://lite-api.jup.ag/lend/v1/earn/earnings?user={}",
            args.user_pubkey
        );

        // Add position parameter if provided
        if let Some(position) = &args.position_address {
            url.push_str(&format!("&positions={position}"));
        }

        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(JupiterEarningsError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(JupiterEarningsError::ApiError(format!(
                "API returned status {status}: {error_text}"
            )));
        }

        let body = response
            .text()
            .await
            .map_err(JupiterEarningsError::HttpError)?;

        // Parse and validate the JSON response
        let earnings: Vec<JupiterEarnings> =
            serde_json::from_str(&body).map_err(JupiterEarningsError::JsonParse)?;

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

            summary.push(json!({
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
        let response = json!({
            "user_pubkey": args.user_pubkey,
            "position_filter": args.position_address,
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

        Ok(response.to_string())
    }
}
