//! Test fixtures for rstest
//!
//! This module provides rstest fixtures that can be used across all e2e tests
//! to reduce setup duplication and ensure consistent test environments.

use anyhow::Result;
use jup_sdk::surfpool::SurfpoolClient;
use rstest::fixture;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use std::{env, str::FromStr};

use crate::common::pubkeys;

/// Fixture that provides the default Solana keypair
#[fixture]
pub async fn default_keypair() -> Keypair {
    // Load the default Solana keypair from ~/.config/solana/id.json
    let home_dir = std::env::var_os("HOME")
        .and_then(|h| h.into_string().ok())
        .expect("Could not find home directory");
    let keypair_path = std::path::PathBuf::from(home_dir).join(".config/solana/id.json");

    let _keypair_bytes = std::fs::read(&keypair_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read keypair from {}: {}",
            keypair_path.display(),
            e
        )
    });

    // Read the keypair manually to avoid serde deserialization issues
    let keypair = solana_sdk::signer::keypair::read_keypair_file(&keypair_path)
        .unwrap_or_else(|e| panic!("Failed to parse keypair: {e}"));
    keypair
}

/// Test fixture for the target public key
#[fixture]
pub fn target_pubkey() -> Pubkey {
    Pubkey::from_str(pubkeys::TARGET).expect("Invalid target public key")
}

/// Fixture that provides the RPC client
#[fixture]
pub async fn rpc_client() -> RpcClient {
    RpcClient::new("http://localhost:8899".to_string())
}

/// Fixture that provides the SURFPOOL client
#[fixture]
pub async fn surfpool_client() -> SurfpoolClient {
    SurfpoolClient::new("http://localhost:8899")
}

/// Fixture that provides a configured environment
#[fixture]
pub async fn configured_env() -> Result<()> {
    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    tracing::info!("✅ ZAI_API_KEY is configured");

    // Disable enhanced OTEL logging to reduce verbosity
    env::set_var("REEV_ENHANCED_OTEL", "0");

    Ok(())
}

/// Fixture that sets up a wallet for transfer tests
#[fixture]
pub async fn transfer_wallet_setup(
    #[future] default_keypair: Keypair,
    #[future] surfpool_client: SurfpoolClient,
) -> Result<(Keypair, f64, f64)> {
    let keypair = default_keypair.await;
    let pubkey = keypair.try_pubkey().expect("Invalid pubkey");

    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        crate::common::helpers::setup_wallet_for_transfer(&pubkey, &surfpool_client.await).await?;

    tracing::info!(
        "✅ Wallet setup completed with {} SOL and {} USDC",
        initial_sol_balance,
        initial_usdc_balance
    );

    Ok((keypair, initial_sol_balance, initial_usdc_balance))
}

/// Fixture that sets up a wallet for swap tests
#[fixture]
pub async fn swap_wallet_setup(
    #[future] default_keypair: Keypair,
    #[future] surfpool_client: SurfpoolClient,
) -> Result<(Keypair, f64, f64)> {
    let keypair = default_keypair.await;
    let pubkey = keypair.try_pubkey().expect("Invalid pubkey");

    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        crate::common::helpers::setup_wallet_for_swap(&pubkey, &surfpool_client.await).await?;

    tracing::info!(
        "✅ Wallet setup completed with {initial_sol_balance} SOL and {initial_usdc_balance} USDC"
    );

    Ok((keypair, initial_sol_balance, initial_usdc_balance))
}

/// Fixture that sets up a wallet for lend tests
#[fixture]
pub async fn lend_wallet_setup(
    #[future] default_keypair: Keypair,
    #[future] surfpool_client: SurfpoolClient,
) -> Result<(Keypair, f64, f64)> {
    let keypair = default_keypair.await;
    let pubkey = keypair.try_pubkey().expect("Invalid pubkey");

    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        crate::common::helpers::setup_wallet_for_lend(&pubkey, &surfpool_client.await).await?;

    tracing::info!(
        "✅ Wallet setup completed with {initial_sol_balance} SOL and {initial_usdc_balance} USDC"
    );

    Ok((keypair, initial_sol_balance, initial_usdc_balance))
}
