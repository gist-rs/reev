//! Jupiter protocol implementation using common traits
//!
//! This module provides a concrete implementation of the Jupiter protocol
//! using the common protocol abstractions defined in the common module.

use reev_protocols::common::{
    HealthStatus, LendProtocol, ProtocolError, ProtocolMetrics, SwapProtocol, TransferProtocol,
};
// use async_trait::async_trait; // Uncomment when async traits are used
use reev_lib::agent::RawInstruction;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::{
    earnings::get_earnings, get_jupiter_config, lend_deposit::handle_jupiter_lend_deposit,
    lend_withdraw::handle_jupiter_lend_withdraw, positions::get_positions,
    swap::handle_jupiter_swap, JupiterConfig,
};

/// Jupiter protocol implementation
#[derive(Debug)]
pub struct JupiterProtocol {
    config: JupiterConfig,
    metrics: Arc<Mutex<ProtocolMetrics>>,
    last_health_check: Arc<Mutex<Option<Instant>>>,
}

impl JupiterProtocol {
    pub fn new(config: JupiterConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(ProtocolMetrics::new())),
            last_health_check: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_env() -> Self {
        Self::new(get_jupiter_config().clone())
    }

    /// Record metrics for an operation
    fn record_metrics<F, T>(&self, operation: &str, f: F) -> Result<T, ProtocolError>
    where
        F: FnOnce() -> Result<T, ProtocolError>,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        match &result {
            Ok(_) => {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.record_success(operation, duration, 0, 0);
                debug!(
                    protocol = "jupiter",
                    operation = operation,
                    duration_ms = duration.as_millis(),
                    "Operation completed successfully"
                );
            }
            Err(e) => {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.record_failure(operation, &e.to_string());
                error!(
                    protocol = "jupiter",
                    operation = operation,
                    duration_ms = duration.as_millis(),
                    error = %e,
                    "Operation failed"
                );
            }
        }

        result
    }
}

// #[async_trait]
impl Protocol for JupiterProtocol {
    fn name(&self) -> &'static str {
        "jupiter"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    // async fn health_check(&self) -> Result<HealthStatus, ProtocolError> {
    fn health_check(&self) -> Result<HealthStatus, ProtocolError> {
        let now = Instant::now();

        // Check if we should run a health check (not too frequently)
        {
            let mut last_check = self.last_health_check.lock().unwrap();
            if let Some(last) = *last_check {
                if now.duration_since(last) < std::time::Duration::from_secs(30) {
                    // Return cached status
                    let metrics = self.metrics.lock().unwrap();
                    return Ok(metrics.health_status.clone());
                }
            }
            *last_check = Some(now);
        }

        // Perform actual health check
        self.record_metrics("health_check", || {
            // Check if we can reach Jupiter API
            let client = reqwest::Client::new();
            let health_url = format!("{}/health", self.config.api_base_url);

            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                match client
                    .get(&health_url)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(response) if response.status().is_success() => {
                        info!(protocol = "jupiter", "Health check successful");
                        let mut metrics = self.metrics.lock().unwrap();
                        metrics.update_health(HealthStatus::Healthy);
                        Ok(HealthStatus::Healthy)
                    }
                    Ok(response) => {
                        warn!(
                            protocol = "jupiter",
                            status = response.status().as_u16(),
                            "Health check returned non-success status"
                        );
                        let status = HealthStatus::Degraded {
                            message: format!("HTTP {}", response.status()),
                        };
                        let mut metrics = self.metrics.lock().unwrap();
                        metrics.update_health(status.clone());
                        Ok(status)
                    }
                    Err(e) => {
                        error!(
                            protocol = "jupiter",
                            error = %e,
                            "Health check failed"
                        );
                        let status = HealthStatus::Unhealthy {
                            message: format!("Connection failed: {}", e),
                        };
                        let mut metrics = self.metrics.lock().unwrap();
                        metrics.update_health(status.clone());
                        Ok(status)
                    }
                }
            })
        })
    }

    fn metrics(&self) -> &ProtocolMetrics {
        // Note: This returns a reference to the metrics, but we need to be careful
        // about the Mutex lock. In practice, we might want to clone the metrics.
        self.metrics.lock().unwrap()
    }

    fn reset_metrics(&mut self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.reset();

        let mut last_check = self.last_health_check.lock().unwrap();
        *last_check = None;

        info!(protocol = "jupiter", "Metrics reset");
    }

    fn validate_config(&self) -> Result<(), ProtocolError> {
        self.config
            .validate()
            .map_err(|e| ProtocolError::Config(e.to_string()))
    }

    fn supported_operations(&self) -> Vec<reev_protocols::common::ProtocolOperation> {
        use reev_protocols::common::ProtocolOperation;
        vec![
            ProtocolOperation::Swap,
            ProtocolOperation::Deposit,
            ProtocolOperation::Withdraw,
            ProtocolOperation::GetPositions,
            ProtocolOperation::GetEarnings,
        ]
    }
}

