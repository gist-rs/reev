//! Jupiter protocol configuration

use crate::config::{get_env_string, get_env_var};
use std::time::Duration;

/// Configuration for Jupiter protocol APIs
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    /// Base URL for Jupiter API endpoints
    pub api_base_url: String,
    /// Timeout for API requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// User agent string for API requests
    pub user_agent: String,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://lite-api.jup.ag".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            user_agent: "reev-agent/0.1.0".to_string(),
        }
    }
}

impl JupiterConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            api_base_url: get_env_string("JUPITER_API_BASE_URL", "https://lite-api.jup.ag"),
            timeout_seconds: get_env_var("JUPITER_TIMEOUT_SECONDS", 30),
            max_retries: get_env_var("JUPITER_MAX_RETRIES", 3),
            user_agent: get_env_string("JUPITER_USER_AGENT", "reev-agent/0.1.0"),
        }
    }

    /// Get the timeout as Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// Get the positions API endpoint URL
    pub fn positions_url(&self) -> String {
        format!("{}/lend/v1/earn/positions", self.api_base_url)
    }

    /// Get the earnings API endpoint URL
    pub fn earnings_url(&self) -> String {
        format!("{}/lend/v1/earn/earnings", self.api_base_url)
    }

    /// Get the quote API endpoint URL for swaps
    pub fn quote_url(&self) -> String {
        format!("{}/v6/quote", self.api_base_url)
    }

    /// Get the swap API endpoint URL
    pub fn swap_url(&self) -> String {
        format!("{}/v6/swap", self.api_base_url)
    }

    /// Get the lend deposit API endpoint URL
    pub fn lend_deposit_url(&self) -> String {
        format!("{}/lend/v1/deposit", self.api_base_url)
    }

    /// Get the lend withdraw API endpoint URL
    pub fn lend_withdraw_url(&self) -> String {
        format!("{}/lend/v1/withdraw", self.api_base_url)
    }
}
