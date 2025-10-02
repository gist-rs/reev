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

    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    let deposit_params = DepositParams {
        asset_mint: sol_mint,
        amount: 100_000_000, // 0.1 SOL (SOL has 9 decimals)
    };

    info!("--- Running Jupiter Lend Deposit Simulation for SOL ---");

    // The .commit() method will orchestrate the full simulation for the deposit
    match jupiter_client.deposit(deposit_params).commit().await {
        Ok(result) => {
            info!("✅ Deposit successful!");
            info!("   Signature: {}", result.signature);
        }
        Err(e) => {
            info!("❌ Deposit simulation failed: {:#?}", e);
        }
    }

    info!("--- Deposit Simulation Complete ---");

    Ok(())
}
