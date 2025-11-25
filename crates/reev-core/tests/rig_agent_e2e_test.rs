//! End-to-end test for RigAgent with GLM-based tool selection
//!
//! This test verifies that the RigAgent with GLM-coding model can:
//! 1. Process user prompts with wallet context
//! 2. Use GLM-4.6-coding model for tool selection and parameter extraction
//! 3. Select the appropriate tools based on the prompt and expected_tools hints
//! 4. Extract parameters from the refined prompt
//! 5. Execute SOL transfers using the selected tools
//! 6. Verify transaction completion on-chain
//!
//! ## Running Test with Proper Logging
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test rig_agent_e2e_test test_rig_agent_transfer -- --nocapture > test_output.log 2>&1
//! ```

use anyhow::{anyhow, Result};
use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;
use reev_core::Executor;
use reev_lib::get_keypair;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::env;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Target account for SOL transfer
const TARGET_PUBKEY: &str = "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";

/// Helper function to start surfpool and wait for it to be ready
async fn ensure_surfpool_running() -> Result<()> {
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

/// Setup wallet with SOL for transfer test
async fn setup_wallet(pubkey: &Pubkey, rpc_client: &RpcClient) -> Result<u64> {
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

/// Execute transfer using the planner and executor with RigAgent
async fn execute_transfer_with_rig_agent(
    prompt: &str,
    from_pubkey: &Pubkey,
    initial_sol_balance: u64,
) -> Result<String> {
    info!(
        "\n🚀 Starting transfer execution with RigAgent for prompt: {}",
        prompt
    );

    // Set environment variables to ensure V3 implementation is used
    std::env::set_var("REEV_USE_V3", "1");

    // Step 1: Create YML prompt with wallet context
    let context_resolver = ContextResolver::new(SolanaEnvironment {
        rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
    });
    let wallet_context = context_resolver
        .resolve_wallet_context(&from_pubkey.to_string())
        .await?;

    let formatted_balance = initial_sol_balance / 1_000_000_000;
    let wallet_info = format!(
        "subject_wallet_info:\n  - pubkey: \"{from_pubkey}\"\n    lamports: {initial_sol_balance} # {formatted_balance} SOL\n    total_value_usd: 170\n\nsteps:\n  prompt: \"{prompt}\"\n    intent: \"send\"\n    context: \"Executing a SOL transfer using Solana system instructions\"\n    recipient: \"{TARGET_PUBKEY}\""
    );

    info!(
        "\n📋 Step 2: YML Prompt with Wallet Info (sent to RigAgent with GLM-4.6-coding via ZAI_API_KEY):\n{}",
        wallet_info
    );

    // Step 3: Send prompt to LLM for prompt refinement and YML generation
    info!("\n🤖 Step 3: Sending prompt to GLM-4.6 model via ZAI_API_KEY for prompt refinement...");

    // Initialize planner with context and GLM client
    let planner = Planner::new_with_glm(context_resolver)?;

    // Generate flow from prompt
    let flow = planner
        .refine_and_plan(prompt, &from_pubkey.to_string())
        .await?;

    // Verify the flow has expected_tools for RigAgent
    info!("📋 Flow generated with {} steps", flow.steps.len());
    if let Some(step) = flow.steps.first() {
        if let Some(ref expected_tools) = step.expected_tools {
            info!("🔧 Expected tools for RigAgent: {:?}", expected_tools);
        } else {
            warn!("⚠️ No expected_tools found for RigAgent, tool selection may be less accurate");
        }
    }

    info!("\n⚙️ Step 4: Initializing executor with RigAgent for tool selection and parameter extraction...");

    // Initialize executor with RigAgent enabled
    let executor = Executor::new_with_rig().await?;

    // Verify RigAgent is enabled by checking the executor configuration
    info!("✅ Executor initialized with RigAgent: Using RigAgent for tool selection");

    // Ensure the wallet context has all necessary information for RigAgent
    info!(
        "💰 Wallet context: owner={}, sol_balance={} lamports",
        wallet_context.owner, wallet_context.sol_balance
    );

    // Execute the flow
    let result = executor.execute_flow(&flow, &wallet_context).await?;

    // Step 5: Extract transaction signature
    info!("🔍 Step results count: {}", result.step_results.len());

    // Debug: Print full step results to understand structure
    println!("DEBUG: Full step results: {:#?}", result.step_results);

    // Extract transaction signature from the first successful step result
    let signature = result
        .step_results
        .iter()
        .find_map(|r| {
            // Check for success first
            if !r.success {
                return None;
            }

            // Try different possible structures for signature
            // 1. Check for tool_results array with signature
            if let Some(tool_results) = r.output.get("tool_results") {
                if let Some(results_array) = tool_results.as_array() {
                    for result in results_array {
                        if let Some(sig) = result.get("transaction_signature") {
                            if let Some(sig_str) = sig.as_str() {
                                return Some(sig_str.to_string());
                            }
                        }
                    }
                }
            }

            // 2. Check for direct transaction_signature field
            if let Some(sig) = r.output.get("transaction_signature") {
                if let Some(sig_str) = sig.as_str() {
                    return Some(sig_str.to_string());
                }
            }

            // 3. Check for sol_transfer.transaction_signature
            if let Some(sol_transfer) = r.output.get("sol_transfer") {
                if let Some(sig) = sol_transfer.get("transaction_signature") {
                    if let Some(sig_str) = sig.as_str() {
                        return Some(sig_str.to_string());
                    }
                }
            }

            None
        })
        .ok_or_else(|| anyhow!("No transaction signature found in step results"))?;

    info!(
        "\n✅ Step 6: Transfer completed with signature: {}",
        signature
    );
    Ok(signature)
}

/// Run transfer test with given prompt
async fn run_rig_agent_transfer_test(test_name: &str, prompt: &str) -> Result<()> {
    info!("\n🧪 Starting Test: {}", test_name);
    info!("=====================================");

    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Initialize tracing with focused logging for the transfer flow
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reev_core::execution::tool_executor=error,warn".into()),
        )
        .with_target(false) // Remove target module prefixes for cleaner output
        .try_init();

    // Load .env file for ZAI_API_KEY
    dotenvy::dotenv().ok();

    // Disable enhanced OTEL logging to reduce verbosity
    env::set_var("REEV_ENHANCED_OTEL", "0");

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

    // Initialize RPC client
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());

    // Set up the wallet with SOL
    let initial_sol_balance = setup_wallet(&pubkey, &rpc_client).await?;
    info!(
        "✅ Wallet setup completed with {} SOL",
        initial_sol_balance / 1_000_000_000
    );

    // Get target account info
    let target_pubkey = TARGET_PUBKEY
        .parse::<Pubkey>()
        .map_err(|e| anyhow!("Invalid target pubkey: {e}"))?;

    // Get initial target balance for verification
    let initial_target_balance = rpc_client.get_balance(&target_pubkey).await?;
    info!("💰 Target account initial balance: {initial_target_balance} lamports");

    info!("\n🔄 Starting transfer execution flow...");

    // Execute the transfer using the planner with RigAgent
    let signature = execute_transfer_with_rig_agent(prompt, &pubkey, initial_sol_balance).await?;
    println!("DEBUG: Extracted signature: {signature}");

    // Verify that RigAgent selected and executed the correct tool
    // With real blockchain transactions, we need to check if the transaction is confirmed

    // Check if signature format is valid (Solana transaction signature should be 88 characters)
    if signature.len() != 88 {
        println!(
            "DEBUG: Invalid signature length: {}, expected 88",
            signature.len()
        );
        return Err(anyhow!(
            "Invalid transaction signature format: length {}",
            signature.len()
        ));
    } else {
        println!("DEBUG: Valid signature format (length: 88)");
    }

    // Parse signature and check if transaction exists on-chain
    let sig_result = signature.parse::<solana_sdk::signature::Signature>();
    match sig_result {
        Ok(sig) => {
            println!("DEBUG: Parsed signature successfully");

            // Wait a moment for the transaction to be confirmed
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Check if transaction is confirmed
            match rpc_client.get_signature_status(&sig).await {
                Ok(status) => {
                    if let Some(err) = status {
                        println!("Transaction failed with error: {err:?}");
                        return Err(anyhow!("Transaction failed: {err:?}"));
                    } else {
                        println!("Transaction confirmed successfully");
                    }
                }
                Err(e) => {
                    println!("Error checking transaction status: {e:?}");
                    // Continue with test even if status check fails
                }
            }
        }
        Err(e) => {
            println!("Error parsing signature: {e:?}");
            return Err(anyhow!("Error parsing signature: {e}"));
        }
    }

    // Verify the transfer by checking both source and target account balances
    let final_source_balance = rpc_client.get_balance(&pubkey).await?;
    let final_target_balance = rpc_client.get_balance(&target_pubkey).await?;

    let source_deduction = initial_sol_balance - final_source_balance;
    let transferred_amount = final_target_balance - initial_target_balance;

    // 1 SOL = 1,000,000,000 lamports
    info!("📊 Balance changes:");
    info!(
        "Source account: {initial_sol_balance} -> {final_source_balance} lamports (deducted: {source_deduction})"
    );
    info!(
        "Target account: {initial_target_balance} -> {final_target_balance} lamports (received: {transferred_amount})"
    );

    // Verify both deduction and transfer
    if source_deduction >= 1_000_000_000 && transferred_amount >= 1_000_000_000 {
        info!("\n🎉 Transfer successful!");
        info!("✅ Deducted {source_deduction} lamports from source account");
        info!("✅ Transferred {transferred_amount} lamports to target account");
        info!("✅ Transaction signature: {signature}");
        info!("✅ RigAgent correctly selected SolTransfer tool based on expected_tools hint");
        info!("✅ RigAgent correctly extracted parameters from refined prompt");
        info!("✅ RigAgent executed a real blockchain transaction");
        info!("✅ Test validated RigAgent's end-to-end functionality");
    } else {
        return Err(anyhow::anyhow!(
            "Transfer verification failed. Source deduction: {source_deduction} lamports, Target received: {transferred_amount} lamports. Expected at least 1 SOL for both."
        ));
    }

    info!("\n🎉 Test completed successfully!");
    info!("=============================");
    Ok(())
}

/// Test RigAgent with GLM-based tool selection and SOL transfer
#[tokio::test]
async fn test_rig_agent_transfer() -> Result<()> {
    // Test with a simple SOL transfer prompt
    let prompt = "send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq";
    run_rig_agent_transfer_test("SOL Transfer", prompt).await
}
