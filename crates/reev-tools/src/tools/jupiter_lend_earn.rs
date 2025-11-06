//! Jupiter earn tool wrapper
//!
//! This tool provides AI agent access to Jupiter's earn functionality,
//! including fetching positions and earnings data.

use reev_protocols::jupiter::{get_earnings_summary, get_positions_summary};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, instrument};

// Import enhanced logging macros
use reev_flow::{log_tool_call, log_tool_completion};

/// The arguments for Jupiter earn tool, which will be provided by the AI model.
#[derive(Deserialize, Debug, Serialize)]
pub struct GetJupiterLendEarnPositionArgs {
    pub user_pubkey: String,
    pub position_address: Option<String>,
    pub operation: GetJupiterLendEarnPositionOperation,
}

/// Jupiter earn operations
#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GetJupiterLendEarnPositionOperation {
    Positions,
    Earnings,
    Both,
}

/// A custom error type for the Jupiter earn tool.
#[derive(Debug, Error)]
pub enum GetJupiterLendEarnPositionError {
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
pub struct GetJupiterLendEarnPositionTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for GetJupiterLendEarnPositionTool {
    const NAME: &'static str = "get_jupiter_lend_earn_position";
    type Error = GetJupiterLendEarnPositionError;
    type Args = GetJupiterLendEarnPositionArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Access Jupiter's earn functionality including positions and earnings data. This tool can fetch lending positions, earnings history, or both in a single call. NOTE: If you need current token prices or APY information, use get_jupiter_lend_earn_tokens tool first.".to_string(),
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
    #[instrument(
        name = "jupiter_earn_tool_call",
        skip(self),
        fields(
            tool_name = "get_jupiter_lend_earn_position",
            user_pubkey = %args.user_pubkey,
            operation = ?args.operation,
            position_address = ?args.position_address
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Use enhanced logging macro for consistent otel tracking
        log_tool_call!("get_jupiter_lend_earn_position", &args);

        info!("[GetJupiterLendEarnPosition] Starting tool execution with OpenTelemetry tracing");
        let start_time = Instant::now();
        // Validate user pubkey
        if args.user_pubkey.is_empty() {
            return Err(GetJupiterLendEarnPositionError::InvalidUserPubkey(
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
            GetJupiterLendEarnPositionOperation::Positions => {
                info!(
                    "[GetJupiterLendEarnPosition] Calling get_positions_summary for user: {}",
                    user_pubkey
                );
                let positions = get_positions_summary(user_pubkey.clone()).await?;
                info!(
                    "[GetJupiterLendEarnPosition] Positions result: {}",
                    serde_json::to_string_pretty(&positions).unwrap_or_default()
                );
                json!({
                    "operation": "positions",
                    "data": positions
                })
            }
            GetJupiterLendEarnPositionOperation::Earnings => {
                info!(
                    "[GetJupiterLendEarnPosition] Calling get_earnings_summary for user: {}, position: {:?}",
                    user_pubkey, args.position_address
                );
                let earnings =
                    get_earnings_summary(user_pubkey.clone(), args.position_address.clone())
                        .await?;
                info!(
                    "[GetJupiterLendEarnPosition] Earnings result: {}",
                    serde_json::to_string_pretty(&earnings).unwrap_or_default()
                );
                json!({
                    "operation": "earnings",
                    "data": earnings
                })
            }
            GetJupiterLendEarnPositionOperation::Both => {
                info!(
                    "[GetJupiterLendEarnPosition] Calling both operations for user: {}",
                    user_pubkey
                );
                let positions = get_positions_summary(user_pubkey.clone()).await?;
                info!(
                    "[GetJupiterLendEarnPosition] Both - Positions result: {}",
                    serde_json::to_string_pretty(&positions).unwrap_or_default()
                );
                let earnings =
                    get_earnings_summary(user_pubkey.clone(), args.position_address.clone())
                        .await?;
                info!(
                    "[GetJupiterLendEarnPosition] Both - Earnings result: {}",
                    serde_json::to_string_pretty(&earnings).unwrap_or_default()
                );
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
            "tool": "get_jupiter_lend_earn_position",
            "user_pubkey": user_pubkey,
            "position_filter": args.position_address,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "result": result
        });

        let total_execution_time = start_time.elapsed().as_millis() as u64;

        // Log tool completion with enhanced otel
        log_tool_completion!(
            "get_jupiter_lend_earn_position",
            total_execution_time,
            &response,
            true
        );

        info!(
            "[GetJupiterLendEarnPosition] Tool execution completed - total_time: {}ms, operation: {:?}",
            total_execution_time, args.operation
        );

        info!(
            "[GetJupiterLendEarnPosition] Final response: {}",
            serde_json::to_string_pretty(&response).unwrap_or_default()
        );
        Ok(response.to_string())
    }
}
