//! Benchmark Runner for Evaluating AI-Generated DeFi Flows
//!
//! This module provides a unified benchmarking framework that supports both static
//! and dynamic benchmark execution, eliminating duplication across test and benchmark
//! scenarios by providing common setup, execution, and verification patterns.

use anyhow::{anyhow, Context, Result};
use jup_sdk::surfpool::SurfpoolClient;
use reev_lib::get_keypair;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::{env, path::PathBuf, time::Instant};
use tracing::{error, info};
use tracing_subscriber;

use crate::benchmark::{
    runner::{types::Flow, DynamicBenchmarkRunner, StaticBenchmarkRunner},
    types::BenchmarkReport,
};

/// Configuration for benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Whether to use surfpool for deterministic testing
    pub use_surfpool: bool,
    /// Whether to initialize surfpool if not running
    pub auto_start_surfpool: bool,
    /// Timeout for benchmark execution in seconds
    pub execution_timeout_secs: u64,
    /// Whether to collect detailed metrics
    pub detailed_metrics: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            use_surfpool: true,
            auto_start_surfpool: true,
            execution_timeout_secs: 30,
            detailed_metrics: true,
        }
    }
}

/// Results of a benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkExecutionResult {
    /// The benchmark report generated
    pub report: BenchmarkReport,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether the benchmark completed successfully
    pub success: bool,
    /// Any error message if execution failed
    pub error: Option<String>,
}

/// Unified benchmark runner that supports both static and dynamic benchmark execution
pub struct BenchmarkRunner {
    pub pubkey: Pubkey,
    config: BenchmarkConfig,
    initialized: bool,
    static_runner: StaticBenchmarkRunner,
    dynamic_runner: Option<DynamicBenchmarkRunner>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner with the default keypair and configuration
    pub fn new() -> Result<Self> {
        let config = BenchmarkConfig::default();
        Self::with_config(config)
    }

    /// Create a new benchmark runner with custom configuration
    pub fn with_config(config: BenchmarkConfig) -> Result<Self> {
        // Initialize tracing for all benchmarks (runs only once)
        let _ = tracing_subscriber::fmt::try_init();

        let keypair = get_keypair()?;
        let pubkey = keypair.pubkey();
        info!("✅ Loaded default keypair: {pubkey}");
        info!("🔑 Using keypair from ~/.config/solana/id.json");

        Ok(Self {
            pubkey,
            config,
            initialized: false,
            static_runner: StaticBenchmarkRunner::new(),
            dynamic_runner: None,
        })
    }

    /// Initialize the benchmark environment
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Initialize benchmark environment
        dotenvy::dotenv().ok();

