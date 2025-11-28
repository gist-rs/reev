//! Common utilities for end-to-end tests
//!
//! This module provides shared helper functions used across multiple e2e tests,
//! reducing code duplication and maintaining consistency.

pub mod mock_helpers;

use anyhow::{anyhow, Result};
use jup_sdk::surfpool::SurfpoolClient;
use reev_lib::get_keypair;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};
use tracing::info;

/// Target account for SOL transfer tests
#[allow(dead_code)]
pub const TARGET_PUBKEY: &str = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

/// Helper function to start surfpool and wait for it to be ready
#[allow(dead_code)]
pub async fn ensure_surfpool_running() -> Result<()> {
    // Kill any existing surfpool process to ensure clean state
    info!("🧹 Killing any existing surfpool processes...");
    reev_lib::server_utils::kill_existing_surfpool(8899).await?;

    // First check if surfpool is already running
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());

    match rpc_client.get_latest_blockhash().await {
        Ok(_) => {
            info!("✅ Surfpool is already running and accessible");
            return Ok(());
        }
        Err(_) => {
            info!("🚀 Surfpool not running, need to start it...");
        }
    }

    // Start surfpool in background
    info!("🚀 Starting surfpool...");
    let output = Command::new("surfpool")
        .args([
            "start",
            "--rpc-url",
            "https://api.mainnet-beta.solana.com",
            "--port",
            "8899",
            "--no-tui",
            "--no-deploy",
            "--disable-instruction-profiling",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow!("Failed to start surfpool: {e}. Is surfpool installed?"))?;

    let pid = output.id();
    info!("✅ Started surfpool with PID: {}", pid);

    // Wait for surfpool to be ready
    info!("⏳ Waiting for surfpool to be ready...");
    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        sleep(Duration::from_secs(2)).await;
        attempts += 1;

        match rpc_client.get_latest_blockhash().await {
            Ok(_) => {
                info!("✅ Surfpool is ready after {} attempts", attempts);
                return Ok(());
            }
            Err(_) => {
                info!(
                    "Attempt {}/{}: Surfpool not ready yet",
                    attempts, max_attempts
                );
            }
        }
    }

    Err(anyhow!(
        "Surfpool did not become ready after {max_attempts} attempts"
    ))
}

/// Setup wallet with SOL for tests
#[allow(dead_code)]
pub async fn setup_wallet(pubkey: &Pubkey, rpc_client: &RpcClient) -> Result<u64> {
    info!("🔄 Setting up test wallet with SOL...");

    // Check current SOL balance
    let balance = rpc_client.get_balance(pubkey).await?;
    info!("💰 Current SOL balance: {} lamports", balance);

    // If balance is less than 2 SOL, airdrop more
    if balance < 2_000_000_000 {
        info!("🔄 Airdropping additional SOL to account...");
        let signature = rpc_client.request_airdrop(pubkey, 2_000_000_000).await?;

        // Wait for airdrop to confirm
        let mut confirmed = false;
        let mut attempts = 0;
        while !confirmed && attempts < 10 {
            sleep(Duration::from_secs(2)).await;
            confirmed = rpc_client.confirm_transaction(&signature).await?;
            attempts += 1;
        }

        let new_balance = rpc_client.get_balance(pubkey).await?;
        info!("✅ Account balance: {} lamports after airdrop", new_balance);
        Ok(new_balance)
    } else {
        Ok(balance)
    }
}

/// Initialize tracing with appropriate filters for tests
#[allow(dead_code)]
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reev_core::execution::tool_executor=error,warn".into()),
        )
        .with_target(false) // Remove target module prefixes for cleaner output
        .try_init();
}

/// Get the default Solana keypair for tests
#[allow(dead_code)]
pub fn get_test_keypair() -> Result<keypair::Keypair> {
    get_keypair().map_err(|e| anyhow::anyhow!("Failed to load keypair from default location: {e}"))
}

/// Parse a public key string with error handling
#[allow(dead_code)]
pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey> {
    pubkey_str
        .parse::<Pubkey>()
        .map_err(|e| anyhow!("Invalid pubkey: {e}"))
}

