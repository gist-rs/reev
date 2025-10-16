//! Core mock data generator for testing and benchmarks
//!
//! This module provides the core MockGenerator struct and basic generation methods.
//! Complex types and scenarios are moved to separate modules for better organization.

use rand::Rng;
use solana_sdk::pubkey::Pubkey;

use super::financial_types::{AccountState, MarketDepth, MarketOrder};
use super::jupiter_types::{JupiterLendingPosition, JupiterSwapQuote};
use crate::agent::RawAccountMeta;

/// Mock data generator for creating realistic test data
pub struct MockGenerator {
    rng: rand::rngs::StdRng,
}

impl Default for MockGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MockGenerator {
    /// Create a new mock data generator with a random seed
    pub fn new() -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    /// Create a new mock data generator with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    /// Generate a random Solana public key
    pub fn random_pubkey(&mut self) -> Pubkey {
        let mut bytes = [0u8; 32];
        self.rng.fill(&mut bytes);
        Pubkey::new_from_array(bytes)
    }

    /// Generate a random amount within a range
    pub fn random_amount(&mut self, min: u64, max: u64) -> u64 {
        self.rng.gen_range(min..=max)
    }

    /// Generate a random USDC amount (1-10,000 USDC)
    pub fn random_usdc_amount(&mut self) -> u64 {
        self.random_amount(1_000_000, 10_000_000_000) // USDC has 6 decimals
    }

    /// Generate a random SOL amount (0.001-10 SOL)
    pub fn random_sol_amount(&mut self) -> u64 {
        self.random_amount(1_000_000, 10_000_000_000) // SOL has 9 decimals
    }

    /// Generate a random slippage basis points (10-1000 bps = 0.1%-10%)
    pub fn random_slippage_bps(&mut self) -> u16 {
        self.rng.gen_range(10..=1000)
    }

    /// Generate a random price within a range
    pub fn random_price(&mut self, min: f64, max: f64) -> f64 {
        self.rng.gen_range(min..=max)
    }

    /// Generate a raw account meta structure
    pub fn raw_account_meta(
        &mut self,
        pubkey: Option<Pubkey>,
        is_writable: bool,
    ) -> RawAccountMeta {
        let pubkey_str = match pubkey {
            Some(pk) => pk.to_string(),
            None => self.random_pubkey().to_string(),
        };

        RawAccountMeta {
            pubkey: pubkey_str,
            is_writable,
            is_signer: self.rng.gen_bool(0.3),
        }
    }

    /// Generate a Jupiter swap quote
    pub fn jupiter_swap_quote(&mut self, input_mint: &str, output_mint: &str) -> JupiterSwapQuote {
        let input_amount = self.random_usdc_amount();
        let output_amount = self.random_sol_amount();
        let price_impact = self.rng.gen_range(0.001..0.05); // 0.1-5% price impact

        JupiterSwapQuote {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            input_amount,
            output_amount,
            price_impact_pct: price_impact * 100.0,
            slippage_bps: self.random_slippage_bps(),
            routes: vec![format!("route_{}", self.rng.gen_range(0..100))],
            fee_amount: input_amount / 1000, // 0.1% fee
            fee_pct: 0.1,
        }
    }

