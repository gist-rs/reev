use anyhow::Result;
use jup_sdk::{api::tokens::search_tokens, models::TokenSearchParams};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    info!("--- Jupiter Token Search Example ---");

    // Example 1: Search for multiple tokens
    info!("Searching for SOL and USDC tokens...");

    let params = TokenSearchParams {
        query: "So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
    };

    match search_tokens(&params).await {
        Ok(tokens) => {
            info!("✅ Found {} tokens", tokens.len());

            for token in &tokens {
                info!("🪙 {} ({})", token.name, token.symbol);
                info!("   ID: {}", token.id);
                info!("   Decimals: {}", token.decimals);

                if let Some(price) = token.usd_price {
                    if token.symbol == "USDC" {
                        info!("   Price: ${:.6}", price);
                    } else {
                        info!("   Price: ${:.2}", price);
                    }
                }

                if let Some(verified) = token.is_verified {
                    info!("   Verified: {}", verified);
                }

                if let Some(liquidity) = token.liquidity {
                    info!("   Liquidity: ${:.2}", liquidity);
                }

                if let Some(stats) = &token.stats_24h {
                    if let Some(volume_change) = stats.volume_change {
                        info!("   24h Volume Change: {:.2}%", volume_change);
                    }
                    if let Some(price_change) = stats.price_change {
                        info!("   24h Price Change: {:.2}%", price_change);
                    }
                }

                info!("   ---");
            }
        }
        Err(e) => {
            info!("❌ Token search failed: {:#?}", e);
            return Err(e);
        }
    }

    // Example 2: Search for a single token by symbol
    info!("\nSearching for USDT token...");

    let usdt_params = TokenSearchParams {
        query: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
    };

    match search_tokens(&usdt_params).await {
        Ok(tokens) => {
            if !tokens.is_empty() {
                let usdt = &tokens[0];
                info!("✅ Found {} token", usdt.symbol);
                info!("   Name: {}", usdt.name);
                info!("   Price: ${:.6}", usdt.usd_price.unwrap_or(0.0));

                if let Some(tags) = &usdt.tags {
                    info!("   Tags: {:?}", tags);
                }
            } else {
                info!("❌ USDT token not found");
            }
        }
        Err(e) => {
            info!("❌ USDT search failed: {:#?}", e);
        }
    }

    // Example 3: Search with multiple tokens including some that might not exist
    info!("\nSearching with multiple tokens...");

    let multi_params = TokenSearchParams {
        query: "So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v,Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB,invalid_token_address".to_string(),
    };

    match search_tokens(&multi_params).await {
        Ok(tokens) => {
            info!("✅ Found {} valid tokens", tokens.len());

            for token in &tokens {
                info!("   - {} ({})", token.name, token.symbol);
            }
        }
        Err(e) => {
            info!("❌ Multi-token search failed: {:#?}", e);
        }
    }

    info!("\n--- Token Search Example Complete ---");

    Ok(())
}
