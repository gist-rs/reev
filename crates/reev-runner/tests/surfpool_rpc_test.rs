//! # Core Testing Philosophy: Surfpool + Real Mainnet Programs
//!
//! All integration tests in the `reev` framework operate on `surfpool`, a high-speed
//! local Solana test validator. `surfpool` instantly forks Solana mainnet, meaning
//! any on-chain account not explicitly mocked in the test setup is fetched live from
//! mainnet. This allows tests to interact with real, deployed programs (like SPL Token
//! or Jupiter) without any mocking of program logic. Test assertions are based on the
//! real outcomes of these transactions. This approach ensures that a passing test gives
//! a strong signal of real-world viability.

//! # Surfpool RPC Cheat Code Verification Test
//!
//! This test file provides an isolated verification for the `surfnet_setTokenAccount`
//! RPC "cheat code". The purpose is to prove that the cheat code works as expected
//! by interacting directly with the `surfpool` RPC server, completely bypassing
//! the `SolanaEnv` and its complex setup requirements.
//!
//! These tests require a running Solana validator at http://127.0.0.1:8899
//! They will be skipped if the validator is not available.

mod common;

use anyhow::{Context, Result, anyhow};
use jup_sdk::surfpool::SurfpoolClient;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    str::FromStr,
    time::Duration,
};
use tokio::time::sleep;
use tracing::{error, info};

/// A simple RAII guard to ensure the reev-agent process is killed after the test.
struct AgentProcessGuard {
    process: Child,
}

impl Drop for AgentProcessGuard {
    fn drop(&mut self) {
        info!("Shutting down reev-agent for test...");
        if let Err(e) = self.process.kill() {
            error!(error = ?e, "Failed to kill re-agent process");
        }
    }
}

/// Starts the `reev-agent` process, which hosts the `surfpool` RPC server.
/// It waits for the agent to become healthy before returning.
async fn start_agent_for_test() -> Result<AgentProcessGuard> {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir)?;
    let log_file_path = log_dir.join("reev-agent-rpc-test.log");
    let log_file = fs::File::create(&log_file_path)?;
    let stderr_log = log_file.try_clone()?;

    info!("Starting reev-agent for RPC test...");
    let agent_process = Command::new("cargo")
        .args(["run", "--package", "reev-agent"])
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(stderr_log))
        .spawn()
        .context("Failed to spawn reev-agent process using 'cargo run'")?;

    let guard = AgentProcessGuard {
        process: agent_process,
    };

    // Health check to ensure the server is ready.
    let client = reqwest::Client::new();
    let health_check_url = "http://127.0.0.1:9090/health";
    for _ in 0..20 {
        if let Ok(response) = client.get(health_check_url).send().await {
            if response.status().is_success() {
                info!("reev-agent is healthy.");
                return Ok(guard);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!(
        "Timed out waiting for reev-agent to become healthy."
    ))
}

/// Verifies that the `surfnet_setTokenAccount` RPC method correctly sets
/// the balance of an SPL token account on a local `surfpool` instance.
#[tokio::test(flavor = "multi_thread")]
async fn test_set_usdc_balance_via_rpc() -> Result<()> {
    // Skip test if validator is not available
    if !is_validator_available().await {
        println!("⚠️  Skipping test: Solana validator not available at http://127.0.0.1:8899");
        return Ok(());
    }

    // Initialize tracing to capture logs.
    let _ = tracing_subscriber::fmt::try_init();

    // 1. Start the agent/surfpool server. The guard ensures it's killed on test exit.
    let _agent_guard = start_agent_for_test().await?;
    info!("✅ Agent started, surfpool RPC is live.");

    // 2. Define the wallet, mint, and amount.
    let user_wallet = Pubkey::new_unique();
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount_to_set = 100_000_000; // 100 USDC with 6 decimals.

    // 3. Use the cheat code to set the token balance.
    let surfpool_client = SurfpoolClient::new("http://127.0.0.1:8899");
    surfpool_client
        .set_token_account(
            &user_wallet.to_string(),
            &usdc_mint.to_string(),
            amount_to_set,
        )
        .await?;
    info!("✅ Cheat code `surfnet_setTokenAccount` executed successfully.");

    // 4. Verify the balance using a standard RPC client.
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    let user_usdc_ata = get_associated_token_address(&user_wallet, &usdc_mint);

    // Retry mechanism to handle eventual consistency of the RPC server state.
    let mut balance_check_attempts = 0;
    loop {
        if balance_check_attempts >= 10 {
            return Err(anyhow!("Timed out waiting for token balance to update."));
        }
        match rpc_client.get_token_account_balance(&user_usdc_ata) {
            Ok(balance) => {
                info!("✅ Fetched balance: {}", balance.ui_amount_string);
                assert_eq!(
                    balance.amount.parse::<u64>()?,
                    amount_to_set,
                    "The token account balance must match the amount set by the cheat code."
                );
                info!("✅ Assertion passed!");
                break;
            }
            Err(_) => {
                balance_check_attempts += 1;
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    Ok(())
}

/// Check if the Solana validator is available
async fn is_validator_available() -> bool {
    use std::time::Duration;

    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8899/";

    // Try to connect with a short timeout
    match client.get(url).timeout(Duration::from_secs(2)).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
