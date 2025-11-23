//! Context Resolution for Verifiable AI-Generated DeFi Flows
//!
//! This module provides context resolution for wallet information in both production
//! and benchmark modes, with support for SURFPOOL integration in benchmark mode.

use reev_types::benchmark::TokenBalance;
use reev_types::flow::WalletContext;
// use reev_types::tools::ToolName; // Currently unused

// Define SolanaEnvironment locally as it's not available in reev-types
#[derive(Debug, Clone)]
pub struct SolanaEnvironment {
    pub rpc_url: Option<String>,
}

impl Default for SolanaEnvironment {
    fn default() -> Self {
        Self {
            rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
        }
    }
}
use anyhow::{anyhow, Result};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, instrument};

/// Context resolver for wallet information in different modes
pub struct ContextResolver {
    /// Solana environment configuration
    solana_env: SolanaEnvironment,
    /// Context resolution timeout in seconds
    timeout_seconds: u64,
    /// Cache for resolved contexts
    cache: HashMap<String, CacheEntry>,
    /// SURFPOOL RPC URL for benchmark mode
    surfpool_rpc_url: String,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new(solana_env: SolanaEnvironment) -> Self {
        Self {
            solana_env,
            timeout_seconds: 30,
            cache: HashMap::new(),
            surfpool_rpc_url: std::env::var("SURFPOOL_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        }
    }

    /// Set timeout for context resolution
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Resolve wallet context, handling both production and benchmark modes
    #[instrument(skip(self))]
    pub async fn resolve_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        // Check cache first
        if let Some(cached) = self.cache.get(pubkey) {
            if !cached.is_expired() {
                debug!("Using cached wallet context for {}", pubkey);
                return Ok(cached.context.clone());
            }
        }

        let context = if self.is_benchmark_mode(pubkey) {
            debug!("Resolving wallet context in benchmark mode for {}", pubkey);
            self.resolve_benchmark_wallet_context(pubkey).await?
        } else {
            debug!("Resolving wallet context in production mode for {}", pubkey);
            self.resolve_production_wallet_context(pubkey).await?
        };

        // Update cache
        // Note: We can't modify self here in a non-mutable method.
        // In a real implementation, we would use Arc<Mutex<Cache>> or similar.
        // For now, we'll just return the context without caching.
        info!("Context resolved for {} (caching disabled)", pubkey);

        Ok(context)
    }

    /// Check if we're in benchmark mode (using USER_WALLET_PUBKEY placeholder)
    fn is_benchmark_mode(&self, pubkey: &str) -> bool {
        pubkey == "USER_WALLET_PUBKEY" || std::env::var("BENCHMARK_MODE").is_ok()
    }

    /// Resolve wallet context in production mode
    async fn resolve_production_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        // Create a basic wallet context with available information
        let mut context = WalletContext::new(pubkey.to_string());

        // In a real implementation, this would fetch actual wallet data
        // For now, we'll create a placeholder that should be replaced with actual data
        let client = solana_client::rpc_client::RpcClient::new(
            self.solana_env
                .rpc_url
                .as_ref()
                .ok_or_else(|| anyhow!("RPC URL not configured"))?,
        );

        // Get account info with timeout
        let account_info = client
            .get_account(
                &pubkey
                    .parse()
                    .map_err(|e| anyhow!("Invalid pubkey {pubkey}: {e}"))?,
            )
            .map_err(|e| anyhow!("Error fetching account info for {pubkey}: {e}"))?;

        context.sol_balance = account_info.lamports;

        // For simplicity, we'll skip fetching token balances here
        // In a real implementation, this would fetch all token accounts

        context.calculate_total_value();
        Ok(context)
    }

    /// Resolve wallet context in benchmark mode using SURFPOOL
    async fn resolve_benchmark_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        // For benchmark mode with USER_WALLET_PUBKEY, we need to use SURFPOOL
        if pubkey == "USER_WALLET_PUBKEY" {
            return self.setup_benchmark_wallet().await;
        }