/// Setup wallet with SOL and USDC for swap tests
#[allow(dead_code)]
pub async fn setup_wallet_for_swap(
    pubkey: &Pubkey,
    surfpool_client: &SurfpoolClient,
) -> Result<(f64, f64)> {
    // Airdrop 5 SOL to the account
    info!("🔄 Airdropping 5 SOL to account...");
    surfpool_client
        .set_account(&pubkey.to_string(), 5_000_000_000)
        .await
        .map_err(|e| anyhow!("Failed to airdrop SOL: {e}"))?;

    // Verify SOL balance
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    let balance = rpc_client.get_balance(pubkey).await?;
    let sol_balance = balance as f64 / 1_000_000_000.0_f64;

    info!("✅ Account balance: {sol_balance} SOL");

    // Set up USDC token account with 100 USDC
    let usdc_mint = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let usdc_ata = spl_associated_token_account::get_associated_token_address(pubkey, &usdc_mint);

    info!("🔄 Setting up USDC token account with 100 USDC...");
    surfpool_client
        .set_token_account(&pubkey.to_string(), &usdc_mint.to_string(), 100_000_000)
        .await
        .map_err(|e| anyhow!("Failed to set up USDC token account: {e}"))?;

    // Verify USDC balance
    let usdc_balance = rpc_client.get_token_account_balance(&usdc_ata).await?;
    let usdc_amount = &usdc_balance.ui_amount_string;
    info!("✅ USDC balance: {usdc_amount}");

    let usdc_balance_f64 = usdc_balance.ui_amount.unwrap_or(0.0);

    Ok((sol_balance, usdc_balance_f64))
}

/// Setup wallet with SOL and USDC for transfer tests
#[allow(dead_code)]
pub async fn setup_wallet_for_transfer(
    pubkey: &Pubkey,
    surfpool_client: &SurfpoolClient,
) -> Result<(f64, f64)> {
    // Airdrop 5 SOL to account for transaction fees
    info!("🔄 Airdropping 5 SOL to account for transaction fees...");
    surfpool_client
        .set_account(&pubkey.to_string(), 5_000_000_000)
        .await
        .map_err(|e| anyhow!("Failed to airdrop SOL: {e}"))?;

    // Verify SOL balance
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    let balance = rpc_client.get_balance(pubkey).await?;
    let sol_balance = balance as f64 / 1_000_000_000.0_f64;

    info!("✅ Account balance: {sol_balance} SOL");

    // Set up USDC token account with 100 USDC (for completeness, not used in transfer)
    let usdc_mint = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let usdc_ata = spl_associated_token_account::get_associated_token_address(pubkey, &usdc_mint);

    info!("🔄 Setting up USDC token account with 100 USDC for transfer test...");
    surfpool_client
        .set_token_account(&pubkey.to_string(), &usdc_mint.to_string(), 100_000_000)
        .await
        .map_err(|e| anyhow!("Failed to set up USDC token account: {e}"))?;

    // Verify USDC balance
    let usdc_balance = rpc_client.get_token_account_balance(&usdc_ata).await?;
    let usdc_amount = &usdc_balance.ui_amount_string;
    info!("✅ USDC balance: {usdc_amount}");

    let usdc_balance_f64 = usdc_balance.ui_amount.unwrap_or(0.0);

    Ok((sol_balance, usdc_balance_f64))
}

/// Setup wallet with SOL and USDC for lend tests
#[allow(dead_code)]
pub async fn setup_wallet_for_lend(
    pubkey: &Pubkey,
    surfpool_client: &SurfpoolClient,
) -> Result<(f64, f64)> {
    // Airdrop 5 SOL to the account for transaction fees
    info!("🔄 Airdropping 5 SOL to account for transaction fees...");
    surfpool_client
        .set_account(&pubkey.to_string(), 5_000_000_000)
        .await
        .map_err(|e| anyhow!("Failed to airdrop SOL: {e}"))?;

    // Verify SOL balance
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    let balance = rpc_client.get_balance(pubkey).await?;
    let sol_balance = balance as f64 / 1_000_000_000.0_f64;

    info!("✅ Account balance: {sol_balance} SOL");

    // Set up USDC token account with 200 USDC (more than we'll lend for the test)
    let usdc_mint = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let usdc_ata = spl_associated_token_account::get_associated_token_address(pubkey, &usdc_mint);

    info!("🔄 Setting up USDC token account with 200 USDC for lending test...");
    surfpool_client
        .set_token_account(&pubkey.to_string(), &usdc_mint.to_string(), 200_000_000)
        .await
        .map_err(|e| anyhow!("Failed to set up USDC token account: {e}"))?;

    // Verify USDC balance
    let usdc_balance = rpc_client.get_token_account_balance(&usdc_ata).await?;
    let usdc_amount = &usdc_balance.ui_amount_string;
    info!("✅ USDC balance: {usdc_amount}");

    let usdc_balance_f64 = usdc_balance.ui_amount.unwrap_or(0.0);

    Ok((sol_balance, usdc_balance_f64))
}
