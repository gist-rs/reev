//! End-to-end swap test using default Solana keypair
//!
//! This test loads the wallet from ~/.config/solana/id.json, airdrops SOL via surfpool,
//! uses the planner to process the prompt, lets the LLM handle tool calling via rig,
//! signs the transaction with the default keypair, and verifies completion.
//!
//! ## Jupiter Transaction Retry Behavior
//!
//! Jupiter transactions are time-sensitive and cannot be simply retried:
//! - Jupiter swap routes are based on current market conditions and liquidity
//! - Solana transactions are tied to specific blockhashes that expire
//! - Proper retry would require getting a fresh quote from Jupiter API with current blockhash
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test end_to_end_swap test_swap_0_1_sol_for_usdc -- --nocapture > test_output.log 2>&1
//! RUST_LOG=info cargo test -p reev-core --test end_to_end_swap test_sell_all_sol_for_usdc -- --nocapture > test_output.log 2>&1
//! ```
//!
//! ## Test Flow (6 Steps)
//!
//! 1. Prompt: "swap 1 SOL for USDC" or "sell all SOL for USDC"
//! 2. Shows log info for YML prompt with wallet info from SURFPOOL sent to GLM-coding
//! 3. Shows log info for swap tool calling from LLM
//! 4. Shows the transaction generated from that tool
//! 5. Signs the transaction with default keypair at ~/.config/solana/id.json
//! 6. Shows transaction completion result from SURFPOOL

use anyhow::{anyhow, Result};
use jup_sdk::surfpool::SurfpoolClient;
use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;
use reev_core::Executor;
use reev_lib::get_keypair;
use reev_lib::server_utils::kill_existing_surfpool;
use tracing::{debug, error, info};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use std::env;

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
// debug is already imported above

/// Global reference to surfpool process
static SURFPOOL_PROCESS: std::sync::OnceLock<std::sync::Arc<Mutex<Option<u32>>>> =
    std::sync::OnceLock::new();

/// Helper function to start surfpool and wait for it to be ready
async fn ensure_surfpool_running() -> Result<()> {
    // First, kill any existing SURFPOOL process to ensure a clean start
    info!("🔄 Restarting SURFPOOL for clean test environment...");
    kill_existing_surfpool(8899).await?;

    // Kill any existing surfpool process to ensure clean state
    info!("🧹 Killing any existing surfpool processes...");
    kill_existing_surfpool(8899).await?;

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

    // Try to start surfpool programmatically
    let process_ref = SURFPOOL_PROCESS.get_or_init(|| Arc::new(Mutex::new(None)));

    // Check if we already started it
    let already_started = {
        let process_guard = process_ref
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex error: {e}"))?;
        process_guard.is_some()
    };

    if already_started {
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
        .map_err(|e| anyhow::anyhow!("Failed to start surfpool: {e}. Is surfpool installed?"))?;

    let pid = output.id();
    info!("✅ Started surfpool with PID: {}", pid);

    // Store the PID in the global reference
    {
        let mut process_guard = process_ref
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex error: {e}"))?;
        *process_guard = Some(pid);
    } // Guard is dropped here

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
        "Surfpool did not become ready after {max_attempts} attempts"
    ))
}