        // Otherwise, use normal resolution
        self.resolve_production_wallet_context(pubkey).await
    }

    /// Setup a benchmark wallet via SURFPOOL
    #[instrument(skip(self))]
    async fn setup_benchmark_wallet(&self) -> Result<WalletContext> {
        info!("Setting up benchmark wallet via SURFPOOL");

        // Create HTTP client for SURFPOOL requests
        let client = reqwest::Client::new();

        // Create request to set up account
        let mut request_body = serde_json::Map::new();
        request_body.insert("jsonrpc".to_string(), json!("2.0"));
        request_body.insert("id".to_string(), json!(1));
        request_body.insert("method".to_string(), json!("surfnet_setAccount"));

        let mut params = serde_json::Map::new();
        params.insert("lamports".to_string(), json!(5_000_000_000i64)); // 5 SOL

        // Add some common tokens for testing
        let mut tokens = Vec::new();

        // USDC
        let mut usdc = serde_json::Map::new();
        usdc.insert(
            "mint".to_string(),
            json!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        );
        usdc.insert("amount".to_string(), json!(200_000_000)); // 200 USDC
        usdc.insert("decimals".to_string(), json!(6));
        tokens.push(json!(usdc));

        // SOL
        let mut sol = serde_json::Map::new();
        sol.insert(
            "mint".to_string(),
            json!("So11111111111111111111111111111111111111111112"),
        );
        sol.insert("amount".to_string(), json!(1_000_000_000)); // 1 SOL
        sol.insert("decimals".to_string(), json!(9));
        tokens.push(json!(sol));

        params.insert("tokens".to_string(), json!(tokens));
        request_body.insert("params".to_string(), json!(vec![params]));

        // Make request to SURFPOOL
        let response = timeout(
            Duration::from_secs(self.timeout_seconds),
            client
                .post(&self.surfpool_rpc_url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send(),
        )
        .await
        .map_err(|_| anyhow!("Timeout setting up benchmark account via SURFPOOL"))?
        .map_err(|e| anyhow!("Error setting up benchmark account via SURFPOOL: {e}"))?;

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Error parsing SURFPOOL response: {e}"))?;

        if let Some(error) = response_json.get("error") {
            return Err(anyhow!("SURFPOOL error: {error}"));
        }

        let result = response_json
            .get("result")
            .ok_or_else(|| anyhow!("Missing result in SURFPOOL response"))?;

        let pubkey = result
            .get("pubkey")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow!("Missing pubkey in SURFPOOL result"))?;

        info!("Created benchmark wallet: {}", pubkey);

        // Now build a context with the created wallet
        let mut context = WalletContext::new(pubkey.to_string());
        context.sol_balance = 5_000_000_000; // 5 SOL

        // Add token balances from the response
        if let Some(tokens) = result.get("tokens").and_then(|t| t.as_array()) {
            for token in tokens {
                if let (Some(mint), Some(amount), Some(decimals)) = (
                    token.get("mint").and_then(|m| m.as_str()),
                    token.get("amount").and_then(|a| a.as_u64()),
                    token.get("decimals").and_then(|d| d.as_u64()),
                ) {
                    let balance =
                        TokenBalance::new(mint.to_string(), amount).with_decimals(decimals as u8);
                    context.add_token_balance(mint.to_string(), balance);
                }
            }
        }

        // Add some default prices for common tokens
        context.add_token_price(
            "So11111111111111111111111111111111111111112".to_string(),
            150.0,
        ); // SOL
        context.add_token_price(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1.0,
        ); // USDC

        context.calculate_total_value();
        Ok(context)
    }

    /// Get placeholder mappings for template variables
    pub async fn get_placeholder_mappings(
        &self,
        context: &WalletContext,
    ) -> HashMap<String, String> {
        let mut mappings = HashMap::new();

        mappings.insert("WALLET_PUBKEY".to_string(), context.owner.clone());
        mappings.insert(
            "SOL_BALANCE".to_string(),
            format!("{:.9}", context.sol_balance_sol()),
        );
        mappings.insert(
            "TOTAL_VALUE_USD".to_string(),
            format!("{:.2}", context.total_value_usd),
        );

        // Add token balances
        for balance in context.token_balances.values() {
            if let Some(symbol) = &balance.symbol {
                let key = format!("{}_BALANCE", symbol.to_uppercase());
                if let (Some(decimals), amount) = (balance.decimals, balance.balance) {
                    let formatted_amount = amount as f64 / 10_f64.powi(decimals as i32);
                    mappings.insert(key, format!("{formatted_amount:.6}"));
                }
            }
        }

        // Add token prices
        for (mint, price) in &context.token_prices {
            if let Some(symbol) = self.get_token_symbol(mint) {
                let key = format!("{}_PRICE", symbol.to_uppercase());
                mappings.insert(key, format!("{price:.6}"));
            }
        }

        mappings
    }

    /// Get token symbol from mint address
    fn get_token_symbol(&self, mint: &str) -> Option<String> {
        match mint {
            "So11111111111111111111111111111111111111112" => Some("SOL".to_string()),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some("USDC".to_string()),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some("USDT".to_string()),
            _ => None,
        }
    }

    /// Clear all cached contexts
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total = self.cache.len();
        let expired = self
            .cache
            .values()
            .filter(|entry| entry.is_expired())
            .count();
        (total, expired)
    }
}

