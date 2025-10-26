//! Unit test for the Jupiter Tokens API.
//! This test verifies that token search works correctly with real network calls.

use anyhow::Result;
use jup_sdk::{api::tokens::search_tokens, models::TokenSearchParams};
use tracing::info;

// This test checks that we can search for tokens using the Jupiter Tokens API
#[tokio::test(flavor = "multi_thread")]
async fn test_token_search() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    info!("--- Testing Jupiter Token Search API ---");

    // Test with SOL and USDC mint addresses
    let params = TokenSearchParams {
        query: "So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    };

    match search_tokens(&params).await {
        Ok(tokens) => {
            info!("✅ Token search successful!");
            info!("   Found {} tokens", tokens.len());

            // Verify we got exactly 2 tokens
            assert_eq!(tokens.len(), 2);

            // Find SOL token
            let sol_token = tokens
                .iter()
                .find(|t| t.id == "So11111111111111111111111111111111111111112")
                .expect("SOL token should be found");

            // Verify SOL token properties
            assert_eq!(sol_token.symbol, "SOL");
            assert_eq!(sol_token.name, "Wrapped SOL");
            assert_eq!(sol_token.decimals, 9);
            assert!(sol_token.is_verified.unwrap_or(false));
            assert!(sol_token.usd_price.is_some());
            assert!(sol_token.liquidity.is_some());

            info!("   SOL Token: {} ({})", sol_token.name, sol_token.symbol);
            if let Some(price) = sol_token.usd_price {
                info!("   SOL Price: ${:.2}", price);
            }
            if let Some(liquidity) = sol_token.liquidity {
                info!("   SOL Liquidity: ${:.2}", liquidity);
            }

            // Find USDC token
            let usdc_token = tokens
                .iter()
                .find(|t| t.id == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .expect("USDC token should be found");

            // Verify USDC token properties
            assert_eq!(usdc_token.symbol, "USDC");
            assert_eq!(usdc_token.name, "USD Coin");
            assert_eq!(usdc_token.decimals, 6);
            assert!(usdc_token.is_verified.unwrap_or(false));
            assert!(usdc_token.usd_price.is_some());
            assert!(usdc_token.total_supply.is_some());

            info!("   USDC Token: {} ({})", usdc_token.name, usdc_token.symbol);
            if let Some(price) = usdc_token.usd_price {
                info!("   USDC Price: ${:.6}", price);
            }
            if let Some(supply) = usdc_token.total_supply {
                info!("   USDC Total Supply: {:.2}", supply);
            }

            // Verify stats are available for both tokens
            assert!(sol_token.stats_24h.is_some(), "SOL should have 24h stats");
            assert!(usdc_token.stats_24h.is_some(), "USDC should have 24h stats");

            // Test SOL 24h stats
            if let Some(stats) = &sol_token.stats_24h {
                info!(
                    "   SOL 24h volume change: {:.2}%",
                    stats.volume_change.unwrap_or(0.0)
                );
                info!(
                    "   SOL 24h price change: {:.2}%",
                    stats.price_change.unwrap_or(0.0)
                );
            }

            // Test USDC 24h stats
            if let Some(stats) = &usdc_token.stats_24h {
                info!(
                    "   USDC 24h volume change: {:.2}%",
                    stats.volume_change.unwrap_or(0.0)
                );
                info!(
                    "   USDC 24h price change: {:.2}%",
                    stats.price_change.unwrap_or(0.0)
                );
            }

            // Log all token details for inspection
            info!("   Token details: {:#?}", tokens);

            info!("✅ Token search test completed successfully!");
        }
        Err(e) => {
            info!("❌ Token search failed: {:#?}", e);
            return Err(e);
        }
    }

    // Test search with a single token
    info!("--- Testing Single Token Search ---");

    let single_token_params = TokenSearchParams {
        query: "So11111111111111111111111111111111111111112".to_string(),
    };

    match search_tokens(&single_token_params).await {
        Ok(tokens) => {
            info!("✅ Single token search successful!");
            info!("   Found {} tokens", tokens.len());
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].symbol, "SOL");
        }
        Err(e) => {
            info!("❌ Single token search failed: {:#?}", e);
            return Err(e);
        }
    }

    info!("--- Token Search Tests Complete ---");

    Ok(())
}
