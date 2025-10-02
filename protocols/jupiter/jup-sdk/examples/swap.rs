use anyhow::Result;
use jup_sdk::{Jupiter, models::SwapParams};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
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
    // Ensure your surfpool instance is running on this address
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let jupiter_client = Jupiter::surfpool(rpc_client).with_signer(&signer);

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    let swap_params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 50_000_000, // 50 USDC
        slippage_bps: 500,  // 0.5%
    };

    info!("--- Running Jupiter Swap Simulation ---");

    // The .commit() method orchestrates the entire simulation:
    // 1. Funds the signer wallet with SOL and the input token.
    // 2. Fetches the quote and instructions from the Jupiter API.
    // 3. Builds the transaction with a fresh blockhash.
    // 4. Pre-loads all necessary accounts from mainnet into surfpool.
    // 5. Signs and executes the transaction.
    // 6. Returns a detailed result upon confirmation.
    match jupiter_client.swap(swap_params).commit().await {
        Ok(result) => {
            info!("✅ Swap successful!");
            info!("   Signature: {}", result.signature);
            // You can inspect result.debug_info for more details
        }
        Err(e) => {
            info!("❌ Swap simulation failed: {:#?}", e);
        }
    }

    info!("--- Swap Simulation Complete ---");

    Ok(())
}
