//! Jupiter earn tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn functionality,
//! including fetching positions and earnings data.

use crate::protocols::jupiter::{get_earnings_summary, get_positions_summary};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

/// The arguments for the Jupiter earn tool, which will be provided by the AI model.
#[derive(Deserialize, Debug)]
pub struct JupiterEarnArgs {
    pub user_pubkey: String,
    pub position_address: Option<String>,
    pub operation: JupiterEarnOperation,
}

/// Jupiter earn operations
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum JupiterEarnOperation {
    Positions,
    Earnings,
    Both,
}

/// A custom error type for the Jupiter earn tool.
#[derive(Debug, Error)]
pub enum JupiterEarnError {
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] anyhow::Error),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Invalid user pubkey: {0}")]
    InvalidUserPubkey(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// A `rig` tool for accessing Jupiter's earn functionality.
/// This tool provides a unified interface for positions and earnings data.
#[derive(Deserialize, Serialize)]
pub struct JupiterEarnTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for JupiterEarnTool {
    const NAME: &'static str = "jupiter_earn";
    type Error = JupiterEarnError;
    type Args = JupiterEarnArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Access Jupiter's earn functionality including positions and earnings data. This tool can fetch lending positions, earnings history, or both in a single call.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user's wallet to fetch data for."
                    },
                    "position_address": {
                        "type": "string",
                        "description": "Optional: The specific position address to filter by. If not provided, returns data for all positions."
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["positions", "earnings", "both"],
                        "description": "The operation to perform: 'positions' for lending positions, 'earnings' for earnings data, or 'both' for both."
                    }
                },
                "required": ["user_pubkey", "operation"],
            }),
        }
    }

    /// Executes the tool's logic: calls the appropriate Jupiter protocol handler.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate user pubkey
        if args.user_pubkey.is_empty() {
            return Err(JupiterEarnError::InvalidUserPubkey(
                "User pubkey cannot be empty".to_string(),
            ));
        }

        // Get the resolved user pubkey from key_map or use the provided one
        let user_pubkey = self
            .key_map
            .get("USER_WALLET_PUBKEY")
            .unwrap_or(&args.user_pubkey)
            .clone();

        // Execute the requested operation
        let result = match args.operation {
            JupiterEarnOperation::Positions => {
                let positions = get_positions_summary(user_pubkey.clone()).await?;
                json!({
                    "operation": "positions",
                    "data": positions
                })
            }
            JupiterEarnOperation::Earnings => {
                let earnings =
                    get_earnings_summary(user_pubkey.clone(), args.position_address.clone())
                        .await?;
                json!({
                    "operation": "earnings",
                    "data": earnings
                })
            }
            JupiterEarnOperation::Both => {
                let positions = get_positions_summary(user_pubkey.clone()).await?;
                let earnings =
                    get_earnings_summary(user_pubkey.clone(), args.position_address.clone())
                        .await?;
                json!({
                    "operation": "both",
                    "data": {
                        "positions": positions,
                        "earnings": earnings
                    }
                })
            }
        };

        // Create the final response
        let response = json!({
            "tool": "jupiter_earn",
            "user_pubkey": user_pubkey,
            "position_filter": args.position_address,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "result": result
        });

        Ok(response.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jupiter_earn_args_serialization() {
        let args = json!({
            "user_pubkey": "test_user_pubkey",
            "position_address": "test_position",
            "operation": "both"
        });

        let parsed: JupiterEarnArgs = serde_json::from_value(args).unwrap();
        assert_eq!(parsed.user_pubkey, "test_user_pubkey");
        assert_eq!(parsed.position_address, Some("test_position".to_string()));
        assert!(matches!(parsed.operation, JupiterEarnOperation::Both));
    }

    #[test]
    fn test_jupiter_earn_operation_enum() {
        let positions = json!("positions");
        let parsed: JupiterEarnOperation = serde_json::from_value(positions).unwrap();
        assert!(matches!(parsed, JupiterEarnOperation::Positions));

        let earnings = json!("earnings");
        let parsed: JupiterEarnOperation = serde_json::from_value(earnings).unwrap();
        assert!(matches!(parsed, JupiterEarnOperation::Earnings));

        let both = json!("both");
        let parsed: JupiterEarnOperation = serde_json::from_value(both).unwrap();
        assert!(matches!(parsed, JupiterEarnOperation::Both));
    }
}
