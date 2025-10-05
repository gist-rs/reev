//! Native Solana operations configuration

use crate::config::{get_env_string, get_env_var};
use std::time::Duration;

/// Configuration for native Solana operations
#[derive(Debug, Clone)]
pub struct NativeConfig {
    /// Solana RPC endpoint URL
    pub rpc_url: String,
    /// WebSocket endpoint URL for subscriptions
    pub ws_url: String,
    /// Timeout for RPC requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Confirmations required for transaction finality
    pub confirmations: u64,
    /// Compute units per transaction
    pub compute_units: u32,
    /// Priority fee in lamports
    pub priority_fee_lamports: u64,
    /// User agent string for RPC requests
    pub user_agent: String,
}

impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            confirmations: 1,
            compute_units: 200_000,
            priority_fee_lamports: 10_000,
            user_agent: "reev-agent/0.1.0".to_string(),
        }
    }
}

impl NativeConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            rpc_url: get_env_string("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com"),
            ws_url: get_env_string("SOLANA_WS_URL", "wss://api.mainnet-beta.solana.com"),
            timeout_seconds: get_env_var("SOLANA_TIMEOUT_SECONDS", 30),
            max_retries: get_env_var("SOLANA_MAX_RETRIES", 3),
            confirmations: get_env_var("SOLANA_CONFIRMATIONS", 1),
            compute_units: get_env_var("SOLANA_COMPUTE_UNITS", 200_000),
            priority_fee_lamports: get_env_var("SOLANA_PRIORITY_FEE_LAMPORTS", 10_000),
            user_agent: get_env_string("SOLANA_USER_AGENT", "reev-agent/0.1.0"),
        }
    }

    /// Get the timeout as Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// Get RPC client configuration
    pub fn rpc_config(&self) -> solana_client_config::Config {
        solana_client_config::Config {
            rpc_url: self.rpc_url.clone(),
            ws_url: Some(self.ws_url.clone()),
            commitment: solana_sdk::commitment_config::CommitmentConfig::confirmed(),
            confirm_transaction_initial_timeout: Some(self.timeout_duration()),
        }
    }

    /// Get transaction options for native operations
    pub fn transaction_options(&self) -> solana_sdk::transaction::TransactionOptions {
        solana_sdk::transaction::TransactionOptions {
            max_retries: Some(self.max_retries),
            preflight_commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
        }
    }
}
