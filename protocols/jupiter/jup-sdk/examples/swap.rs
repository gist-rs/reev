use anyhow::Result;
use jup_sdk::swap::swap;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let native_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    info!("--- Running Jupiter Swap ---");
    swap(usdc_mint, native_mint, 50_000_000, 500).await?;
    info!("--- Swap Complete ---");

    Ok(())
}
