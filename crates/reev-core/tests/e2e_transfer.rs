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

use anyhow::Result;
use common::{
    ensure_surfpool_running, get_test_keypair, init_tracing, parse_pubkey,
    setup_wallet_for_transfer, TARGET_PUBKEY,
};
use jup_sdk::surfpool::SurfpoolClient;
use reev_core::utils::yml_utils::create_subject_wallet_info_yml;
use reev_core::utils::{execute_six_step_flow, extract_transaction_signature};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::collections::HashMap;
use std::env;
use tracing::info;

/// Execute transfer using standardized 6-step flow
async fn execute_transfer_with_rig_agent(
    prompt: &str,
    from_pubkey: &Pubkey,
    initial_sol_balance: u64,
) -> Result<String> {
    info!("\n🚀 Starting transfer execution with prompt: {}", prompt);

    // Create token balances map (empty for transfer test)
    let token_balances = HashMap::new();

    // Create YML prompt using standardized utility
    let wallet_info = create_subject_wallet_info_yml(
        from_pubkey,
        initial_sol_balance,
        None,
        170.0, // Default USD value
    );
    info!(
        "\n📋 Step 2: YML Prompt with Wallet Info (sent to GLM-coding via ZAI_API_KEY):\n{}",
        wallet_info
    );

    // Execute standardized 6-step flow
    let (result, _signature) = execute_six_step_flow(
        prompt,
        from_pubkey,
        initial_sol_balance,
        Some(token_balances),
        170.0,
    )
    .await?;

    // Extract transaction signature using standardized utility
    let signature = extract_transaction_signature(&result)?;

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

    // Set up the wallet with SOL and USDC
    let surfpool_client = SurfpoolClient::new("http://localhost:8899");
    let (initial_sol_balance, initial_usdc_balance) =
        setup_wallet_for_transfer(&pubkey, &surfpool_client).await?;
    info!(
        "✅ Wallet setup completed with {} SOL and {} USDC",
        initial_sol_balance / 1_000_000_000.0,
        initial_usdc_balance
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

    // Execute the transfer using standardized utilities
    let signature =
        execute_transfer_with_rig_agent(prompt, &pubkey, initial_sol_balance as u64).await?;

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