/// Cleanup function to kill surfpool after tests
async fn cleanup_surfpool() -> Result<()> {
    let process_ref = SURFPOOL_PROCESS
        .get()
        .ok_or_else(|| anyhow::anyhow!("Process reference not initialized"))?;
    let mut process_guard = process_ref
        .lock()
        .map_err(|e| anyhow::anyhow!("Mutex error: {e}"))?;

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
    let sol_balance = balance as f64 / 1_000_000_000.0_f64;

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
    info!("\n🚀 Starting swap execution with prompt: \"{}\"", prompt);

    // Step 1: Display the prompt being processed
    println!("\n📋 Step 1: Processing prompt: \"{prompt}\"");
    debug!(
        "DEBUG: Initial wallet state - SOL: {}, USDC: {}",
        initial_sol_balance, initial_usdc_balance
    );

    // Create a structured YML prompt with wallet info
    let yml_prompt = format!(
        r#"subject_wallet_info:
  - pubkey: "{}"
    lamports: {} # {} SOL
    total_value_usd: {}

steps:
  prompt: "{}"
    context: "Executing a swap using Jupiter"
"#,
        pubkey,
        (initial_sol_balance * 1_000_000_000.0) as u64,
        initial_sol_balance,
        initial_sol_balance * 150.0 + initial_usdc_balance, // Assuming SOL = $150
        prompt
    );

    // Step 2: Show YML prompt with wallet info that will be sent to GLM-coding
    println!("\n📋 Step 2: YML Prompt with Wallet Info (sent to GLM-coding via ZAI_API_KEY):");
    println!("{yml_prompt}");

    // Set up the context resolver with explicit RPC URL like the transfer test
    let context_resolver = ContextResolver::new(SolanaEnvironment {
        rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
    });

    // Create a planner with GLM client
    let planner = Planner::new_with_glm(context_resolver.clone())?;

    info!("\n🤖 Step 3: Sending prompt to GLM-4.6 model via ZAI_API_KEY...");
    // Generate the flow using the planner
    let yml_flow = planner.refine_and_plan(prompt, &pubkey.to_string()).await?;
    info!("✅ Flow generated with ID: {}", yml_flow.flow_id);

    // Get the wallet context from the resolver, similar to transfer test
    let wallet_context = context_resolver
        .resolve_wallet_context(&pubkey.to_string())
        .await?;

    debug!(
        "DEBUG: Created wallet context with SOL: {} lamports, USDC: {} (raw)",
        wallet_context.sol_balance,
        wallet_context
            .token_balances
            .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            .map(|t| t.balance)
            .unwrap_or(0)
    );

    info!("\n⚙️ Step 4: Executing swap tool call from LLM...");

    // Log the wallet context being passed to the executor
    info!("Wallet context for executor:");
    info!("  Owner: {}", wallet_context.owner);
    info!("  SOL balance: {} lamports", wallet_context.sol_balance);
    info!("  Token balances: {}", wallet_context.token_balances.len());
    for (mint, balance) in &wallet_context.token_balances {
        info!("    {}: {} tokens", mint, balance.balance);
    }

    // Execute flow using the Executor
    let executor = Executor::new()?;

    info!("About to call executor.execute_flow");
    let result = executor.execute_flow(&yml_flow, &wallet_context).await?;
    info!("executor.execute_flow completed successfully");

    // Log the result structure
    info!("Flow execution result:");
    info!("  Flow ID: {}", result.flow_id);
    info!("  Success: {}", result.success);
    info!("  Step results count: {}", result.step_results.len());

    // Log detailed step results
    for (i, step_result) in result.step_results.iter().enumerate() {
        info!("Step {} result:", i + 1);
        info!("  Step ID: {}", step_result.step_id);
        info!("  Success: {}", step_result.success);
        info!("  Tool calls: {:?}", step_result.tool_calls);

        // Log the full output for debugging
        info!(
            "  Full output: {}",
            serde_json::to_string_pretty(&step_result.output).unwrap_or_default()
        );

        // Debug: Print the entire output structure
        debug!(
            "  Output keys: {:?}",
            step_result
                .output
                .as_object()
                .map(|o| o.keys().collect::<Vec<_>>())
                .unwrap_or_default()
        );

        // Debug: Check if jupiter_swap field exists
        if step_result.output.get("jupiter_swap").is_some() {
            debug!("  jupiter_swap field found in output");
        } else {
            debug!("  jupiter_swap field NOT found in output");
        }
    }

    // Extract transaction signature from the step results, matching format from the executor
    // Based on executor's process_transaction_with_instructions_step_result function
    let signature = result
        .step_results
        .iter()
        .find_map(|r| {
            // Look for signature in output.jupiter_swap.transaction_signature (current format)
            if let Some(jupiter_swap) = r.output.get("jupiter_swap") {
                if let Some(sig) = jupiter_swap.get("transaction_signature") {
                    if let Some(sig_str) = sig.as_str() {
                        return Some(sig_str.to_string());
                    }
                }
            }
            // Also check for transaction_signature directly in output (for mock transactions)
            if let Some(sig) = r.output.get("transaction_signature") {
                if let Some(sig_str) = sig.as_str() {
                    return Some(sig_str.to_string());
                }
            }
            // Also check tool calls array for signatures
            for call in &r.tool_calls {
                if call.contains("transaction_signature") {
                    // Extract signature from JSON string
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(call) {
                        if let Some(sig) = json.get("transaction_signature") {
                            if let Some(sig_str) = sig.as_str() {
                                return Some(sig_str.to_string());
                            }
                        }
                    }
                }
            }
            None
        })
        .ok_or_else(|| anyhow!("No transaction signature in result"))?;

    info!("\n✅ Step 6: Swap completed with signature: {}", signature);

    // Check if the execution was successful
    if result.step_results.iter().any(|r| !r.success) {
        error!("❌ Some steps in the flow failed");
        return Err(anyhow::anyhow!("Some steps in the flow failed"));
    }

    Ok(signature)
}

