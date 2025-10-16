//! Financial scenarios and complex mock data generation
//!
//! This module provides predefined financial scenarios and utilities for
//! generating complex mock data for testing and benchmarking.

use rand::Rng;

use super::financial_types::{FinancialTransaction, MarketDepth, MarketOrder};
use super::jupiter_types::{JupiterLendingPosition, JupiterSwapQuote};

/// Mock financial data scenarios
pub struct FinancialScenarios;

impl FinancialScenarios {
    /// Generate a DeFi trading scenario with realistic parameters
    pub fn defi_trading_scenario(
        &self,
        rng: &mut rand::rngs::StdRng,
    ) -> (Vec<JupiterSwapQuote>, Vec<FinancialTransaction>) {
        let mut quotes = Vec::new();
        let mut transactions = Vec::new();

        // Generate 3-5 swap quotes
        for i in 0..rng.gen_range(3..=5) {
            let input_amount = rng.gen_range(1_000_000..10_000_000); // 1-10 USDC
            let output_amount = rng.gen_range(50_000_000..500_000_000); // 0.05-0.5 SOL
            let price_impact = rng.gen_range(0.001..0.05); // 0.1-5% price impact

            quotes.push(JupiterSwapQuote {
                input_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                output_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                input_amount,
                output_amount,
                price_impact_pct: price_impact * 100.0,
                slippage_bps: rng.gen_range(10..100),
                routes: vec![format!("route_{}", i)],
                fee_amount: input_amount / 1000, // 0.1% fee
                fee_pct: 0.1,
            });

            transactions.push(FinancialTransaction {
                id: format!("tx_{i}"),
                transaction_type: "swap".to_string(),
                input_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                output_mint: "So11111111111111111111111111111111111111112".to_string(),
                input_amount,
                output_amount,
                price_usd: output_amount as f64 / 100_000_000.0 * 150.0, // Assuming $150 SOL price
                timestamp: chrono::Utc::now(),
            });
        }

        (quotes, transactions)
    }

    /// Generate a high-frequency trading scenario
    pub fn high_frequency_trading_scenario(
        &self,
        rng: &mut rand::rngs::StdRng,
    ) -> (MarketDepth, Vec<FinancialTransaction>) {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        let mut transactions = Vec::new();

        // Generate market depth
        let base_price = 150.0; // SOL price in USD
        for i in 0..10 {
            let bid_price = base_price - (i as f64 * 0.01);
            let ask_price = base_price + (i as f64 * 0.01);

            bids.push(MarketOrder {
                price: bid_price,
                amount: rng.gen_range(100..1000),
                is_bid: true,
            });

            asks.push(MarketOrder {
                price: ask_price,
                amount: rng.gen_range(100..1000),
                is_bid: false,
            });
        }

        // Generate high-frequency transactions
        for i in 0..50 {
            let is_buy = rng.gen_bool(0.5);
            let price = if is_buy {
                asks[rng.gen_range(0..asks.len())].price
            } else {
                bids[rng.gen_range(0..bids.len())].price
            };

            transactions.push(FinancialTransaction {
                id: format!("hft_tx_{i}"),
                transaction_type: if is_buy { "buy" } else { "sell" }.to_string(),
                input_mint: if is_buy {
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
                } else {
                    "So11111111111111111111111111111111111111112".to_string()
                },
                output_mint: if is_buy {
                    "So11111111111111111111111111111111111111112".to_string()
                } else {
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
                },
                input_amount: rng.gen_range(1_000_000..10_000_000),
                output_amount: (rng.gen_range(1_000_000..10_000_000) as f64 / price) as u64,
                price_usd: price,
                timestamp: chrono::Utc::now(),
            });
        }

        (MarketDepth { bids, asks }, transactions)
    }

    /// Generate lending positions scenario
    pub fn lending_positions_scenario(
        &self,
        rng: &mut rand::rngs::StdRng,
        num_positions: usize,
    ) -> Vec<JupiterLendingPosition> {
        let mut positions = Vec::new();

        for i in 0..num_positions {
            let deposit_amount = rng.gen_range(1_000_000..100_000_000);
            let apy = rng.gen_range(2.0..15.0);
            let time_elapsed = rng.gen_range(30..365); // days

            positions.push(JupiterLendingPosition {
                user_pubkey: format!("user_{i}"),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                deposit_amount,
                current_amount: deposit_amount
                    + (deposit_amount as f64 * apy / 100.0 * time_elapsed as f64 / 365.0) as u64,
                value_usd: deposit_amount as f64 / 1_000_000.0,
                apy,
                accrued_interest: (deposit_amount as f64 * apy / 100.0 * time_elapsed as f64
                    / 365.0) as u64,
                last_updated: chrono::Utc::now(),
            });
        }

        positions
    }
}

/// Utility functions for generating realistic financial parameters
pub struct FinancialUtils;

impl FinancialUtils {
    /// Generate a realistic price with some volatility
    pub fn generate_price(base_price: f64, volatility: f64, rng: &mut rand::rngs::StdRng) -> f64 {
        let change = rng.gen_range(-volatility..volatility);
        base_price * (1.0 + change)
    }

    /// Generate realistic slippage based on trade size
    pub fn calculate_slippage(
        trade_size: u64,
        market_depth: u64,
        rng: &mut rand::rngs::StdRng,
    ) -> f64 {
        let base_slippage = (trade_size as f64 / market_depth as f64) * 0.01;
        let random_factor = rng.gen_range(0.8..1.2);
        base_slippage * random_factor
    }

    /// Generate realistic APY based on market conditions
    pub fn generate_apy(
        base_apy: f64,
        market_volatility: f64,
        rng: &mut rand::rngs::StdRng,
    ) -> f64 {
        let volatility_adjustment = market_volatility * rng.gen_range(-0.5..0.5);
        (base_apy + volatility_adjustment).max(0.0)
    }
}
