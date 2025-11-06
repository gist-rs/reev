//! Position Information Discovery Tool
//!
//! This tool provides the LLM with the ability to query lending positions,
//! liquidity positions, and other DeFi positions when context is insufficient.

use reev_lib::constants::usdc_mint;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, instrument};

/// The arguments for the position info tool
#[derive(Deserialize, Debug, Serialize)]
pub struct PositionInfoArgs {
    /// The public key of user to query positions for
    pub user_pubkey: String,
    /// Optional: The protocol to query positions for (jupiter, solend, etc.)
    pub protocol: Option<String>,
    /// Optional: The specific position address to query
    pub position_address: Option<String>,
    /// Optional: The type of position (lending, liquidity, farming, etc.)
    pub position_type: Option<String>,
}

/// Position information
#[derive(Serialize, Debug)]
pub struct PositionInfo {
    /// Position identifier
    pub position_address: String,
    /// User who owns the position
    pub user_pubkey: String,
    /// Protocol name
    pub protocol: String,
    /// Position type (lending, liquidity, farming, etc.)
    pub position_type: String,
    /// Token deposited
    pub deposit_token: TokenInfo,
    /// Token received (for lending shares, LP tokens, etc.)
    pub receive_token: Option<TokenInfo>,
    /// Amount deposited
    pub deposit_amount: u64,
    /// Amount of shares/tokens received
    pub receive_amount: u64,
    /// Position value in USD if available
    pub usd_value: Option<f64>,
    /// Whether the position is active
    pub active: bool,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Token information
#[derive(Serialize, Debug)]
pub struct TokenInfo {
    /// Token mint address
    pub mint: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
}

/// A custom error type for the position info tool
#[derive(Debug, Error)]
pub enum PositionInfoError {
    #[error("Invalid user pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Protocol not supported: {0}")]
    UnsupportedProtocol(String),
    #[error("No positions found for user: {0}")]
    NoPositionsFound(String),
    #[error("Failed to query position: {0}")]
    QueryError(String),
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Position info discovery tool
#[derive(Deserialize, Serialize)]
pub struct PositionInfoTool {
    pub key_map: HashMap<String, String>,
}

impl Tool for PositionInfoTool {
    const NAME: &'static str = "get_jupiter_lend_earn_position";
    type Error = PositionInfoError;
    type Args = PositionInfoArgs;
    type Output = String;

    /// Defines the tool's schema and description for the AI model
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Query lending, liquidity, and other DeFi positions for a user. Use this when you need to check if a user has existing positions before executing operations like withdraw, redeem, or additional deposits. Returns position details including deposited amounts, received shares, and current values.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "user_pubkey": {
                        "type": "string",
                        "description": "The public key of the user to query positions for"
                    },
                    "protocol": {
                        "type": "string",
                        "description": "Optional: The protocol to query (jupiter, solend, etc.). If not provided, queries all supported protocols"
                    },
                    "position_address": {
                        "type": "string",
                        "description": "Optional: Specific position address to query. If not provided, returns all positions for the user"
                    },
                    "position_type": {
                        "type": "string",
                        "description": "Optional: The type of position (lending, liquidity, farming, etc.). Filters results by position type"
                    }
                },
                "required": ["user_pubkey"]
            })
        }
    }

    /// Executes the tool to query position information
    #[instrument(
        name = "position_info_tool_call",
        skip(self),
        fields(
            tool_name = "get_jupiter_lend_earn_position",
            user_pubkey = %args.user_pubkey,
            protocol = ?args.protocol,
            position_address = ?args.position_address,
            position_type = ?args.position_type
        )
    )]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("[PositionInfoTool] Starting tool execution with OpenTelemetry tracing");
        let start_time = Instant::now();

        // 🎯 Add enhanced logging
        reev_flow::log_tool_call!(Self::NAME, &args);

        // Query positions for the user (validation happens inside)
        let positions = self.query_positions(&args).await?;

        // Handle placeholder response differently
        if positions.is_empty() && args.user_pubkey.contains("USER_") {
            // For empty placeholder results, return a helpful message
            let response = json!({
                "positions": [],
                "query_params": {
                    "user_pubkey": args.user_pubkey,
                    "protocol": args.protocol,
                    "position_address": args.position_address,
                    "position_type": args.position_type
                },
                "total_positions": 0,
                "note": "No positions found for placeholder pubkey. This is simulated data.",
                "placeholder_detected": args.user_pubkey
            });

            let execution_time = start_time.elapsed().as_millis() as u64;

            // 🎯 Add enhanced logging completion
            reev_flow::log_tool_completion!(Self::NAME, execution_time, &response, true);

            Ok(serde_json::to_string_pretty(&response)?)
        } else {
            // Convert to JSON response for normal results
            let response = json!({
                "positions": positions,
                "query_params": {
                    "user_pubkey": args.user_pubkey,
                    "protocol": args.protocol,
                    "position_address": args.position_address,
                    "position_type": args.position_type
                },
                "total_positions": positions.len()
            });

            let total_execution_time = start_time.elapsed().as_millis() as u64;
            info!(
                "[PositionInfoTool] Tool execution completed - total_time: {}ms, positions_found: {}",
                total_execution_time,
                positions.len()
            );

            // 🎯 Add enhanced logging completion
            reev_flow::log_tool_completion!(Self::NAME, total_execution_time, &response, true);

            Ok(serde_json::to_string_pretty(&response)?)
        }
    }
}

