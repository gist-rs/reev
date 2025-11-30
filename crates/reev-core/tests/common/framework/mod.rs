//! Test framework for end-to-end tests
//!
//! This module provides a standardized testing framework that eliminates
//! duplication across all e2e tests by providing common setup, execution,
//! and verification patterns.

use anyhow::Result;
use reev_lib::get_keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::env;
use tracing::info;

/// Standardized test runner that handles common setup and execution
pub struct TestRunner {
    pub pubkey: Pubkey,
    initialized: bool,
}

impl TestRunner {
    /// Create a new test runner with the default keypair
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
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize test environment
        dotenvy::dotenv().ok();
        env::set_var("REEV_ENHANCED_OTEL", "0");

        // Check for ZAI_API_KEY
        let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
            anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
        })?;

        info!("✅ ZAI_API_KEY is configured");

        // Check if SURFPOOL is running
        match solana_client::nonblocking::rpc_client::RpcClient::new(
            "http://localhost:8899".to_string(),
        )
        .get_latest_blockhash()
        .await
        {
            Ok(_) => {
                info!("✅ SURFPOOL is running and ready");
            }
            Err(_) => {
                // SURFPOOL is not running, try to start it
                info!("⏳ SURFPOOL is not running, attempting to start it...");
                reev_lib::server_utils::kill_existing_surfpool(8899).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Use ensure_surfpool_running from helpers to start SURFPOOL
                crate::common::helpers::ensure_surfpool_running().await?;
                info!("✅ SURFPOOL is now running");
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Execute a test operation using the standardized flow
    pub async fn execute_operation<T: crate::common::operations::TestOperation>(
        &self,
        operation: &T,
    ) -> Result<String> {
        info!("\n🧪 Starting test operation: {}", operation.prompt());
        crate::common::operations::execute_standardized_operation(operation, &self.pubkey).await
    }

    /// Get the pubkey of the test runner
    pub fn pubkey(&self) -> Pubkey {
        self.pubkey
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
