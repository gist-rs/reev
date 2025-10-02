//! Unit test for real mainnet swap transactions using the Jupiter SDK.
//! This test verifies that unsigned transactions can be built for wallet signing.

use anyhow::Result;
use jup_sdk::{Jupiter, models::SwapParams};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use tracing::info;

// This test checks that we can build unsigned transactions for mainnet swaps
#[tokio::test(flavor = "multi_thread")]
async fn test_swap_transaction_building() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info,jup_sdk=info")
        .init();

    // In a real test, you would load this from environment or keypair file
    let signer = Keypair::new();

    // For mainnet, use the default public RPC
    let jupiter_client = Jupiter::new(solana_client::rpc_client::RpcClient::new(
        "https://api.mainnet-beta.solana.com".to_string(),
    ));

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;

    let swap_params = SwapParams {
        input_mint: usdc_mint,
        output_mint: sol_mint,
        amount: 10_000_000, // 10 USDC
        slippage_bps: 500,  // 0.5%
    };

    info!("--- Building Jupiter Swap Transaction ---");

    // Build an unsigned transaction ready for wallet signing
    let unsigned_tx = jupiter_client
        .with_signer(&signer)
        .swap(swap_params)
        .build_unsigned_transaction()
        .await?;

    info!("✅ Swap transaction built successfully!");
    info!(
        "   Transaction signature: {}",
        unsigned_tx.transaction.signatures[0]
    );
    info!(
        "   Last valid block height: {}",
        unsigned_tx.last_valid_block_height
    );

    // Verify the transaction is properly formatted for wallet signing
    assert!(!unsigned_tx.transaction.signatures.is_empty());

    info!("✅ Transaction ready for wallet signing");
    info!("   - Send this to a wallet provider (e.g., Phantom, Solflare)");
    info!("   - Wallet should sign and submit to mainnet");

    Ok(())
}