        // Check for ZAI_API_KEY if needed for dynamic benchmarks
        if self.config.auto_start_surfpool || self.config.use_surfpool {
            let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
                anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
            })?;

            info!("✅ ZAI_API_KEY is configured");
        }

        // Initialize surfpool if needed
        if self.config.use_surfpool {
            // Always kill existing surfpool process to ensure clean state for each benchmark
            info!("🧹 Killing existing surfpool process for clean benchmark environment...");
            if self.config.auto_start_surfpool {
                reev_lib::server_utils::kill_existing_surfpool(8899).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // Start a fresh surfpool instance
                info!("🚀 Starting fresh surfpool instance...");
                Self::ensure_surfpool_running().await?;
                info!("✅ SURFPOOL is now running with clean state");
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Execute a static benchmark from a YML file
    pub async fn execute_static_benchmark(
        &mut self,
        yml_file_path: &PathBuf,
    ) -> Result<BenchmarkExecutionResult> {
        let start_time = Instant::now();

        // Ensure environment is initialized
        if !self.initialized {
            self.initialize().await?;
        }

        info!("🧪 Starting static benchmark: {}", yml_file_path.display());

        // Execute the benchmark using the static runner
        // Load the flow from the YML file
        // Load flow from YML file
        let flow_content =
            std::fs::read_to_string(yml_file_path).context("Failed to read benchmark YML file")?;
        let flow: Flow =
            serde_yaml::from_str(&flow_content).context("Failed to parse benchmark YML file")?;

        match self.static_runner.execute_flow(&flow).await {
            Ok(report) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                info!("✅ Static benchmark completed in {}ms", execution_time);
                info!("📊 Overall score: {}", report.overall_score);

                Ok(BenchmarkExecutionResult {
                    report,
                    execution_time_ms: execution_time,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                error!("❌ Static benchmark failed: {}", e);

                Ok(BenchmarkExecutionResult {
                    report: BenchmarkReport::default(),
                    execution_time_ms: execution_time,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Execute a dynamic benchmark from a prompt
    pub async fn execute_dynamic_benchmark(
        &mut self,
        prompt: &str,
    ) -> Result<BenchmarkExecutionResult> {
        let start_time = Instant::now();

        // Ensure environment is initialized
        if !self.initialized {
            self.initialize().await?;
        }

        // Initialize dynamic runner if not already done
        if self.dynamic_runner.is_none() {
            info!("🚀 Initializing dynamic benchmark runner...");
            let mut runner = DynamicBenchmarkRunner::new().await?;
            runner.initialize().await?;
            self.dynamic_runner = Some(runner);
        }

        info!("🧪 Starting dynamic benchmark: {}", prompt);

        // Execute the benchmark using the dynamic runner
        match self
            .dynamic_runner
            .as_mut()
            .unwrap()
            .execute_prompt(prompt)
            .await
        {
            Ok(report) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                info!("✅ Dynamic benchmark completed in {}ms", execution_time);
                info!("📊 Overall score: {}", report.overall_score);

                Ok(BenchmarkExecutionResult {
                    report,
                    execution_time_ms: execution_time,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                error!("❌ Dynamic benchmark failed: {}", e);

                Ok(BenchmarkExecutionResult {
                    report: BenchmarkReport::default(),
                    execution_time_ms: execution_time,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Execute multiple benchmarks from a directory
    pub async fn execute_benchmark_directory(
        &mut self,
        dir_path: &PathBuf,
    ) -> Result<Vec<BenchmarkExecutionResult>> {
        let mut results = Vec::new();

        // Find all YML files in the directory
        let yml_files = std::fs::read_dir(dir_path)
            .context("Failed to read benchmark directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "yml" || ext == "yaml")
            })
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();

        info!("🔍 Found {} benchmark files", yml_files.len());

        for yml_file in yml_files {
            let result = self.execute_static_benchmark(&yml_file).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get the pubkey of the benchmark runner
    pub fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    /// Reset wallet balance to a known state for benchmarking
    pub async fn reset_wallet_balance(&self) -> Result<()> {
        if !self.config.use_surfpool {
            info!("⚠️ Skipping wallet reset as surfpool is not enabled");
            return Ok(());
        }

        info!("🔄 Resetting wallet balance for benchmark...");

        let surfpool_client = SurfpoolClient::new("http://localhost:8899");

        // Airdrop 5 SOL to ensure we have enough for benchmarks
        surfpool_client
            .set_account(&self.pubkey.to_string(), 5_000_000_000)
            .await
            .map_err(|e| anyhow!("Failed to reset wallet balance: {e}"))?;

        info!("✅ Wallet balance reset to 5 SOL");
        Ok(())
    }

    /// Set token balance for benchmarking
    pub async fn set_token_balance(&mut self, mint: &str, amount: u64) -> Result<()> {
        if !self.config.use_surfpool {
            info!("⚠️ Skipping token balance set as surfpool is not enabled");
            return Ok(());
        }

        let symbol = match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT",
            "So11111111111111111111111111111111111111112" => "SOL",
            _ => return Err(anyhow!("Unknown token mint: {mint}")),
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

        // For SPL tokens, we need to set up the token balance in the test blockchain
        if mint != "So11111111111111111111111111111111111111112" {
            let surfpool_client = SurfpoolClient::new("http://localhost:8899");

            // Set the token account with the specified amount
            surfpool_client
                .set_token_account(&self.pubkey.to_string(), mint, amount)
                .await
                .map_err(|e| anyhow!("Failed to set token account: {e}"))?;

            info!("✅ Set {} token balance to {}", symbol, amount);
        }

        Ok(())
    }

    /// Generate a summary report from multiple benchmark results
    pub fn generate_summary_report(&self, results: &[BenchmarkExecutionResult]) -> String {
        let total_count = results.len();
        let success_count = results.iter().filter(|r| r.success).count();
        let total_time_ms: u64 = results.iter().map(|r| r.execution_time_ms).sum();
        let avg_score = if !results.is_empty() {
            results
                .iter()
                .filter(|r| r.success)
                .map(|r| r.report.overall_score)
                .sum::<f64>()
                / success_count.max(1) as f64
        } else {
            0.0
        };

        format!(
            r#"
Benchmark Summary Report
========================
Total Benchmarks: {}
Successful: {} ({:.1}%)
Failed: {} ({:.1}%)
Total Execution Time: {}ms ({:.2}s)
Average Score: {:.3}
"#,
            total_count,
            success_count,
            (success_count as f64 / total_count.max(1) as f64) * 100.0,
            total_count - success_count,
            ((total_count - success_count) as f64 / total_count.max(1) as f64) * 100.0,
            total_time_ms,
            total_time_ms as f64 / 1000.0,
            avg_score
        )
    }

    /// Helper function to start surfpool and wait for it to be ready
    async fn ensure_surfpool_running() -> Result<()> {
        use solana_client::rpc_client::RpcClient;
        use std::process::{Command, Stdio};

        // Check if surfpool is already running
        let rpc_client = RpcClient::new("http://localhost:8899".to_string());

        match rpc_client.get_latest_blockhash() {
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
            .context("Failed to start surfpool process")?;

        info!("🚀 Surfpool starting with PID: {}", output.id());

        // Wait for surfpool to be ready
        info!("⏳ Waiting for surfpool to become ready...");
        for i in 0..30 {
            // Wait up to 15 seconds
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            match rpc_client.get_latest_blockhash() {
                Ok(_) => {
                    info!("✅ Surfpool is ready after {} attempts", i + 1);
                    return Ok(());
                }
                Err(_) => {
                    // Continue waiting
                }
            }
        }

        Err(anyhow!("Timed out waiting for surfpool to become ready"))
    }
}

/// Macro to simplify writing parameterized benchmark tests
#[macro_export]
macro_rules! create_parameterized_benchmark {
    (
        $test_name:ident,
        $benchmarks:expr
    ) => {
        #[rstest::rstest]
        #[tokio::test(flavor = "multi_thread")]
        $benchmarks
        async fn $test_name(params: (PathBuf, &str)) -> Result<()> {
            let (benchmark_path, test_name) = params;

            let mut runner = $crate::benchmark::runner::BenchmarkRunner::new()?;
            runner.initialize().await?;

            info!("🧪 Running benchmark case: {}", test_name);

            let result = runner.execute_static_benchmark(&benchmark_path).await?;

            assert!(
                result.success,
                "Benchmark '{}' failed with error: {:?}",
                test_name,
                result.error
            );

            info!("✅ Benchmark case '{}' completed with score: {}", test_name, result.report.overall_score);
            Ok(())
        }
    };
}