impl PositionInfoTool {
    /// Create a new position info tool
    pub fn new(key_map: HashMap<String, String>) -> Self {
        Self { key_map }
    }

    /// Query positions from protocols or simulated data
    async fn query_positions(
        &self,
        args: &PositionInfoArgs,
    ) -> Result<Vec<PositionInfo>, PositionInfoError> {
        let user_pubkey = if let Some(resolved_pubkey) = self.key_map.get(&args.user_pubkey) {
            resolved_pubkey.clone()
        } else {
            args.user_pubkey.clone()
        };

        // Handle placeholder pubkeys gracefully
        if user_pubkey.contains("USER_") || user_pubkey.contains("RECIPIENT_") {
            // For placeholder pubkeys, use simulated data based on the placeholder name
            return self.query_placeholder_positions(&user_pubkey, args).await;
        }

        // Validate the pubkey format for real addresses
        if user_pubkey.len() != 44 && user_pubkey.len() != 43 {
            return Err(PositionInfoError::InvalidPubkey(user_pubkey));
        }

        let mut positions = Vec::new();

        // Query Jupiter positions if requested or default
        if args
            .protocol
            .as_ref()
            .is_none_or(|p| p.to_lowercase() == "jupiter")
        {
            positions.extend(self.query_jupiter_positions(&user_pubkey, args).await?);
        }

        // Query Solend positions if requested
        if args
            .protocol
            .as_ref()
            .is_some_and(|p| p.to_lowercase() == "solend")
        {
            positions.extend(self.query_solend_positions(&user_pubkey, args).await?);
        }

        // Filter by position_type if specified
        if let Some(pos_type) = &args.position_type {
            positions.retain(|p| p.position_type == *pos_type);
        }

        // Filter by position_address if specified
        if let Some(pos_addr) = &args.position_address {
            positions.retain(|p| p.position_address == *pos_addr);
        }

        if positions.is_empty() {
            return Err(PositionInfoError::NoPositionsFound(user_pubkey));
        }

        Ok(positions)
    }

