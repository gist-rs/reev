//! Programmatic mock data generator for testing and benchmarks
//!
//! This module provides utilities to generate realistic mock data for testing
//! financial transactions, account states, and other blockchain-related data.

use rand::Rng;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::agent::RawAccountMeta;
use crate::constants::{sol_mint, usdc_mint};

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

    /// Generate a random USDC amount (1-1000 USDC)
    pub fn random_usdc_amount(&mut self) -> u64 {
        let usdc_amount = self.random_amount(1_000_000, 1_000_000_000); // 1-1000 USDC with 6 decimals
                                                                        // Round to nearest multiple of 1000 for cleaner amounts
        (usdc_amount / 1000) * 1000
    }

    /// Generate a random SOL amount (0.001-10 SOL)
    pub fn random_sol_amount(&mut self) -> u64 {
        self.random_amount(1_000_000, 10_000_000_000) // 0.001-10 SOL with 9 decimals
    }

    /// Generate a random slippage percentage (1-20%)
    pub fn random_slippage_bps(&mut self) -> u16 {
        self.rng.gen_range(100..=2000) // 1-20% in basis points
    }

    /// Generate a random price with reasonable decimal places
    pub fn random_price(&mut self, min: f64, max: f64) -> f64 {
        let price = self.rng.gen_range(min..=max);
        (price * 100.0).round() / 100.0 // Round to 2 decimal places
    }

    /// Generate mock raw account metadata
    pub fn raw_account_meta(
        &mut self,
        pubkey: Pubkey,
        is_writable: bool,
        is_signer: bool,
    ) -> RawAccountMeta {
        RawAccountMeta {
            pubkey: pubkey.to_string(),
            is_signer,
            is_writable,
        }
    }

    /// Generate mock Jupiter swap quote data
    pub fn jupiter_swap_quote(
        &mut self,
        input_amount: u64,
        input_mint: Pubkey,
        output_mint: Pubkey,
    ) -> JupiterSwapQuote {
        let output_amount = if input_mint == sol_mint() {
            // SOL to USDC with some price impact
            let usdc_price = self.random_price(50.0, 150.0); // 1 SOL = 50-150 USDC
            (input_amount as f64 * usdc_price / 1_000_000_000.0) as u64
        } else {
            // USDC to SOL
            let sol_price = self.random_price(0.007, 0.015); // 1 USDC = 0.007-0.015 SOL
            (input_amount as f64 * sol_price) as u64
        };

        let price_impact = self.random_price(0.1, 5.0); // 0.1-5% price impact

        JupiterSwapQuote {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            input_amount,
            output_amount,
            price_impact_pct: price_impact,
            slippage_bps: self.random_slippage_bps(),
            routes: vec![],
            fee_amount: self.random_amount(1000, 10000), // Small fee
            fee_pct: price_impact * 0.1,                 // Fee as percentage of price impact
        }
    }

    /// Generate mock Jupiter lending position data
    pub fn jupiter_lending_position(
        &mut self,
        user_pubkey: Pubkey,
        mint: Pubkey,
        deposit_amount: u64,
    ) -> JupiterLendingPosition {
        let current_price = if mint == usdc_mint() {
            1.0 // USDC stable coin
        } else {
            self.random_price(0.95, 1.05) // Slight variation around 1.0
        };

        let apy = self.random_price(0.5, 15.0); // 0.5-15% APY
        let accrued_interest = deposit_amount as f64 * (apy / 100.0) * 0.1; // Approximate for ~36 days

        JupiterLendingPosition {
            user_pubkey: user_pubkey.to_string(),
            mint: mint.to_string(),
            deposit_amount,
            current_amount: deposit_amount,
            value_usd: deposit_amount as f64 * current_price,
            apy,
            accrued_interest: accrued_interest as u64,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Generate mock account state for testing
    pub fn account_state(
        &mut self,
        pubkey: Pubkey,
        lamports: u64,
        owner: Option<Pubkey>,
        data: Option<Vec<u8>>,
    ) -> AccountState {
        AccountState {
            pubkey: pubkey.to_string(),
            lamports,
            owner: owner.map(|p| p.to_string()),
            data: data.unwrap_or_default(),
            executable: false,
        }
    }

    /// Generate a sequence of mock transaction amounts for testing
    pub fn transaction_sequence(
        &mut self,
        count: usize,
        min_amount: u64,
        max_amount: u64,
    ) -> Vec<u64> {
        (0..count)
            .map(|_| self.random_amount(min_amount, max_amount))
            .collect()
    }

    /// Generate mock market depth data
    pub fn market_depth(&mut self, base_price: f64, levels: usize) -> MarketDepth {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..levels {
            let price_offset = (i + 1) as f64 * 0.001; // 0.1% price increments
            let amount = self.random_amount(1000000, 10000000); // 1-10 USDC amounts

            // Bids (below base price)
            bids.push(MarketOrder {
                price: base_price * (1.0 - price_offset),
                amount,
                is_bid: true,
            });

            // Asks (above base price)
            asks.push(MarketOrder {
                price: base_price * (1.0 + price_offset),
                amount,
                is_bid: false,
            });
        }

        MarketDepth { bids, asks }
    }
}

/// Mock Jupiter swap quote data
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

/// Mock Jupiter lending position data
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

/// Mock account state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: Option<String>,
    pub data: Vec<u8>,
    pub executable: bool,
}