impl Default for ContextResolver {
    fn default() -> Self {
        Self::new(SolanaEnvironment::default())
    }
}

/// Cache entry for resolved contexts
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached context
    context: WalletContext,
    /// When the entry was created
    created_at: chrono::DateTime<chrono::Utc>,
    /// Time-to-live in seconds
    ttl_seconds: u64,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(context: WalletContext, ttl_seconds: u64) -> Self {
        Self {
            context,
            created_at: chrono::Utc::now(),
            ttl_seconds,
        }
    }

    /// Check if the entry is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let elapsed = (now - self.created_at).num_seconds();
        elapsed > self.ttl_seconds as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_placeholder_mappings() {
        let mut context = WalletContext::new("test_pubkey".to_string());
        context.sol_balance = 1_000_000_000; // 1 SOL
        context.add_token_balance(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            TokenBalance::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                1_000_000,
            )
            .with_decimals(6)
            .with_symbol("USDC".to_string()),
        );
        context.add_token_price(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1.0,
        );
        context.calculate_total_value();

        let resolver = ContextResolver::new(SolanaEnvironment::default());
        let mappings = resolver.get_placeholder_mappings(&context).await;

        assert_eq!(
            mappings.get("WALLET_PUBKEY"),
            Some(&"test_pubkey".to_string())
        );
        assert_eq!(
            mappings.get("SOL_BALANCE"),
            Some(&"1.000000000".to_string())
        );
        assert_eq!(mappings.get("USDC_BALANCE"), Some(&"1.000000".to_string()));
        assert_eq!(mappings.get("USDC_PRICE"), Some(&"1.000000".to_string()));
    }

    #[test]
    fn test_benchmark_mode_detection() {
        let resolver = ContextResolver::new(SolanaEnvironment::default());

        assert!(resolver.is_benchmark_mode("USER_WALLET_PUBKEY"));
        assert!(!resolver.is_benchmark_mode("some_other_pubkey"));
    }

    #[test]
    fn test_cache_entry_expiration() {
        let context = WalletContext::new("test".to_string());
        let entry = CacheEntry::new(context, 60);

        assert!(!entry.is_expired());

        let mut expired_entry = entry;
        expired_entry.created_at = chrono::Utc::now() - chrono::Duration::seconds(120);
        assert!(expired_entry.is_expired());
    }
}