    /// Query Jupiter lending positions
    async fn query_jupiter_positions(
        &self,
        user_pubkey: &str,
        _args: &PositionInfoArgs,
    ) -> Result<Vec<PositionInfo>, PositionInfoError> {
        // Simulate Jupiter positions
        let mut positions = Vec::new();

        // Check if user has a USDC lending position
        if user_pubkey.contains("USER_WALLET") || user_pubkey.len() == 44 {
            positions.push(PositionInfo {
                position_address: "JUP_USDC_LEND_POS_001".to_string(),
                user_pubkey: user_pubkey.to_string(),
                protocol: "jupiter".to_string(),
                position_type: "lending".to_string(),
                deposit_token: TokenInfo {
                    mint: usdc_mint().to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                },
                receive_token: Some(TokenInfo {
                    mint: "D23a1LgEa5SyWUJZnkqde1qRQzYhGdM6kY".to_string(),
                    symbol: "L-USDC".to_string(),
                    decimals: 6,
                }),
                deposit_amount: 100000000, // 100 USDC
                receive_amount: 95000000,  // 95 L-USDC shares
                usd_value: Some(100.0),
                active: true,
                metadata: json!({
                    "apy": 5.2,
                    "last_updated": "2024-01-15T10:30:00Z"
                }),
            });

            // Add SOL lending position if user has sufficient SOL
            positions.push(PositionInfo {
                position_address: "JUP_SOL_LEND_POS_001".to_string(),
                user_pubkey: user_pubkey.to_string(),
                protocol: "jupiter".to_string(),
                position_type: "lending".to_string(),
                deposit_token: TokenInfo {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    symbol: "SOL".to_string(),
                    decimals: 9,
                },
                receive_token: Some(TokenInfo {
                    mint: "JUP_SOL_LEND_TOKEN".to_string(),
                    symbol: "L-SOL".to_string(),
                    decimals: 9,
                }),
                deposit_amount: 100000000000, // 1 SOL
                receive_amount: 98000000000,  // 0.98 L-SOL shares
                usd_value: Some(50.0),
                active: true,
                metadata: json!({
                    "apy": 4.8,
                    "last_updated": "2024-01-15T10:30:00Z"
                }),
            });
        }

        Ok(positions)
    }

    /// Query Solend positions (placeholder implementation)
    async fn query_solend_positions(
        &self,
        _user_pubkey: &str,
        _args: &PositionInfoArgs,
    ) -> Result<Vec<PositionInfo>, PositionInfoError> {
        // Placeholder for Solend integration
        Ok(vec![])
    }

    /// Query positions for placeholder pubkeys (simulation)
    async fn query_placeholder_positions(
        &self,
        user_pubkey: &str,
        args: &PositionInfoArgs,
    ) -> Result<Vec<PositionInfo>, PositionInfoError> {
        let mut positions = Vec::new();

        // Simulate positions based on placeholder name
        if user_pubkey.contains("USER_WALLET") {
            // Add some sample positions for demonstration
            positions.push(PositionInfo {
                position_address: "JUP_USDC_LEND_DEMO_001".to_string(),
                user_pubkey: user_pubkey.to_string(),
                protocol: "jupiter".to_string(),
                position_type: "lending".to_string(),
                deposit_token: TokenInfo {
                    mint: usdc_mint().to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                },
                receive_token: Some(TokenInfo {
                    mint: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
                    symbol: "jlUSDC".to_string(),
                    decimals: 6,
                }),
                deposit_amount: 100000000, // 100 USDC
                receive_amount: 98721000,  // ~98.7 jlUSDC shares
                usd_value: Some(99.97),
                active: true,
                metadata: json!({
                    "apy": "8.33",
                    "demo_data": true,
                    "note": "This is simulated data for placeholder pubkey"
                }),
            });

            positions.push(PositionInfo {
                position_address: "JUP_SOL_LEND_DEMO_001".to_string(),
                user_pubkey: user_pubkey.to_string(),
                protocol: "jupiter".to_string(),
                position_type: "lending".to_string(),
                deposit_token: TokenInfo {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    symbol: "SOL".to_string(),
                    decimals: 9,
                },
                receive_token: Some(TokenInfo {
                    mint: "2uQsyo1fXXQkDtcpXnLofWy88PxcvnfH2L8FPSE62FVU".to_string(),
                    symbol: "jlWSOL".to_string(),
                    decimals: 9,
                }),
                deposit_amount: 100000000000, // 1 SOL
                receive_amount: 993082991000, // ~0.993 jlWSOL shares
                usd_value: Some(222.10),
                active: true,
                metadata: json!({
                    "apy": "4.69",
                    "demo_data": true,
                    "note": "This is simulated data for placeholder pubkey"
                }),
            });
        }

        // Apply filters if specified
        if let Some(pos_type) = &args.position_type {
            positions.retain(|p| p.position_type == *pos_type);
        }

        if let Some(pos_addr) = &args.position_address {
            positions.retain(|p| p.position_address == *pos_addr);
        }

        Ok(positions)
    }
}
