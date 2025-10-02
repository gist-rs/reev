use anyhow::Result;
use jup_sdk::{Jupiter, models::DepositParams};
use solana_client::rpc_client::RpcClient;
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

    // Initialize the Jupiter client for surfpool simulation
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let jupiter_client = Jupiter::surfpool(rpc_client).with_signer(&signer);

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    let deposit_params = DepositParams {
        asset_mint: usdc_mint,
        amount: 100_000, // 0.1 USDC
    };

    info!("--- Running Jupiter Lend Deposit Simulation ---");

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
