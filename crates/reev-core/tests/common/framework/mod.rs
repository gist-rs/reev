//! Test framework for end-to-end tests
//!
//! This module provides a standardized testing framework that eliminates
//! duplication across all e2e tests by providing common setup, execution,
//! and verification patterns.

use anyhow::Result;
use jup_sdk::surfpool::SurfpoolClient;
use reev_lib::get_keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::env;
use tracing::info;

/// Standardized test runner that handles common setup and execution
#[allow(dead_code)]
pub struct TestRunner {
    pub pubkey: Pubkey,
    initialized: bool,
}

impl TestRunner {
    /// Create a new test runner with the default keypair
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        // Initialize tracing for all tests (runs only once)
        let _ = tracing_subscriber::fmt::try_init();

        let keypair = get_keypair()?;
        let pubkey = keypair.pubkey();
        info!("✅ Loaded default keypair: {pubkey}");
        info!("🔑 Using keypair from ~/.config/solana/id.json");

        Ok(Self {
            pubkey,
            initialized: false,
        })
    }

    /// Initialize the test environment
    #[allow(dead_code)]
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize test environment
        dotenvy::dotenv().ok();

        // Check for ZAI_API_KEY
        let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
            anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
        })?;

        info!("✅ ZAI_API_KEY is configured");

        // Always kill existing surfpool process to ensure clean state for each test
        info!("🧹 Killing existing surfpool process for clean test environment...");
        reev_lib::server_utils::kill_existing_surfpool(8899).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Start a fresh surfpool instance
        info!("🚀 Starting fresh surfpool instance...");
        crate::common::helpers::ensure_surfpool_running().await?;
        info!("✅ SURFPOOL is now running with clean state");

        self.initialized = true;
        Ok(())
    }

    /// Execute a test operation using the standardized flow
    #[allow(dead_code)]
    pub async fn execute_operation<T: crate::common::operations::TestOperation>(
        &self,
        operation: &T,
    ) -> Result<String> {
        info!("\n🧪 Starting test operation: {}", operation.prompt());
        crate::common::operations::execute_standardized_operation(operation, &self.pubkey).await
    }

    /// Get the pubkey of the test runner
    #[allow(dead_code)]
    pub fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    /// Reset wallet balance to a known state for testing
    #[allow(dead_code)]
    pub async fn reset_wallet_balance(&self) -> Result<()> {
        info!("🔄 Resetting wallet balance for test...");

        let surfpool_client = SurfpoolClient::new("http://localhost:8899");

        // Airdrop 5 SOL to ensure we have enough for tests
        surfpool_client
            .set_account(&self.pubkey.to_string(), 5_000_000_000)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to reset wallet balance: {e}"))?;

        info!("✅ Wallet balance reset to 5 SOL");
        Ok(())
    }

    /// Set token balance for testing
    #[allow(dead_code)]
    pub async fn set_token_balance(&mut self, mint: &str, amount: u64) -> Result<()> {
        // In a real implementation, this would interact with the blockchain
        // For testing, we'll just log the token balance setup
        let symbol = match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
            "So11111111111111111111111111111111111111112" => "SOL",
            _ => return Err(anyhow::anyhow!("Unknown token mint: {mint}")),
        };

        // Determine decimals based on token
        let decimals = match symbol {
            "SOL" => 9,
            _ => 6, // Most SPL tokens use 6 decimals
        };

        info!(
            "Setting {} balance to {} ({:.6} tokens)",
            symbol,
            amount,
            amount as f64 / 10_f64.powi(decimals as i32)
        );

        // In a real implementation, this would interact with the test blockchain
        // For now, we just log the setup
        Ok(())
    }
}

/// Macro to simplify writing parameterized tests
#[macro_export]
macro_rules! create_parameterized_test {
    (
        $test_name:ident,
        $operation_type:ty,
        $test_cases:expr
    ) => {
        #[rstest::rstest]
        #[tokio::test(flavor = "multi_thread")]
        $test_cases
        async fn $test_name(params: ($operation_type, &str)) -> Result<()> {
            let (operation, test_name) = params;

            let mut runner = $crate::common::framework::TestRunner::new()?;
            runner.initialize().await?;

            info!("🧪 Running test case: {}", test_name);

            let signature = runner.execute_operation(&operation).await?;

            info!("✅ Test case '{}' completed with signature: {}", test_name, signature);
            Ok(())
        }
    };
}