/// Common test function that executes a swap prompt
async fn run_swap_test(test_name: &str, prompt: &str) -> Result<()> {
    info!("\n🧪 Starting Test: {}", test_name);
    info!("=====================================");

    // Initialize tracing with focused logging for the swap flow
    // Filter to show only relevant logs: planner, executor, and tool execution
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "reev_core::planner=info,reev_core::executor=info,jup_sdk=info,warn".into()
            }),
        )
        .with_target(false) // Remove target module prefixes for cleaner output
        .try_init();

    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Check for ZAI_API_KEY
    let _zai_api_key = env::var("ZAI_API_KEY").map_err(|_| {
        anyhow::anyhow!("ZAI_API_KEY environment variable not set. Please set it in .env file.")
    })?;

    info!("✅ ZAI_API_KEY is configured");

    // Restart SURFPOOL for a clean test environment
    info!("🔄 Restarting SURFPOOL for clean test environment...");
    kill_existing_surfpool(8899).await?;

    // Give SURFPOOL time to restart
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;
    info!("✅ SURFPOOL is running and ready");

    // Load the default Solana keypair from ~/.config/solana/id.json
    let keypair = get_keypair()
        .map_err(|e| anyhow::anyhow!("Failed to load keypair from default location: {e}"))?;

    let pubkey = keypair.pubkey();
    info!("✅ Loaded default keypair: {pubkey}");
    info!("🔑 Using keypair from ~/.config/solana/id.json");

    // Initialize surfpool client
    let surfpool_client = SurfpoolClient::new("http://localhost:8899");

    info!("\n💰 Setting up test wallet with SOL and USDC...");
    // Set up the wallet with SOL and USDC
    let (initial_sol_balance, initial_usdc_balance) =
        setup_wallet(&pubkey, &surfpool_client).await?;
    println!(
        "✅ Wallet setup completed with {initial_sol_balance} SOL and {initial_usdc_balance} USDC"
    );

    info!("\n🔄 Starting swap execution flow...");
    // Execute the swap using the planner and LLM
    // Note: We don't retry Jupiter transactions here because:
    // 1. Jupiter transactions have time-sensitive routes based on current market conditions
    // 2. Solana transactions are tied to specific blockhashes that expire
    // 3. Proper retry would require getting a fresh quote from Jupiter API with current blockhash
    let signature =
        execute_swap_with_planner(prompt, &pubkey, initial_sol_balance, initial_usdc_balance)
            .await?;

    // Initialize RPC client
    let client =
        solana_client::nonblocking::rpc_client::RpcClient::new("http://localhost:8899".to_string());

    // Check transaction status
    match client
        .get_signature_status_with_commitment(
            &signature.parse()?,
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        )
        .await?
    {
        Some(status) => {
            if let Err(err) = status {
                error!("❌ Transaction failed on-chain: {:?}", err);

                return Err(anyhow::anyhow!("Transaction failed on-chain: {err:?}"));
            }
            info!("✅ Transaction confirmed successfully on-chain");
        }
        None => {
            error!("❌ Transaction not found on-chain");
            return Err(anyhow::anyhow!("Transaction not found on-chain"));
        }
    }

    // Verify final balances to ensure swap actually happened
    info!("\n🔍 Verifying final wallet balances...");
    let final_balance = client.get_balance(&pubkey).await?;
    let final_sol_balance = final_balance as f64 / 1_000_000_000.0;

    info!("Final SOL balance: {}", final_sol_balance);
    info!("Initial SOL balance: {}", initial_sol_balance);

    // Calculate expected SOL balance based on the prompt
    // Extract the amount to swap from the prompt
    let swap_amount = if prompt.contains("sell all") {
        // Reserve 0.05 SOL for gas fees
        initial_sol_balance - 0.05
    } else if let Some(amount_str) = prompt.split_whitespace().nth(1) {
        // Try to parse the amount (e.g., "0.1" in "swap 0.1 SOL")
        amount_str.parse::<f64>().unwrap_or(0.1)
    } else {
        0.1 // Default to 0.1 SOL
    };

    let expected_sol_balance = initial_sol_balance - swap_amount;
    let balance_diff = (final_sol_balance - expected_sol_balance).abs();

    // Increase tolerance to account for gas fees and slippage
    if balance_diff > 0.06 {
        error!("❌ Final SOL balance doesn't match expected swap amount");
        error!(
            "Expected: {}, Got: {}, Difference: {}",
            expected_sol_balance, final_sol_balance, balance_diff
        );
        return Err(anyhow::anyhow!(
            "Final balance doesn't match expected swap amount"
        ));
    }

    info!("✅ Final SOL balance matches expected swap amount");

    // TODO: Verify the final balances
    // This would involve checking the final SOL and USDC balances and ensuring
    // that the appropriate amount of SOL was exchanged for USDC

    info!("\n🎉 Test completed successfully!");
    info!("=============================");
    info!("Final transaction signature: {}", signature);
    Ok(())
}

