//! Jupiter protocol implementation
//!
//! This module provides real API integration with Jupiter's various services
//! including swaps, lending, and earning operations.

// Inline config helpers
fn get_env_string(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_var<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub mod earnings;
pub mod lend_deposit;
pub mod lend_mint;
pub mod lend_withdraw;
pub mod positions;
pub mod swap;

// Re-export all Jupiter functions
pub use earnings::*;
pub use lend_deposit::*;
pub use lend_mint::*;
pub use lend_withdraw::*;
pub use positions::*;
pub use swap::*;

/// Jupiter protocol configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    /// Base URL for Jupiter API
    pub api_base_url: String,
    /// Timeout for API requests
    pub timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
    /// User agent string
    pub user_agent: String,
    /// Default slippage tolerance in basis points
    pub default_slippage_bps: u16,
    /// Maximum slippage tolerance allowed
    pub max_slippage_bps: u16,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Custom RPC endpoint for surfpool (if different from default)
    pub surfpool_rpc_url: Option<String>,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://lite-api.jup.ag".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            user_agent: "reev-agent/0.1.0".to_string(),
            default_slippage_bps: 50, // 0.5%
            max_slippage_bps: 1000,   // 10%
            debug_logging: false,
            surfpool_rpc_url: None,
        }
    }
}

impl JupiterConfig {
    /// Load configuration from environment variables with dotenvy support
    pub fn from_env() -> Self {
        // Load .env file if it exists
        let _ = dotenvy::dotenv();

        Self {
            api_base_url: get_env_string("JUPITER_API_BASE_URL", "https://lite-api.jup.ag"),
            timeout: Duration::from_secs(get_env_var("JUPITER_TIMEOUT_SECONDS", 30)),
            max_retries: get_env_var("JUPITER_MAX_RETRIES", 3),
            user_agent: get_env_string("JUPITER_USER_AGENT", "reev-agent/0.1.0"),
            default_slippage_bps: get_env_var("JUPITER_DEFAULT_SLIPPAGE_BPS", 50),
            max_slippage_bps: get_env_var("JUPITER_MAX_SLIPPAGE_BPS", 1000),
            debug_logging: get_env_var("JUPITER_DEBUG", false),
            surfpool_rpc_url: std::env::var("JUPITER_SURFPOOL_RPC_URL").ok(),
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.default_slippage_bps > self.max_slippage_bps {
            return Err(anyhow::anyhow!(
                "Default slippage ({}) cannot exceed maximum slippage ({})",
                self.default_slippage_bps,
                self.max_slippage_bps
            ));
        }

        if self.max_slippage_bps > 10000 {
            return Err(anyhow::anyhow!(
                "Maximum slippage cannot exceed 100% (10000 bps), got {}",
                self.max_slippage_bps
            ));
        }

        if self.timeout.as_secs() == 0 {
            return Err(anyhow::anyhow!("Timeout must be greater than 0 seconds"));
        }

        if self.max_retries == 0 {
            return Err(anyhow::anyhow!("Max retries must be greater than 0"));
        }

        Ok(())
    }

    /// Get default slippage for swaps
    pub fn default_slippage(&self) -> u16 {
        self.default_slippage_bps
    }

    /// Check if slippage is within allowed range
    pub fn validate_slippage(&self, slippage_bps: u16) -> Result<u16> {
        if slippage_bps > self.max_slippage_bps {
            return Err(anyhow::anyhow!(
                "Slippage {} exceeds maximum allowed {}",
                slippage_bps,
                self.max_slippage_bps
            ));
        }
        Ok(slippage_bps)
    }

    /// Log configuration if debug mode is enabled
    pub fn log_config(&self) {
        if self.debug_logging {
            tracing::debug!("Jupiter Configuration:");
            tracing::debug!("  API Base URL: {}", self.api_base_url);
            tracing::debug!("  Timeout: {}s", self.timeout.as_secs());
            tracing::debug!("  Max Retries: {}", self.max_retries);
            tracing::debug!("  Default Slippage: {} bps", self.default_slippage_bps);
            tracing::debug!("  Max Slippage: {} bps", self.max_slippage_bps);
            tracing::debug!("  Debug Logging: {}", self.debug_logging);
            if let Some(ref rpc_url) = self.surfpool_rpc_url {
                tracing::debug!("  Custom RPC URL: {}", rpc_url);
            }
        }
    }

    /// Create HTTP client with this configuration
    pub fn create_client(&self) -> Result<Client> {
        Client::builder()
            .user_agent(&self.user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(Into::into)
    }

    /// Get positions API endpoint URL
    pub fn positions_url(&self) -> String {
        format!("{}/lend/v1/earn/positions", self.api_base_url)
    }

    /// Get earnings API endpoint URL
    pub fn earnings_url(&self) -> String {
        format!("{}/lend/v1/earn/earnings", self.api_base_url)
    }

    /// Get quote API endpoint URL
    pub fn quote_url(&self) -> String {
        format!("{}/swap/v1/quote", self.api_base_url)
    }

    /// Get swap API endpoint URL
    pub fn swap_url(&self) -> String {
        format!("{}/swap/v1/swap-instructions", self.api_base_url)
    }

    /// Get lend deposit API endpoint URL
    pub fn lend_deposit_url(&self) -> String {
        format!("{}/lend/v1/earn/deposit-instructions", self.api_base_url)
    }

    /// Get lend withdraw API endpoint URL
    pub fn lend_withdraw_url(&self) -> String {
        format!("{}/lend/v1/earn/withdraw-instructions", self.api_base_url)
    }
}

use std::sync::OnceLock;

/// Global Jupiter configuration
static JUPITER_CONFIG: OnceLock<JupiterConfig> = OnceLock::new();

/// Initialize global Jupiter configuration
pub fn init_jupiter_config(config: JupiterConfig) {
    JUPITER_CONFIG
        .set(config)
        .expect("Jupiter config already initialized");
}

/// Get global Jupiter configuration
pub fn get_jupiter_config() -> &'static JupiterConfig {
    JUPITER_CONFIG.get_or_init(JupiterConfig::from_env)
}

/// Execute HTTP request with retry logic
pub async fn execute_request(
    request: reqwest::RequestBuilder,
    max_retries: u32,
) -> Result<reqwest::Response> {
    let mut retries = 0;
    let mut last_error = None;

    while retries <= max_retries {
        match request.try_clone().unwrap().send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response);
                }

                // Don't retry on client errors (4xx)
                if response.status().is_client_error() {
                    return Err(anyhow::anyhow!(
                        "Client error: {} - {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    ));
                }

                last_error = Some(anyhow::anyhow!(
                    "Server error: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or_default()
                ));
            }
            Err(e) => {
                last_error = Some(anyhow::anyhow!("HTTP error: {e}"));
            }
        }

        retries += 1;
        if retries <= max_retries {
            tokio::time::sleep(Duration::from_millis(1000 * retries as u64)).await;
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
}

/// Parse JSON response with error handling
pub async fn parse_json_response(response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status {status}: {body}"
        ));
    }

    serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {e} - Body: {body}"))
}
