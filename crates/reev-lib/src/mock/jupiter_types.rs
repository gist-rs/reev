//! Jupiter-related types for mock data generation
//!
//! This module contains all Jupiter protocol related structures used for
//! testing and benchmarking, including swap quotes, lending positions,
//! and various Jupiter API response types.

use serde::{Deserialize, Serialize};

/// Jupiter swap quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSwapQuote {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact_pct: f64,
    pub slippage_bps: u16,
    pub routes: Vec<String>, // Simplified for mock
    pub fee_amount: u64,
    pub fee_pct: f64,
}

/// Jupiter lending position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLendingPosition {
    pub user_pubkey: String,
    pub mint: String,
    pub deposit_amount: u64,
    pub current_amount: u64,
    pub value_usd: f64,
    pub apy: f64,
    pub accrued_interest: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Jupiter token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterToken {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub asset_address: String,
    pub decimals: u8,
}

/// Jupiter asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterAsset {
    pub symbol: String,
    pub name: String,
    pub price: String,
    pub logo_url: String,
}

/// Jupiter position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionData {
    pub shares: String,
    pub underlying_assets: String,
    pub underlying_balance: String,
    pub underlying_balance_decimal: f64,
    pub usd_value: f64,
    pub allowance: String,
}

/// Jupiter rates information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterRates {
    pub supply_rate_pct: f64,
    pub total_rate_pct: f64,
    pub rewards_rate: String,
}

/// Jupiter liquidity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLiquidity {
    pub total_assets: String,
    pub withdrawable: String,
    pub withdrawal_limit: String,
}

/// Complete Jupiter position summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionSummary {
    pub token: JupiterToken,
    pub asset: JupiterAsset,
    pub position: JupiterPositionData,
    pub rates: JupiterRates,
    pub liquidity: JupiterLiquidity,
}

/// Jupiter position earnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionEarnings {
    pub raw: String,
    pub decimal: f64,
}

/// Jupiter position balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionBalance {
    pub raw: String,
    pub decimal: f64,
}

/// Individual Jupiter position item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionItem {
    pub position_address: String,
    pub owner_address: String,
    pub earnings: JupiterPositionEarnings,
    pub deposits: JupiterPositionBalance,
    pub withdraws: JupiterPositionBalance,
    pub current_balance: JupiterPositionBalance,
    pub total_assets: String,
    pub slot: u64,
}

/// Raw Jupiter earnings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawJupiterEarnings {
    #[serde(rename = "address")]
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
    #[serde(rename = "earnings")]
    pub earnings: String,
    #[serde(rename = "slot")]
    pub slot: u64,
}
