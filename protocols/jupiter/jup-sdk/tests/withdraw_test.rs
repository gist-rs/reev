//! Integration test for real mainnet lend withdraw transactions using the Jupiter SDK.
//! This test builds unsigned transactions that can be signed by a wallet.

use anyhow::Result;
use jup_sdk::{Jupiter, models::WithdrawParams};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use std::str::FromStr;
use tracing::info;

#[tokio::test(flavor = "multi_thread")]
async fn withdraw_test() -> Result<()> {
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

    let withdraw_params = WithdrawParams {
        asset_mint: usdc_mint,
        amount: 10_000, // 0.01 USDC
    };

    info!("--- Building Jupiter Lend Withdraw Transaction ---");

    // Build an unsigned transaction ready for wallet signing
    let unsigned_tx = jupiter_client
        .with_signer(&signer)
        .withdraw(withdraw_params)
        .build_unsigned_transaction()
        .await?;

    info!("✅ Withdraw transaction built successfully!");
    info!(
        "   Transaction signature: {}",
        unsigned_tx.transaction.signatures[0]
    );
    info!(
        "   Last valid block height: {}",
        unsigned_tx.last_valid_block_height
    );

    // Log transaction details for inspection
    info!("   Transaction details: {:#?}", unsigned_tx);

    // Additional assertions
    assert_eq!(unsigned_tx.transaction.signatures.len(), 1);
    assert_eq!(unsigned_tx.transaction.signatures[0], Signature::default());
    assert!(unsigned_tx.last_valid_block_height > 0);
    assert_eq!(unsigned_tx.transaction.message.instructions().len(), 1);
    assert_eq!(
        unsigned_tx
            .transaction
            .message
            .header()
            .num_required_signatures,
        1
    );
    assert!(
        !unsigned_tx
            .transaction
            .message
            .static_account_keys()
            .is_empty()
    );
    assert_eq!(
        unsigned_tx.transaction.message.static_account_keys()[0],
        signer.pubkey()
    );

    info!("✅ Transaction ready for wallet signing");
    info!("   - Send this to a wallet provider (e.g., Phantom, Solflare)");
    info!("   - Wallet should sign and submit to mainnet");

    Ok(())
}
