//! Context Resolver for Dynamic Flows
//!
//! This module handles resolving wallet context including balance, token prices,
//! and other metadata needed for dynamic prompt generation.

use anyhow::Result;
use lru::LruCache;
use reev_lib::solana_env::environment::SolanaEnv;
use reev_types::flow::WalletContext;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, trace};
/// Cache TTL configuration
const WALLET_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const PRICE_CACHE_TTL: Duration = Duration::from_secs(30); // 30 seconds

/// Cached entry with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            created_at: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Context resolver for wallet and token information
pub struct ContextResolver {
    /// Solana environment for placeholder resolution
    solana_env: Option<Arc<tokio::sync::Mutex<SolanaEnv>>>,
    /// Cache for wallet context data
    wallet_cache: Mutex<LruCache<String, CacheEntry<WalletContext>>>,
    /// Cache for token price data
    price_cache: Mutex<LruCache<String, CacheEntry<f64>>>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new() -> Self {
        Self {
            solana_env: None,
            wallet_cache: Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())),
            price_cache: Mutex::new(LruCache::new(NonZeroUsize::new(50).unwrap())),
        }
    }

    /// Create a new context resolver with Solana environment
    pub fn with_solana_env(solana_env: Arc<tokio::sync::Mutex<SolanaEnv>>) -> Self {
        Self {
            solana_env: Some(solana_env),
            wallet_cache: Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap())),
            price_cache: Mutex::new(LruCache::new(NonZeroUsize::new(50).unwrap())),
        }
    }

    /// Check if a string is a placeholder and resolve it to a real pubkey using SolanaEnv
    pub async fn resolve_placeholder(&self, input: &str) -> Result<String> {
        // Check if this looks like a placeholder (all caps with underscores)
        if input.chars().all(|c| c.is_uppercase() || c == '_') && input.contains('_') {
            // Use SolanaEnv if available (same system as static benchmarks)
            if let Some(ref solana_env) = self.solana_env {
                let mut env = solana_env.lock().await;

                // Return existing mapping if available
                if let Some(pubkey) = env.pubkey_map.get(input) {
                    debug!(
                        "Using existing pubkey for placeholder {}: {}",
                        input, pubkey
                    );
                    return Ok(pubkey.to_string());
                }

                // Generate new keypair for this placeholder (same logic as static benchmarks)
                let keypair = solana_sdk::signer::keypair::Keypair::new();
                let pubkey = keypair.pubkey();

                // Store in both maps (same as static benchmarks)
                env.pubkey_map.insert(input.to_string(), pubkey);
                env.keypair_map.insert(input.to_string(), keypair);

                info!(
                    "Generated new address for placeholder '{}': {}",
                    input, pubkey
                );
                Ok(pubkey.to_string())
            } else {
                // Fallback to simple generation if no SolanaEnv available
                debug!(
                    "No SolanaEnv available, using simple pubkey generation for placeholder: {}",
                    input
                );
                Ok(Pubkey::new_unique().to_string())
            }
        } else {
            // Return as-is if not a placeholder
            Ok(input.to_string())
        }
    }

    /// Resolve wallet context including balances and prices
    #[instrument(skip(self))]
    pub async fn resolve_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        debug!("Resolving context for wallet: {}", pubkey);

        // First resolve any placeholders to real pubkeys
        let resolved_pubkey = self.resolve_placeholder(pubkey).await?;
        debug!("Resolved pubkey: {} -> {}", pubkey, resolved_pubkey);

        // Check cache first (using resolved pubkey as cache key)
        {
            let mut cache = self.wallet_cache.lock().await;
            if let Some(entry) = cache.get(&resolved_pubkey) {
                if !entry.is_expired(WALLET_CACHE_TTL) {
                    trace!("Using cached wallet context");
                    return Ok(entry.data.clone());
                }
            }
        }

        // Resolve context from sources (mock for now)
        let context = self.resolve_fresh_wallet_context(&resolved_pubkey).await?;

        // Cache the result (using resolved pubkey as key)
        {
            let mut cache = self.wallet_cache.lock().await;
            cache.put(resolved_pubkey.clone(), CacheEntry::new(context.clone()));
        }

        Ok(context)
    }

    /// Resolve fresh wallet context from data sources
    async fn resolve_fresh_wallet_context(&self, pubkey: &str) -> Result<WalletContext> {
        // For now, return mock data - will be implemented in next task
        // Note: pubkey is already resolved to a real pubkey at this point
        Ok(WalletContext {
            owner: pubkey.to_string(),
            sol_balance: 5_000_000_000, // 5 SOL in lamports
            token_balances: std::collections::HashMap::new(),
            token_prices: std::collections::HashMap::new(),
            total_value_usd: 600.0, // Mock value
        })
    }

    /// Get placeholder mappings (useful for debugging)
    pub async fn get_placeholder_mappings(&self) -> HashMap<String, String> {
        if let Some(ref solana_env) = self.solana_env {
            let env = solana_env.lock().await;
            env.pubkey_map
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Get token price with caching
    #[instrument(skip(self))]
    pub async fn get_token_price(&self, token_mint: &str) -> Result<f64> {
        debug!("Getting price for token: {}", token_mint);

        // Check cache first
        {
            let mut cache = self.price_cache.lock().await;
            if let Some(entry) = cache.get(token_mint) {
                if !entry.is_expired(PRICE_CACHE_TTL) {
                    trace!("Using cached price");
                    return Ok(entry.data);
                }
            }
        }

        // Fetch fresh price (mock for now)
        let price = self.fetch_fresh_token_price(token_mint).await?;

        // Cache the result
        {
            let mut cache = self.price_cache.lock().await;
            cache.put(token_mint.to_string(), CacheEntry::new(price));
        }

        Ok(price)
    }

    /// Fetch fresh token price from data sources
    async fn fetch_fresh_token_price(&self, token_mint: &str) -> Result<f64> {
        // Mock prices for common tokens - will be implemented with real data
        match token_mint {
            "So11111111111111111111111111111111111111112" => Ok(150.0), // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Ok(1.0),  // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Ok(1.0),  // USDT
            _ => Ok(1.0),                                               // Default price
        }
    }

    /// Clear all caches (useful for testing or force refresh)
    #[instrument(skip(self))]
    pub async fn clear_caches(&self) {
        debug!("Clearing all caches");
        self.wallet_cache.lock().await.clear();
        self.price_cache.lock().await.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let wallet_len = self.wallet_cache.lock().await.len();
        let price_len = self.price_cache.lock().await.len();
        (wallet_len, price_len)
    }
}

impl Default for ContextResolver {
    fn default() -> Self {
        Self::new()
    }
}