/// Mock market order data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    pub price: f64,
    pub amount: u64,
    pub is_bid: bool,
}

/// Mock market depth data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub bids: Vec<MarketOrder>,
    pub asks: Vec<MarketOrder>,
}

/// Mock financial data scenarios
pub struct FinancialScenarios;

impl FinancialScenarios {
    /// Generate a realistic DeFi trading scenario
    pub fn defi_trading_scenario() -> Vec<FinancialTransaction> {
        vec![
            FinancialTransaction {
                id: "swap-1".to_string(),
                transaction_type: "swap".to_string(),
                input_mint: sol_mint().to_string(),
                output_mint: usdc_mint().to_string(),
                input_amount: 100_000_000, // 0.1 SOL
                output_amount: 13_500_000, // ~13.5 USDC
                price_usd: 135.0,
                timestamp: chrono::Utc::now(),
            },
            FinancialTransaction {
                id: "lend-1".to_string(),
                transaction_type: "lend".to_string(),
                input_mint: usdc_mint().to_string(),
                output_mint: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(), // jUSDC
                input_amount: 10_000_000,                                                // 10 USDC
                output_amount: 10_050_000, // ~10.05 jUSDC (with interest)
                price_usd: 10.0,
                timestamp: chrono::Utc::now(),
            },
        ]
    }

    /// Generate a high-frequency trading scenario
    pub fn high_frequenc_trading_scenario() -> Vec<FinancialTransaction> {
        let mut transactions = Vec::new();
        let mut generator = MockGenerator::new();

        for i in 0..100 {
            let amount = generator.random_amount(1_000_000, 100_000_000);
            transactions.push(FinancialTransaction {
                id: format!("arb-{i}"),
                transaction_type: "arbitrage".to_string(),
                input_mint: usdc_mint().to_string(),
                output_mint: sol_mint().to_string(),
                input_amount: amount,
                output_amount: (amount as f64 * 0.0075) as u64, // Rough exchange rate
                price_usd: 1.0,
                timestamp: chrono::Utc::now(),
            });
        }

        transactions
    }
}

/// Mock financial transaction data
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

/// Mock Jupiter position token data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterToken {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub asset_address: String,
    pub decimals: u8,
}

/// Mock Jupiter position asset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterAsset {
    pub symbol: String,
    pub name: String,
    pub price: String,
    pub logo_url: String,
}

