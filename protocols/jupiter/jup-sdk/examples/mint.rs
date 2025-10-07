use anyhow::Result;
use jup_sdk::{Jupiter, models::DepositParams};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::str::FromStr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    let signer = Keypair::new();
    let jupiter_client = Jupiter::surfpool().with_signer(&signer);
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    info!("--- Running Jupiter Lend Mint Simulation ---");
    let shares = 100_000; // 0.1 USDC worth of jTokens

    let mint_params = DepositParams {
        asset_mint: usdc_mint,
        amount: shares,
    };

    match jupiter_client.mint(mint_params).commit().await {
        Ok(result) => {
            info!("✅ Mint successful!");
            info!("   Signature: {}", result.signature);
            info!("   Debug info: {:?}", result.debug_info);
        }
        Err(e) => {
            info!("❌ Mint simulation failed: {:#?}", e);
        }
    }

    info!("--- Mint Simulation Complete ---");
    Ok(())
}