    /// Generate a Jupiter lending position
    pub fn jupiter_lending_position(
        &mut self,
        user_pubkey: &str,
        mint: &str,
    ) -> JupiterLendingPosition {
        let deposit_amount = self.random_usdc_amount();
        let apy = self.rng.gen_range(2.0..15.0);
        let days_elapsed = self.rng.gen_range(30..365);

        JupiterLendingPosition {
            user_pubkey: user_pubkey.to_string(),
            mint: mint.to_string(),
            deposit_amount,
            current_amount: deposit_amount
                + (deposit_amount as f64 * apy / 100.0 * days_elapsed as f64 / 365.0) as u64,
            value_usd: deposit_amount as f64 / 1_000_000.0,
            apy,
            accrued_interest: (deposit_amount as f64 * apy / 100.0 * days_elapsed as f64 / 365.0)
                as u64,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Generate an account state
    pub fn account_state(&mut self) -> AccountState {
        AccountState {
            pubkey: self.random_pubkey().to_string(),
            lamports: self.random_amount(1_000_000, 10_000_000_000), // 0.001-10 SOL
            owner: Some(self.random_pubkey().to_string()),
            data: vec![0u8; self.random_amount(0, 1024) as usize],
            executable: self.rng.gen_bool(0.1), // 10% chance of being executable
        }
    }

    /// Generate a transaction sequence
    pub fn transaction_sequence(&mut self, num_transactions: usize) -> Vec<String> {
        (0..num_transactions)
            .map(|i| format!("transaction_{i}"))
            .collect()
    }

    /// Generate market depth
    pub fn market_depth(&mut self, base_price: f64, spread: f64, levels: usize) -> MarketDepth {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..levels {
            let bid_price = base_price - (i as f64 * spread * 0.1);
            let ask_price = base_price + (i as f64 * spread * 0.1);

            bids.push(MarketOrder {
                price: bid_price,
                amount: self.random_amount(100, 1000),
                is_bid: true,
            });

            asks.push(MarketOrder {
                price: ask_price,
                amount: self.random_amount(100, 1000),
                is_bid: false,
            });
        }

        MarketDepth { bids, asks }
    }

    /// Generate mock Jupiter position summary
    pub fn jupiter_position_summary(
        &mut self,
        has_balance: bool,
    ) -> super::jupiter_types::JupiterPositionSummary {
        let shares = if has_balance {
            self.random_amount(100_000_000, 1_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let underlying_assets = if has_balance {
            self.random_amount(100_000_000, 1_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let underlying_balance_decimal = if has_balance {
            self.random_amount(100_000_000, 1_000_000_000) as f64 / 1_000_000.0
        } else {
            0.0
        };

        super::jupiter_types::JupiterPositionSummary {
            token: super::jupiter_types::JupiterToken {
                symbol: "jlUSDC".to_string(),
                name: "jupiter lend USDC".to_string(),
                address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
                asset_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                decimals: 6,
            },
            asset: super::jupiter_types::JupiterAsset {
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                price: format!("{:.8}", self.random_price(0.99, 1.01)),
                logo_url:
                    "https://coin-images.coingecko.com/coins/images/6319/large/usdc.png?1696506694"
                        .to_string(),
            },
            position: super::jupiter_types::JupiterPositionData {
                shares,
                underlying_assets,
                underlying_balance: "0".to_string(),
                underlying_balance_decimal,
                usd_value: underlying_balance_decimal * self.random_price(0.99, 1.01),
                allowance: "0".to_string(),
            },
            rates: super::jupiter_types::JupiterRates {
                supply_rate_pct: self.random_price(3.0, 8.0),
                total_rate_pct: self.random_price(6.0, 12.0),
                rewards_rate: self.random_amount(300, 500).to_string(),
            },
            liquidity: super::jupiter_types::JupiterLiquidity {
                total_assets: self
                    .random_amount(300_000_000_000_000, 400_000_000_000_000)
                    .to_string(),
                withdrawable: self
                    .random_amount(30_000_000_000_000, 50_000_000_000_000)
                    .to_string(),
                withdrawal_limit: self
                    .random_amount(200_000_000_000_000, 300_000_000_000_000)
                    .to_string(),
            },
        }
    }

    /// Generate mock Jupiter position item
    pub fn jupiter_position_item(
        &mut self,
        user_pubkey: &str,
        has_balance: bool,
    ) -> super::jupiter_types::JupiterPositionItem {
        let deposits_raw = if has_balance {
            self.random_amount(1_000_000_000, 5_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let withdraws_raw = if has_balance {
            self.random_amount(0, 2_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let current_balance_raw = if has_balance {
            self.random_amount(500_000_000, 2_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        super::jupiter_types::JupiterPositionItem {
            position_address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            owner_address: user_pubkey.to_string(),
            earnings: super::jupiter_types::JupiterPositionEarnings {
                raw: self.random_amount(1_000_000, 10_000_000).to_string(),
                decimal: self.random_amount(1_000_000, 10_000_000) as f64,
            },
            deposits: super::jupiter_types::JupiterPositionBalance {
                raw: deposits_raw.clone(),
                decimal: deposits_raw.parse::<f64>().unwrap_or(0.0),
            },
            withdraws: super::jupiter_types::JupiterPositionBalance {
                raw: withdraws_raw.clone(),
                decimal: withdraws_raw.parse::<f64>().unwrap_or(0.0),
            },
            current_balance: super::jupiter_types::JupiterPositionBalance {
                raw: current_balance_raw.clone(),
                decimal: current_balance_raw.parse::<f64>().unwrap_or(0.0),
            },
            total_assets: self.random_amount(900_000_000, 1_000_000_000).to_string(),
            slot: self.random_amount(370_000_000, 380_000_000),
        }
    }

    /// Generate mock raw Jupiter earnings
    pub fn raw_jupiter_earnings(
        &mut self,
        user_pubkey: &str,
        has_balance: bool,
    ) -> super::jupiter_types::RawJupiterEarnings {
        let total_deposits = if has_balance {
            self.random_amount(1_000_000_000, 5_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let total_withdraws = if has_balance {
            self.random_amount(0, 2_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        let total_balance = if has_balance {
            self.random_amount(500_000_000, 2_000_000_000).to_string()
        } else {
            "0".to_string()
        };

        super::jupiter_types::RawJupiterEarnings {
            address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            owner_address: user_pubkey.to_string(),
            total_deposits,
            total_withdraws,
            total_balance,
            total_assets: self.random_amount(900_000_000, 1_000_000_000).to_string(),
            earnings: self.random_amount(1_000_000, 10_000_000).to_string(),
            slot: self.random_amount(370_000_000, 380_000_000),
        }
    }
}
