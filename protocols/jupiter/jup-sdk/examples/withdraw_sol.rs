use anyhow::Result;
use jup_sdk::{
    client::Jupiter,
    models::{DepositParams, WithdrawParams},
};
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
    let amount = 100_000_000; // 0.1 SOL

    // --- Step 1: Perform the Deposit ---
    info!("--- Running Jupiter Lend Deposit Simulation ---");
    let deposit_params = DepositParams {
        asset_mint: sol_mint,
        amount,
    };

    match jupiter_client.deposit(deposit_params).commit().await {
        Ok(result) => {
            info!("✅ Deposit successful!");
            info!("   Signature: {}", result.signature);
        }
        Err(e) => {
            info!("❌ Deposit simulation failed: {:#?}", e);
            // If the deposit fails, we shouldn't proceed to withdraw.
            return Err(e);
        }
    }
    info!("--- Deposit Simulation Complete ---");

    // --- Step 2: Perform the Withdraw ---
    info!("\n--- Running Jupiter Lend Withdraw Simulation ---");
    let withdraw_params = WithdrawParams {
        asset_mint: sol_mint,
        amount, // This will be converted to the full L-token balance internally
    };

    match jupiter_client.withdraw(withdraw_params).commit().await {
        Ok(result) => {
            info!("✅ Withdraw successful!");
            info!("   Signature: {}", result.signature);
        }
        Err(e) => {
            info!("❌ Withdraw simulation failed: {:#?}", e);
        }
    }
    info!("--- Withdraw Simulation Complete ---");

    Ok(())
}
