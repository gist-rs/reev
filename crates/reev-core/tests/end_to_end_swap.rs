//! End-to-end swap test using default Solana keypair
//!
//! This test loads the wallet from ~/.config/solana/id.json, airdrops SOL via surfpool,
//! uses the planner to process the prompt "swap 1 SOL for USDC", lets the LLM handle
//! tool calling via rig, signs the transaction with the default keypair, and verifies completion.

use anyhow::Result;
use jup_sdk::surfpool::SurfpoolClient;
// ZAIAgent and LlmRequest are now used through the planner
use reev_core::context::ContextResolver;
// init_glm_client is now used through the planner
use reev_core::planner::Planner;
use reev_core::utils::solana::get_keypair;
// YmlFlow is imported through reev_core
use reev_core::Executor;
use reev_types::flow::WalletContext;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
// HashMap is not used directly in this file
use std::env;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::info;

/// Global reference to the surfpool process
static SURFPOOL_PROCESS: std::sync::OnceLock<std::sync::Arc<Mutex<Option<u32>>>> =
    std::sync::OnceLock::new();

/// Helper function to start surfpool and wait for it to be ready
async fn ensure_surfpool_running() -> Result<()> {
    // First check if surfpool is already running
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());

    match rpc_client.get_latest_blockhash().await {
        Ok(_) => {
            info!("✅ Surfpool is already running and accessible");
            return Ok(());
        }
        Err(_) => {
            info!("🚀 Starting surfpool...");
        }
    }

    // Try to start surfpool programmatically
    let process_ref = SURFPOOL_PROCESS.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut process_guard = process_ref
        .lock()
        .map_err(|e| anyhow::anyhow!("Mutex error: {}", e))?;

    // Check if we already started it
    if process_guard.is_some() {
        // Just verify surfpool is running
        info!("⏳ Checking if surfpool is ready...");
        match rpc_client.get_latest_blockhash().await {
            Ok(_) => {
                info!("✅ Surfpool is ready");
                return Ok(());
            }
            Err(_e) => {
                return Err(anyhow::anyhow!(
                    "Previously started surfpool is not accessible"
                ));
            }
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
        .map_err(|e| anyhow::anyhow!("Failed to start surfpool: {}. Is surfpool installed?", e))?;

    let pid = output.id();
    info!("✅ Started surfpool with PID: {}", pid);
    *process_guard = Some(pid);

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

    Err(anyhow::anyhow!(
        "Surfpool did not become ready after {} attempts",
        max_attempts
    ))
}

/// Cleanup function to kill surfpool after tests
async fn cleanup_surfpool() -> Result<()> {
    let process_ref = SURFPOOL_PROCESS
        .get()
        .ok_or_else(|| anyhow::anyhow!("Process reference not initialized"))?;
    let mut process_guard = process_ref
        .lock()
        .map_err(|e| anyhow::anyhow!("Mutex error: {}", e))?;

    if let Some(pid) = *process_guard {
        info!("🧹 Cleaning up surfpool with PID: {}...", pid);

        // Kill the process
        #[cfg(unix)]
        {
            use std::process;
            let _ = process::Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .output();
        }

        // Reset the process reference
        *process_guard = None;
        info!("✅ Surfpool cleanup completed");
    }

    Ok(())
}

/// Common function to set up a wallet with SOL and USDC
async fn setup_wallet(
    pubkey: &solana_sdk::pubkey::Pubkey,
    surfpool_client: &SurfpoolClient,
) -> Result<(f64, f64)> {
    // Airdrop 5 SOL to the account
    info!("🔄 Airdropping 5 SOL to account...");
    surfpool_client
        .set_account(&pubkey.to_string(), 5_000_000_000)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to airdrop SOL: {e}"))?;

    // Verify SOL balance
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    let balance = rpc_client.get_balance(pubkey).await?;
    let sol_balance = balance as f64 / 1_000_000_000.0;
    info!("✅ Account balance: {sol_balance} SOL");

    // Set up USDC token account with 100 USDC
    let usdc_mint = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let usdc_ata = spl_associated_token_account::get_associated_token_address(pubkey, &usdc_mint);

    info!("🔄 Setting up USDC token account with 100 USDC...");
    surfpool_client
        .set_token_account(&pubkey.to_string(), &usdc_mint.to_string(), 100_000_000)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set up USDC token account: {e}"))?;

    // Verify USDC balance
    let usdc_balance = rpc_client.get_token_account_balance(&usdc_ata).await?;
    let usdc_amount = &usdc_balance.ui_amount_string;
    info!("✅ USDC balance: {usdc_amount}");

    let usdc_balance_f64 = usdc_balance.ui_amount.unwrap_or(0.0);

    Ok((sol_balance, usdc_balance_f64))
}

/// Common function to execute a swap using the planner and LLM
async fn execute_swap_with_planner(
    prompt: &str,
    pubkey: &solana_sdk::pubkey::Pubkey,
    initial_sol_balance: f64,
    initial_usdc_balance: f64,
) -> Result<String> {
    info!("📝 Prompt: {}", prompt);

    // Create the YML wallet info
    let wallet_info = json!({
        "pubkey": pubkey.to_string(),
        "lamports": (initial_sol_balance * 1_000_000_000.0) as u64,
        "total_value_usd": initial_sol_balance * 150.0 + initial_usdc_balance, // Assuming SOL = $150
    });

    // Create a structured YML prompt
    let yml_prompt = format!(
        r#"subject_wallet_info:
  - pubkey: "{}"
    lamports: {} # {} SOL
    total_value_usd: {}

steps:
  prompt: "{prompt}"
    context: "Executing a swap using Jupiter"
"#,
        pubkey,
        (initial_sol_balance * 1_000_000_000.0) as u64,
        initial_sol_balance,
        wallet_info["total_value_usd"]
    );

    info!("📝 YML Prompt:\n{}", yml_prompt);

    // Set up the context resolver
    let context_resolver = ContextResolver::default();

    // Create a planner with GLM client
    let planner = Planner::new_with_glm(context_resolver)?;

    // Generate the flow using the planner
    let yml_flow = planner.refine_and_plan(prompt, &pubkey.to_string()).await?;
    info!("✅ Generated flow: {}", yml_flow.flow_id);

    // Create a wallet context for the executor
    let mut wallet_context = WalletContext::new(pubkey.to_string());
    wallet_context.sol_balance = (initial_sol_balance * 1_000_000_000.0) as u64;

    // Add USDC balance to wallet context
    let usdc_balance = reev_types::benchmark::TokenBalance::new(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        (initial_usdc_balance * 1_000_000.0) as u64, // USDC has 6 decimals
    )
    .with_decimals(6);
    wallet_context.add_token_balance(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        usdc_balance,
    );
    wallet_context.calculate_total_value();

    // Execute the flow using the Executor
    let executor = Executor::new()?;
    let result = executor.execute_flow(&yml_flow, &wallet_context).await?;

    // Extract the transaction signature from the result
    let signature = result
        .step_results
        .iter()
        .find_map(|step| {
            if let Some(transaction_sig) = step.output.get("transaction_signature") {
                transaction_sig.as_str()
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("No transaction signature in result"))?;

    info!("✅ Transaction completed with signature: {}", signature);
    Ok(signature.to_string())
}

/// Test end-to-end swap flow with prompt "swap 1 SOL for USDC"
#[tokio::test]
#[ignore] // Ignore by default since it requires surfpool to be running
async fn test_swap_1_sol_for_usdc() -> Result<()> {
    // Initialize tracing if not already initialized
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    info!("✅ ZAI_API_KEY is configured");

    // Check if surfpool is running
    ensure_surfpool_running().await?;

    // Load the default Solana keypair from ~/.config/solana/id.json
    let keypair = get_keypair()
        .map_err(|e| anyhow::anyhow!("Failed to load keypair from default location: {e}"))?;

    let pubkey = keypair.pubkey();
    info!("✅ Loaded default keypair: {pubkey}");

    // Initialize surfpool client
    let surfpool_client = SurfpoolClient::new("http://localhost:8899");

    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        setup_wallet(&pubkey, &surfpool_client).await?;

    // Execute the swap using the planner and LLM
    let _signature = execute_swap_with_planner(
        "swap 1 SOL for USDC",
        &pubkey,
        initial_sol_balance,
        initial_usdc_balance,
    )
    .await?;

    // TODO: Verify the final balances
    // This would involve checking the final SOL and USDC balances and ensuring
    // that approximately 1 SOL was exchanged for the appropriate amount of USDC

    info!("✅ Swap test completed successfully!");
    Ok(())
}

/// Test end-to-end swap flow with prompt "sell all SOL for USDC"
#[tokio::test]
#[ignore] // Ignore by default since it requires surfpool to be running
async fn test_sell_all_sol_for_usdc() -> Result<()> {
    // Initialize tracing if not already initialized
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    info!("✅ ZAI_API_KEY is configured");

    // Start surfpool
    ensure_surfpool_running().await?;

    // Set up cleanup to kill surfpool after test completes
    // Note: We'll rely on the process being killed when the test exits

    // Load the default Solana keypair from ~/.config/solana/id.json
    let keypair = get_keypair()
        .map_err(|e| anyhow::anyhow!("Failed to load keypair from default location: {e}"))?;

    let pubkey = keypair.pubkey();
    info!("✅ Loaded default keypair: {pubkey}");

    // Initialize surfpool client
    let surfpool_client = SurfpoolClient::new("http://localhost:8899");

    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        setup_wallet(&pubkey, &surfpool_client).await?;

    // Execute the swap using the planner and LLM
    let _signature = execute_swap_with_planner(
        "sell all SOL for USDC",
        &pubkey,
        initial_sol_balance,
        initial_usdc_balance,
    )
    .await?;

    // TODO: Verify the final balances
    // This would involve checking the final SOL and USDC balances and ensuring
    // that all SOL was exchanged for the appropriate amount of USDC

    info!("✅ Sell all SOL test completed successfully!");
    Ok(())
}