/// Test end-to-end swap flow with prompt "swap 1 SOL for USDC"
///
/// This test follows the 6-step process:
/// 1. Prompt: "swap 1 SOL for USDC"
/// 2. Shows log info for YML prompt with wallet info from SURFPOOL sent to GLM-coding
/// 3. Shows log info for swap tool calling via rig framework from LLM
/// 4. Shows the transaction generated from that tool
/// 5. Signs the transaction with default keypair at ~/.config/solana/id.json
/// 6. Shows transaction completion result from SURFPOOL
#[tokio::test(flavor = "multi_thread")]
async fn test_swap_0_1_sol_for_usdc() -> Result<()> {
    run_swap_test("Swap 0.1 SOL for USDC", "swap 0.1 SOL for USDC").await
}

/// Test end-to-end swap flow with prompt "sell all SOL for USDC"
/// Follows the same 6-step process as test_swap_1_sol_for_usdc
/// but with a "sell all SOL" prompt instead.
#[tokio::test(flavor = "multi_thread")]
async fn test_sell_all_sol_for_usdc() -> Result<()> {
    run_swap_test("Sell All SOL for USDC", "sell all SOL for USDC").await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_cleanup_surfpool() -> Result<()> {
    // This test can be used to clean up surfpool if needed
    // First ensure surfpool is running to initialize the static variable
    ensure_surfpool_running().await?;
    // Then clean it up
    cleanup_surfpool().await
}
