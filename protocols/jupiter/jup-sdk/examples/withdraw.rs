use anyhow::Result;
use jup_sdk::lend::withdraw;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::str::FromStr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    let signer = Keypair::new();
    let asset = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount = 100000; // 0.1 USDC

    info!("--- Running Jupiter Lend Withdraw ---");
    withdraw(signer, asset, amount).await?;
    info!("--- Withdraw Complete ---");

    Ok(())
}
