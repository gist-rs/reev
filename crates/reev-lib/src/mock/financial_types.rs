//! Financial market and transaction types for mock data generation
//!
//! This module contains types related to financial markets, trading,
//! and account states used for testing and benchmarking.

use serde::{Deserialize, Serialize};

/// Account state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: Option<String>,
    pub data: Vec<u8>,
    pub executable: bool,
}

/// Market order information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    pub price: f64,
    pub amount: u64,
    pub is_bid: bool,
}

/// Market depth information with bids and asks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub bids: Vec<MarketOrder>,
    pub asks: Vec<MarketOrder>,
}

/// Financial transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialTransaction {
    pub id: String,
    pub transaction_type: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_usd: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