// #[async_trait]
impl SwapProtocol for JupiterProtocol {
    // async fn swap(
    fn swap(
        &self,
        user_pubkey: &str,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Vec<RawInstruction>, ProtocolError> {
        self.record_metrics("swap", || {
            // Validate inputs
            let user_pubkey_parsed =
                Pubkey::from_str(user_pubkey).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;
            let input_mint_parsed =
                Pubkey::from_str(input_mint).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;
            let output_mint_parsed =
                Pubkey::from_str(output_mint).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;

            // Validate slippage
            self.config
                .validate_slippage(slippage_bps)
                .map_err(|e| ProtocolError::Validation(e.to_string()))?;

            if amount == 0 {
                return Err(ProtocolError::Validation(
                    "Amount must be greater than 0".to_string(),
                ));
            }

            if input_mint == output_mint {
                return Err(ProtocolError::Validation(
                    "Input and output mints cannot be the same".to_string(),
                ));
            }

            info!(
                protocol = "jupiter",
                operation = "swap",
                user = user_pubkey,
                input = input_mint,
                output = output_mint,
                amount = amount,
                slippage_bps = slippage_bps,
                "Executing swap"
            );

            // Execute swap
            let rt = tokio::runtime::Handle::current();
            let instructions = rt
                .block_on(async {
                    handle_jupiter_swap(
                        user_pubkey_parsed,
                        input_mint_parsed,
                        output_mint_parsed,
                        amount,
                        slippage_bps,
                    )
                })
                .map_err(|e| ProtocolError::ProtocolSpecific {
                    protocol: "jupiter".to_string(),
                    message: e.to_string(),
                })?;

            Ok(instructions)
        })
    }

    // async fn get_quote(
    fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<reev_protocols::common::SwapQuote, ProtocolError> {
        self.record_metrics("get_quote", || {
            // For now, return a placeholder quote
            // In a real implementation, this would call Jupiter's quote API
            let quote = reev_protocols::common::SwapQuote {
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                input_amount: amount,
                output_amount: amount * 95 / 100, // Assume 5% fee/slippage
                price_impact_pct: 0.5,
                slippage_bps: self.config.default_slippage_bps,
                routes: vec![reev_protocols::common::SwapRoute {
                    protocol: "jupiter".to_string(),
                    percentage: 100.0,
                }],
                fee_amount: amount * 50 / 10000, // 0.5% fee
                fee_pct: 0.5,
                valid_until: chrono::Utc::now() + chrono::Duration::minutes(5),
            };

            Ok(quote)
        })
    }

    // async fn supported_pairs(
    fn supported_pairs(&self) -> Result<Vec<reev_protocols::common::TokenPair>, ProtocolError> {
        self.record_metrics("supported_pairs", || {
            // For now, return some common pairs
            // In a real implementation, this would query Jupiter's API
            let pairs = vec![
                reev_protocols::common::TokenPair {
                    input_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                    output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                    liquidity_usd: 1000000.0,
                    volume_24h: 500000.0,
                },
                reev_protocols::common::TokenPair {
                    input_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                    output_mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
                    liquidity_usd: 800000.0,
                    volume_24h: 400000.0,
                },
            ];

            Ok(pairs)
        })
    }

    // async fn minimum_amount(
    fn minimum_amount(&self, _input_mint: &str, _output_mint: &str) -> Result<u64, ProtocolError> {
        self.record_metrics("minimum_amount", || {
            // Return a minimum of 0.001 SOL equivalent
            Ok(1000000) // 0.001 SOL in lamports
        })
    }
}

// #[async_trait]
impl LendProtocol for JupiterProtocol {
    // async fn deposit(
    fn deposit(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError> {
        self.record_metrics("deposit", || {
            // Validate inputs
            let user_pubkey_parsed =
                Pubkey::from_str(user_pubkey).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;
            let token_mint_parsed =
                Pubkey::from_str(token_mint).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;

            if amount == 0 {
                return Err(ProtocolError::Validation(
                    "Amount must be greater than 0".to_string(),
                ));
            }

            info!(
                protocol = "jupiter",
                operation = "deposit",
                user = user_pubkey,
                token = token_mint,
                amount = amount,
                "Executing deposit"
            );

            // Execute deposit
            let rt = tokio::runtime::Handle::current();
            let instructions = rt
                .block_on(async {
                    handle_jupiter_lend_deposit(user_pubkey_parsed, token_mint_parsed, amount)
                })
                .map_err(|e| ProtocolError::ProtocolSpecific {
                    protocol: "jupiter".to_string(),
                    message: e.to_string(),
                })?;

            Ok(instructions)
        })
    }

    // async fn withdraw(
    fn withdraw(
        &self,
        user_pubkey: &str,
        token_mint: &str,
        amount: u64,
    ) -> Result<Vec<RawInstruction>, ProtocolError> {
        self.record_metrics("withdraw", || {
            // Validate inputs
            let user_pubkey_parsed =
                Pubkey::from_str(user_pubkey).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;
            let token_mint_parsed =
                Pubkey::from_str(token_mint).map_err(|e| ProtocolError::InvalidAddress {
                    address: e.to_string(),
                })?;

            if amount == 0 {
                return Err(ProtocolError::Validation(
                    "Amount must be greater than 0".to_string(),
                ));
            }

            info!(
                protocol = "jupiter",
                operation = "withdraw",
                user = user_pubkey,
                token = token_mint,
                amount = amount,
                "Executing withdraw"
            );

            // Execute withdraw
            let rt = tokio::runtime::Handle::current();
            let instructions = rt
                .block_on(async {
                    handle_jupiter_lend_withdraw(user_pubkey_parsed, token_mint_parsed, amount)
                })
                .map_err(|e| ProtocolError::ProtocolSpecific {
                    protocol: "jupiter".to_string(),
                    message: e.to_string(),
                })?;

            Ok(instructions)
        })
    }

    // async fn get_positions(
    //     &self,
    //     user_pubkey: &str,
    // ) -> Result<Vec<reev_protocols::common::LendingPosition>, ProtocolError> {
    fn get_positions(
        &self,
        user_pubkey: &str,
    ) -> Result<Vec<reev_protocols::common::LendingPosition>, ProtocolError> {
        self.record_metrics("get_positions", || {
            let rt = tokio::runtime::Handle::current();
            let positions = rt
                .block_on(async { get_positions(user_pubkey.to_string()).await })
                .map_err(|e| ProtocolError::ProtocolSpecific {
                    protocol: "jupiter".to_string(),
                    message: e.to_string(),
                })?;

            // Convert to common format
            let common_positions = positions
                .into_iter()
                .map(|pos| reev_protocols::common::LendingPosition {
                    position_id: pos
                        .get("position_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    token_mint: pos
                        .get("mint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    deposited_amount: pos
                        .get("deposited_amount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    current_value: pos
                        .get("current_value")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    apy: pos.get("apy").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    protocol: "jupiter".to_string(),
                    created_at: chrono::Utc::now(),
                    last_updated: chrono::Utc::now(),
                })
                .collect();

            Ok(common_positions)
        })
    }

    // async fn get_earnings(
    fn get_earnings(
        &self,
        user_pubkey: &str,
        position_id: Option<String>,
    ) -> Result<Vec<reev_protocols::common::EarningInfo>, ProtocolError> {
        self.record_metrics("get_earnings", || {
            let rt = tokio::runtime::Handle::current();
            let earnings = rt
                .block_on(async { get_earnings(user_pubkey.to_string(), position_id).await })
                .map_err(|e| ProtocolError::ProtocolSpecific {
                    protocol: "jupiter".to_string(),
                    message: e.to_string(),
                })?;

            // Convert to common format
            let common_earnings = earnings
                .into_iter()
                .map(|earn| reev_protocols::common::EarningInfo {
                    position_id: earn
                        .get("position_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    token_mint: earn
                        .get("mint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    earned_amount: earn
                        .get("earned_amount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    usd_value: earn
                        .get("usd_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    earned_at: chrono::Utc::now(),
                    apy: earn.get("apy").and_then(|v| v.as_f64()).unwrap_or(0.0),
                })
                .collect();

            Ok(common_earnings)
        })
    }

    // async fn available_markets(
    fn available_markets(
        &self,
    ) -> Result<Vec<reev_protocols::common::LendingMarket>, ProtocolError> {
        self.record_metrics("available_markets", || {
            // For now, return some common markets
            // In a real implementation, this would query Jupiter's API
            let markets = vec![
                reev_protocols::common::LendingMarket {
                    token_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                    token_symbol: "SOL".to_string(),
                    total_supply: 1000000000,
                    total_borrowed: 200000000,
                    supply_apy: 5.5,
                    borrow_apy: 7.2,
                    utilization_rate: 0.2,
                    protocol: "jupiter".to_string(),
                },
                reev_protocols::common::LendingMarket {
                    token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                    token_symbol: "USDC".to_string(),
                    total_supply: 50000000000,
                    total_borrowed: 10000000000,
                    supply_apy: 4.8,
                    borrow_apy: 6.5,
                    utilization_rate: 0.2,
                    protocol: "jupiter".to_string(),
                },
            ];

            Ok(markets)
        })
    }

    // async fn get_apy(&self, token_mint: &str) -> Result<f64, ProtocolError> {
    fn get_apy(&self, token_mint: &str) -> Result<f64, ProtocolError> {
        self.record_metrics("get_apy", || {
            // Return mock APY based on token
            match token_mint {
                "So11111111111111111111111111111111111111112" => Ok(5.5), // SOL
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Ok(4.8), // USDC
                _ => Ok(4.0),                                             // Default
            }
        })
    }
}

// Jupiter doesn't implement TransferProtocol as it focuses on swaps and lending
// Native SOL/SPL transfers are handled by the Native protocol
