//! Context Resolution for Verifiable AI-Generated DeFi Flows
//!
//! This module provides context resolution for wallet information in both production
//! and benchmark modes, with support for SURFPOOL integration to fetch real wallet data.

use reev_types::flow::WalletContext;

// Import jup-sdk for token price fetching
use jup_sdk::api::tokens::search_tokens;
use jup_sdk::models::{TokenInfo, TokenSearchParams};

// Define SolanaEnvironment locally as it's not available in reev-types
#[derive(Debug, Clone)]
pub struct SolanaEnvironment {
    pub rpc_url: Option<String>,
}

impl Default for SolanaEnvironment {
    fn default() -> Self {
        // Default to mainnet until SURFPOOL context resolution issues are resolved
        let rpc_url = std::env::var("SURFPOOL_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        Self {
            rpc_url: Some(rpc_url),
        }
    }
}
use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, info, instrument};

/// Context resolver for wallet information in different modes
#[derive(Clone)]
pub struct ContextResolver {
    /// Context resolution timeout in seconds
    timeout_seconds: u64,
    /// Cache for resolved contexts
    cache: HashMap<String, CacheEntry>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new(_solana_env: SolanaEnvironment) -> Self {
        Self {
            timeout_seconds: 30,
            cache: HashMap::new(),
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
    pub fn is_benchmark_mode(&self, pubkey: &str) -> bool {
        pubkey == "USER_WALLET_PUBKEY" || std::env::var("BENCHMARK_MODE").is_ok()
    }

    /// Resolve wallet context in production mode
    async fn resolve_production_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        // Create a basic wallet context with available information
        let mut context = WalletContext::new(pubkey.to_string());

        // Try to connect to SURFPOOL to get real wallet data
        let surfpool_url = std::env::var("SURFPOOL_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:8899".to_string());

        // Create an RPC client to fetch data from SURFPOOL
        let rpc_client = solana_client::rpc_client::RpcClient::new(&surfpool_url);

        // Get the SOL balance
        let pubkey_obj = solana_sdk::pubkey::Pubkey::from_str(pubkey)
            .map_err(|e| anyhow::anyhow!("Invalid pubkey: {e}"))?;

        // Fetch SOL balance from SURFPOOL
        let account = rpc_client
            .get_account(&pubkey_obj)
            .map_err(|e| anyhow::anyhow!("Failed to fetch SOL balance from SURFPOOL: {e}"))?;

        context.sol_balance = account.lamports;
        info!(
            "✅ Fetched SOL balance from SURFPOOL: {} lamports",
            account.lamports
        );

        let mut total_value_usd = 0.0;

        // Fetch token information including prices from Jupiter API
        // Create a comma-separated list of mint addresses to query
        let token_mints = [
            "So11111111111111111111111111111111111111112",  // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
        ]
        .join(",");

        // Create the search parameters
        let params = TokenSearchParams { query: token_mints };

        // Fetch token information from Jupiter API
        let tokens = search_tokens(&params).await.map_err(|e| {
            anyhow::anyhow!("Failed to fetch token information from Jupiter API: {e}")
        })?;

        info!("✅ Fetched {} tokens from Jupiter API", tokens.len());

        // Create a map of mint -> token info for easier lookup
        let mut token_info_map = std::collections::HashMap::<String, &TokenInfo>::new();
        for token in &tokens {
            token_info_map.insert(token.id.clone(), token);
        }

        // Calculate SOL value
        if let Some(sol_token) = token_info_map.get("So11111111111111111111111111111111111111112") {
            if let Some(sol_price) = sol_token.usd_price {
                let sol_value = (context.sol_balance as f64 / 10_f64.powi(9)) * sol_price;
                total_value_usd += sol_value;
                info!(
                    "✅ SOL value: ${:.2} ({} SOL @ ${:.2}/SOL)",
                    sol_value,
                    context.sol_balance / 1_000_000_000,
                    sol_price
                );
            } else {
                return Err(anyhow::anyhow!("SOL price not available from Jupiter API"));
            }
        } else {
            return Err(anyhow::anyhow!(
                "SOL token information not found from Jupiter API"
            ));
        }

        // Fetch token balances and calculate their values
        for token in &tokens {
            // Skip SOL which we've already processed
            if token.id == "So11111111111111111111111111111111111111112" {
                continue;
            }

            let mint_pubkey = solana_sdk::pubkey::Pubkey::from_str(&token.id)
                .map_err(|e| anyhow::anyhow!("Invalid mint address {}: {}", token.id, e))?;

            // Get the associated token account address
            let ata = spl_associated_token_account::get_associated_token_address(
                &pubkey_obj,
                &mint_pubkey,
            );

            match rpc_client.get_token_account_balance(&ata) {
                Ok(balance) => {
                    let amount = balance.amount.parse::<u64>().unwrap_or(0);
                    if amount > 0 {
                        context.add_token_balance(
                            token.id.clone(),
                            reev_types::benchmark::TokenBalance::new(token.id.clone(), amount)
                                .with_decimals(token.decimals)
                                .with_symbol(token.symbol.clone()),
                        );

                        // Add token price to context if available
                        if let Some(price) = token.usd_price {
                            context.add_token_price(token.id.clone(), price);

                            // Add to total USD value
                            let token_value =
                                (amount as f64 / 10_f64.powi(token.decimals as i32)) * price;
                            total_value_usd += token_value;

                            info!(
                                "✅ {} value: ${:.2} ({} {} @ ${:.6})",
                                token.symbol,
                                token_value,
                                amount as f64 / 10_f64.powi(token.decimals as i32),
                                token.symbol,
                                price
                            );
                        } else {
                            return Err(anyhow::anyhow!(
                                "Price not available for token: {}",
                                token.symbol
                            ));
                        }
                    }
                }
                Err(e) => {
                    debug!("⚠️ Failed to fetch {} balance: {}", token.symbol, e);
                    // Not all accounts will have all tokens, so this is expected
                }
            }
        }

        context.total_value_usd = total_value_usd;

        info!(
            "✅ Resolved wallet context with total value: ${:.2}",
            context.total_value_usd
        );
        Ok(context)
    }

    /// Resolve wallet context in benchmark mode using simplified implementation
    async fn resolve_benchmark_wallet_context(&self, _pubkey: &str) -> Result<WalletContext> {
        // For benchmark mode, we'll use a simplified implementation for tests
        let mut context = WalletContext::new("USER_WALLET_PUBKEY".to_string());
        context.sol_balance = 5_000_000_000; // 5 SOL
        context.total_value_usd = 750.0; // $750 total value

        // Add some common tokens for testing
        context.add_token_balance(
            "So11111111111111111111111111111111111111111112".to_string(),
            reev_types::benchmark::TokenBalance::new(
                "So11111111111111111111111111111111111111111112".to_string(),
                5_000_000_000, // 5 SOL
            )
            .with_decimals(9)
            .with_symbol("SOL".to_string()),
        );

        context.add_token_balance(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            reev_types::benchmark::TokenBalance::new(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                200_000_000, // 200 USDC
            )
            .with_decimals(6)
            .with_symbol("USDC".to_string()),
        );

        context.add_token_price(
            "So11111111111111111111111111111111111111111112".to_string(),
            150.0, // $150 SOL
        );

        context.add_token_price(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1.0, // $1 USDC
        );

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

    /// Get token mint address from symbol
    pub fn get_token_mint(&self, symbol: &str) -> Option<String> {
        match symbol.to_uppercase().as_str() {
            "SOL" => Some("So11111111111111111111111111111111111111112".to_string()),
            "USDC" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            "USDT" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
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
pub struct CacheEntry {
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
