//! End-to-end SOL transfer test using default Solana keypair
//!
//! This test loads the wallet from ~/.config/solana/id.json, uses the planner to process
//! the transfer prompt, lets the LLM handle tool calling via rig, signs the transaction
//! with the default keypair, and verifies completion.
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test e2e_transfer test_send_1_sol_to_target -- --nocapture > test_output.log 2>&1
//! ```
//!
//! ## Test Flow (6 Steps)
//!
//! 1. Prompt: "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
//! 2. Shows log info for YML prompt with wallet info from SURFPOOL sent to GLM-coding
//! 3. Shows log info for transfer tool calling from LLM
//! 4. Shows the transaction generated from that tool
//! 5. Signs the transaction with default keypair at ~/.config/solana/id.json
//! 6. Shows transaction completion result from SURFPOOL

mod common;

use anyhow::{anyhow, Result};
use common::{
    ensure_surfpool_running, get_test_keypair, init_tracing, parse_pubkey, setup_wallet,
    TARGET_PUBKEY,
};
use reev_core::context::{ContextResolver, SolanaEnvironment};
use reev_core::planner::Planner;
use reev_core::Executor;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::env;
use tracing::info;

/// Execute transfer using the planner and LLM
async fn execute_transfer_with_planner(
    prompt: &str,
    from_pubkey: &Pubkey,
    initial_sol_balance: u64,
) -> Result<String> {
    info!("\n🚀 Starting transfer execution with prompt: {}", prompt);

    // Step 2: Create YML prompt with wallet context
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
        "\n📋 Step 2: YML Prompt with Wallet Info (sent to GLM-coding via ZAI_API_KEY):\n{}",
        wallet_info
    );

    // Step 3: Send prompt to LLM
    info!("\n🤖 Step 3: Sending prompt to GLM-4.6 model via ZAI_API_KEY...");

    // Initialize planner with context and GLM client
    let planner = Planner::new_with_glm(context_resolver)?;

    // Generate flow from prompt
    let flow = planner
        .refine_and_plan(prompt, &from_pubkey.to_string())
        .await?;
    info!("\n⚙️ Step 4: Executing transfer tool call from LLM...");

    // Initialize executor with RigAgent
    let executor = Executor::new_with_rig().await?;

    // Execute the flow
    let result = executor.execute_flow(&flow, &wallet_context).await?;

    // Step 5: Extract transaction signature
    // Find transaction signature in step results
    let signature = result
        .step_results
        .iter()
        .find_map(|r| {
            // Check for signature in tool_results array (RigAgent format)
            if let Some(tool_results) = r.output.get("tool_results") {
                if let Some(results_array) = tool_results.as_array() {
                    for result in results_array {
                        // Check for transaction_signature directly in the tool result
                        if let Some(sig) = result.get("transaction_signature") {
                            if let Some(sig_str) = sig.as_str() {
                                return Some(sig_str.to_string());
                            }
                        }
                        // Also check under sol_transfer if present
                        if let Some(sol_transfer) = result.get("sol_transfer") {
                            if let Some(sig) = sol_transfer.get("transaction_signature") {
                                if let Some(sig_str) = sig.as_str() {
                                    return Some(sig_str.to_string());
                                }
                            }
                        }
                    }
                }
            }

            // Look for signature in output.sol_transfer.transaction_signature
            if let Some(sol_transfer) = r.output.get("sol_transfer") {
                if let Some(sig) = sol_transfer.get("transaction_signature") {
                    if let Some(sig_str) = sig.as_str() {
                        return Some(sig_str.to_string());
                    }
                }
            }
            // Also check for transaction_signature directly in output
            if let Some(sig) = r.output.get("transaction_signature") {
                if let Some(sig_str) = sig.as_str() {
                    return Some(sig_str.to_string());
                }
            }
            // Also check tool calls array
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

    info!(
        "\n✅ Step 6: Transfer completed with signature: {}",
        signature
    );
    Ok(signature)
}

/// Run transfer test with given prompt
async fn run_transfer_test(test_name: &str, prompt: &str) -> Result<()> {
    info!("\n🧪 Starting Test: {}", test_name);
    info!("=====================================");

    // Initialize tracing with focused logging for the transfer flow
    init_tracing();

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
    let keypair = get_test_keypair()?;

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
    let target_pubkey = parse_pubkey(TARGET_PUBKEY)?;

    // Get initial target balance for verification
    let initial_target_balance = rpc_client.get_balance(&target_pubkey).await?;
    info!(
        "💰 Target account initial balance: {} lamports",
        initial_target_balance
    );

    info!("\n🔄 Starting transfer execution flow...");

    // Execute the transfer using the planner and LLM
    let signature = execute_transfer_with_planner(prompt, &pubkey, initial_sol_balance).await?;

    // Verify the transfer by checking target account balance
    let final_target_balance = rpc_client.get_balance(&target_pubkey).await?;
    let transferred_amount = final_target_balance - initial_target_balance;

    // 1 SOL = 1,000,000,000 lamports
    if transferred_amount >= 1_000_000_000 {
        info!("\n🎉 Transfer successful!");
        info!(
            "✅ Transferred {} lamports to target account",
            transferred_amount
        );
        info!("✅ Transaction signature: {}", signature);
    } else {
        return Err(anyhow::anyhow!(
            "Transfer verification failed. Expected at least 1 SOL, got {transferred_amount} lamports"
        ));
    }

    info!("\n🎉 Test completed successfully!");
    info!("=============================");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_send_1_sol_to_target() -> Result<()> {
    run_transfer_test(
        "Send 1 SOL to target account",
        "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
    )
    .await
}