/// Mock Jupiter position position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionData {
    pub shares: String,
    pub underlying_assets: String,
    pub underlying_balance: String,
    pub underlying_balance_decimal: f64,
    pub usd_value: f64,
    pub allowance: String,
}

/// Mock Jupiter position rates data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterRates {
    pub supply_rate_pct: f64,
    pub total_rate_pct: f64,
    pub rewards_rate: String,
}

/// Mock Jupiter position liquidity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterLiquidity {
    pub total_assets: String,
    pub withdrawable: String,
    pub withdrawal_limit: String,
}

/// Mock Jupiter position summary item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionSummary {
    pub token: JupiterToken,
    pub asset: JupiterAsset,
    pub position: JupiterPositionData,
    pub rates: JupiterRates,
    pub liquidity: JupiterLiquidity,
}

/// Mock Jupiter position earnings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionEarnings {
    pub raw: String,
    pub decimal: f64,
}

/// Mock Jupiter position balance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterPositionBalance {
    pub raw: String,
    pub decimal: f64,
}

/// Mock Jupiter position item
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

/// Mock raw Jupiter earnings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawJupiterEarnings {
    pub address: String,
    pub ownerAddress: String,
    pub totalDeposits: String,
    pub totalWithdraws: String,
    pub totalBalance: String,
    pub totalAssets: String,
    pub earnings: String,
    pub slot: u64,
}

impl MockGenerator {
    /// Generate mock Jupiter position summary
    pub fn jupiter_position_summary(&mut self, has_balance: bool) -> JupiterPositionSummary {
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

        JupiterPositionSummary {
            token: JupiterToken {
                symbol: "jlUSDC".to_string(),
                name: "jupiter lend USDC".to_string(),
                address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
                asset_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                decimals: 6,
            },
            asset: JupiterAsset {
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                price: format!("{:.8}", self.random_price(0.99, 1.01)),
                logo_url:
                    "https://coin-images.coingecko.com/coins/images/6319/large/usdc.png?1696506694"
                        .to_string(),
            },
            position: JupiterPositionData {
                shares,
                underlying_assets,
                underlying_balance: "0".to_string(),
                underlying_balance_decimal,
                usd_value: underlying_balance_decimal * self.random_price(0.99, 1.01),
                allowance: "0".to_string(),
            },
            rates: JupiterRates {
                supply_rate_pct: self.random_price(3.0, 8.0),
                total_rate_pct: self.random_price(6.0, 12.0),
                rewards_rate: self.random_amount(300, 500).to_string(),
            },
            liquidity: JupiterLiquidity {
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
    ) -> JupiterPositionItem {
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

        JupiterPositionItem {
            position_address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            owner_address: user_pubkey.to_string(),
            earnings: JupiterPositionEarnings {
                raw: self.random_amount(1_000_000, 10_000_000).to_string(),
                decimal: self.random_amount(1_000_000, 10_000_000) as f64,
            },
            deposits: JupiterPositionBalance {
                raw: deposits_raw.clone(),
                decimal: deposits_raw.parse::<f64>().unwrap_or(0.0),
            },
            withdraws: JupiterPositionBalance {
                raw: withdraws_raw.clone(),
                decimal: withdraws_raw.parse::<f64>().unwrap_or(0.0),
            },
            current_balance: JupiterPositionBalance {
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
    ) -> RawJupiterEarnings {
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

        RawJupiterEarnings {
            address: "9BEcn9aPEmhSPbPQeFGjidRiEKki46fVQDyPpSQXPA2D".to_string(),
            ownerAddress: user_pubkey.to_string(),
            totalDeposits: total_deposits,
            totalWithdraws: total_withdraws,
            totalBalance: total_balance,
            totalAssets: self.random_amount(900_000_000, 1_000_000_000).to_string(),
            earnings: self.random_amount(1_000_000, 10_000_000).to_string(),
            slot: self.random_amount(370_000_000, 380_000_000),
        }
    }
}
