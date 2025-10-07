use anyhow::Result;
use jup_sdk::{Jupiter, models::DepositParams};
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

    // Create a temporary signer for the simulation
    let signer = Keypair::new();

    // Initialize the Jupiter client for surfpool simulation using the default URL.
    let jupiter_client = Jupiter::surfpool().with_signer(&signer);

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    let redeem_params = DepositParams {
        asset_mint: usdc_mint,
        amount: 100_000, // 0.1 USDC worth of jTokens to redeem
    };

    info!("--- Running Jupiter Lend Redeem Simulation ---");

    // The .commit() method will orchestrate the full simulation for the redeem
    match jupiter_client.redeem(redeem_params).commit().await {
        Ok(result) => {
            info!("✅ Redeem successful!");
            info!("   Signature: {}", result.signature);
        }
        Err(e) => {
            info!("❌ Redeem simulation failed: {:#?}", e);
        }
    }

    info!("--- Redeem Simulation Complete ---");

    Ok(())
}
