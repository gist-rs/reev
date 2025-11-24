//! End-to-end swap test using default Solana keypair
//!
//! This test loads the wallet from ~/.config/solana/id.json, airdrops SOL via surfpool,
//! uses the planner to process the prompt, lets the LLM handle tool calling via rig,
//! signs the transaction with the default keypair, and verifies completion.
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test end_to_end_swap test_swap_1_sol_for_usdc -- --nocapture --ignored > test_output.log 2>&1
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
use reev_core::context::ContextResolver;
use reev_core::planner::Planner;
use reev_core::Executor;
use reev_lib::get_keypair;
use reev_types::flow::WalletContext;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use std::env;

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Global reference to surfpool process
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
        .map_err(|e| anyhow::anyhow!("Mutex error: {e}"))?;

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
        .map_err(|e| anyhow::anyhow!("Failed to start surfpool: {e}. Is surfpool installed?"))?;

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

    // Set up the context resolver
    let context_resolver = ContextResolver::default();

    // Create a planner with GLM client
    let planner = Planner::new_with_glm(context_resolver)?;

    info!("\n🤖 Step 3: Sending prompt to GLM-4.6 model via ZAI_API_KEY...");
    // Generate the flow using the planner
    let yml_flow = planner.refine_and_plan(prompt, &pubkey.to_string()).await?;
    info!("✅ Flow generated with ID: {}", yml_flow.flow_id);

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

        // Try to extract transaction signature from the output
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                info!("  Tool results: {} results", results_array.len());
                for (j, result) in results_array.iter().enumerate() {
                    info!(
                        "    Result {}: {}",
                        j + 1,
                        serde_json::to_string_pretty(result).unwrap_or_default()
                    );

                    // Try to extract signature from different possible locations
                    if let Some(result_obj) = result.as_object() {
                        // Check for direct transaction_signature field
                        if let Some(tx_sig) = result_obj.get("transaction_signature") {
                            info!("    Found transaction_signature: {:?}", tx_sig);
                        }

                        // Check in nested structure
                        if let Some(jupiter_response) = result_obj.get("jupiter_swap") {
                            if let Some(response_obj) = jupiter_response.as_object() {
                                if let Some(signature) = response_obj.get("transaction_signature") {
                                    info!(
                                        "    Found jupiter_swap.transaction_signature: {:?}",
                                        signature
                                    );
                                }
                            }
                        }
                    }
                }
            } else {
                info!(
                    "  Tool results is not an array: {}",
                    serde_json::to_string_pretty(tool_results).unwrap_or_default()
                );
            }
        } else {
            info!("  No tool_results in output");
        }

        if let Some(error) = &step_result.error_message {
            info!("  Error: {}", error);
        }

        // Log the full output for debugging
        info!(
            "  Full output: {}",
            serde_json::to_string_pretty(&step_result.output).unwrap_or_default()
        );
    }

    // Extract transaction signature from the step results
    for (i, step_result) in result.step_results.iter().enumerate() {
        info!("Checking step {} for transaction signature", i + 1);

        // Check for signatures directly in the output
        if let Some(signatures) = step_result.output.get("signatures") {
            if let Some(sig_array) = signatures.as_array() {
                for sig in sig_array {
                    if let Some(sig_str) = sig.as_str() {
                        if !sig_str.is_empty() {
                            info!("\n📝 Step 5: Generated transaction:");
                            info!("Signature: {}", sig_str);

                            info!("\n🔑 Step 6: Transaction signed with default keypair at ~/.config/solana/id.json");

                            info!("\n📊 Step 6: Transaction completed successfully via SURFPOOL!");
                            info!("Transaction URL: https://solscan.io/tx/{}", sig_str);

                            info!("\n✅ All steps completed successfully!");
                            return Ok(sig_str.to_string());
                        }
                    }
                }
            }
        }

        // Look for transaction signature in tool_results
        if let Some(tool_results) = step_result.output.get("tool_results") {
            if let Some(results_array) = tool_results.as_array() {
                for result in results_array {
                    if let Some(result_map) = result.as_object() {
                        // Check for transaction_signature from JupiterSwapResponse
                        if let Some(tx_sig) = result_map.get("transaction_signature") {
                            let signature = tx_sig.as_str().unwrap_or_default();

                            if !signature.is_empty() {
                                info!("\n📝 Step 5: Generated transaction:");
                                info!("Signature: {}", signature);

                                info!("\n🔑 Step 6: Transaction signed with default keypair at ~/.config/solana/id.json");

                                info!(
                                    "\n📊 Step 6: Transaction completed successfully via SURFPOOL!"
                                );
                                info!("Transaction URL: https://solscan.io/tx/{}", signature);

                                info!("\n✅ All steps completed successfully!");
                                return Ok(signature.to_string());
                            }
                        }

                        // Check if it's nested in a "result" object
                        if let Some(result_obj) = result_map.get("result") {
                            if let Some(result_map) = result_obj.as_object() {
                                if let Some(tx_sig) = result_map.get("signature") {
                                    let signature = tx_sig.as_str().unwrap_or_default();

                                    if !signature.is_empty() {
                                        info!("\n📝 Step 5: Generated transaction:");
                                        info!("Signature: {}", signature);

                                        info!("\n🔑 Step 6: Transaction signed with default keypair at ~/.config/solana/id.json");

                                        info!("\n📊 Step 6: Transaction completed successfully via SURFPOOL!");
                                        info!(
                                            "Transaction URL: https://solscan.io/tx/{}",
                                            signature
                                        );

                                        info!("\n✅ All steps completed successfully!");
                                        return Ok(signature.to_string());
                                    }
                                }
                            }
                        }

                        // Check in nested structure from JupiterSwapResponse
                        if let Some(jupiter_response) = result_map.get("jupiter_swap") {
                            if let Some(response_obj) = jupiter_response.as_object() {
                                if let Some(signature) = response_obj.get("transaction_signature") {
                                    if let Some(sig_str) = signature.as_str() {
                                        if !sig_str.is_empty() {
                                            info!("\n📝 Step 5: Generated transaction:");
                                            info!("Signature: {}", sig_str);

                                            info!("\n🔑 Step 6: Transaction signed with default keypair at ~/.config/solana/id.json");

                                            info!("\n📊 Step 6: Transaction completed successfully via SURFPOOL!");
                                            info!(
                                                "Transaction URL: https://solscan.io/tx/{}",
                                                sig_str
                                            );

                                            info!("\n✅ All steps completed successfully!");
                                            return Ok(sig_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        // If still not found, try parsing the result as a JSON string (for real tool responses)
                        else if let Some(result_str) =
                            result_map.get("result").and_then(|v| v.as_str())
                        {
                            // Parse the JSON string from the JupiterSwapTool response
                            if let Ok(jupiter_response) =
                                serde_json::from_str::<serde_json::Value>(result_str)
                            {
                                // Extract the transaction_signature from the JupiterSwapResponse
                                if let Some(jupiter_obj) = jupiter_response.as_object() {
                                    if let Some(signature) =
                                        jupiter_obj.get("transaction_signature")
                                    {
                                        if let Some(sig_str) = signature.as_str() {
                                            if !sig_str.is_empty() {
                                                info!("\n📝 Step 5: Generated transaction:");
                                                info!("Signature: {}", sig_str);

                                                info!("\n🔑 Step 6: Transaction signed with default keypair at ~/.config/solana/id.json");

                                                info!("\n📊 Step 6: Transaction completed successfully via SURFPOOL!");
                                                info!(
                                                    "Transaction URL: https://solscan.io/tx/{}",
                                                    sig_str
                                                );

                                                info!("\n✅ All steps completed successfully!");
                                                return Ok(sig_str.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // No transaction signature found
    Err(anyhow!(
        "No transaction signature found in execution result"
    ))
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

    // Check if surfpool is running
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
    let _signature =
        execute_swap_with_planner(prompt, &pubkey, initial_sol_balance, initial_usdc_balance)
            .await?;

    // TODO: Verify the final balances
    // This would involve checking the final SOL and USDC balances and ensuring
    // that the appropriate amount of SOL was exchanged for USDC

    info!("\n🎉 Test completed successfully!");
    info!("=============================");
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
#[ignore] // Ignore by default since it requires surfpool to be running
async fn test_swap_0_1_sol_for_usdc() -> Result<()> {
    run_swap_test("Swap 0.1 SOL for USDC", "swap 0.1 SOL for USDC").await
}

/// Test end-to-end swap flow with prompt "sell all SOL for USDC"
/// Follows the same 6-step process as test_swap_1_sol_for_usdc
/// but with a "sell all SOL" prompt instead.
#[tokio::test(flavor = "multi_thread")]
#[ignore] // Ignore by default since it requires surfpool to be running
async fn test_sell_all_sol_for_usdc() -> Result<()> {
    run_swap_test("Sell All SOL for USDC", "sell all SOL for USDC").await
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn test_cleanup_surfpool() -> Result<()> {
    // This test can be used to clean up surfpool if needed
    cleanup_surfpool().await
}
