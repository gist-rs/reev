//! End-to-end SOL transfer test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the new QueryHandler
//! to test the complete flow from user prompt to transaction execution.
//!
//! ## Transfer Process Flow
//!
//! 1. User query is passed to QueryHandler
//! 2. QueryHandler resolves wallet context
//! 3. QueryHandler refines the prompt (handling "all" keyword)
//! 4. QueryHandler generates and executes a flow
//! 5. Transaction is performed on-chain
//!
//! For "send all sol" transfers, the QueryHandler will:
//! - Detect the "all" keyword in the user query
//! - Determine the actual wallet balance at runtime
//! - Reserve a small amount for transaction fees
//! - Execute the transfer with the calculated amount
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test e2e_transfer -- --nocapture > test_output.log 2>&1
//! ```

mod common;

use anyhow::Result;
use reev_core::QueryHandler;
use rstest::*;
use serial_test::serial;
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::info;

/// Test with specific amount
#[rstest]
#[case("send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_transfer_specific_amount(#[case] prompt: &str) -> Result<()> {
    info!("Testing prompt: {prompt}");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let mut query_handler = QueryHandler::new().await?;

    // Reset wallet balance to ensure we have enough SOL for tests
    runner.reset_wallet_balance().await?;

    // Process the query
    let result = query_handler.process_query(prompt, &runner.pubkey).await?;

    // Verify the query was processed successfully
    assert!(
        result.success,
        "Query processing failed: {:?}",
        result.error_message
    );
    assert!(
        result.transaction_signature.is_some(),
        "No transaction signature returned"
    );

    // Verify transfer was successful
    info!(
        "✅ Transfer completed with signature: {:?}",
        result.transaction_signature
    );

    Ok(())
}

/// Test with "all" keyword - should calculate max transferable amount
#[rstest]
#[case("send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_transfer_all_keyword(#[case] prompt: &str) -> Result<()> {
    info!("Testing prompt: {prompt}");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let mut query_handler = QueryHandler::new().await?;

    // Reset wallet balance to ensure we have enough SOL for tests
    runner.reset_wallet_balance().await?;

    // Get the initial wallet balance directly from blockchain
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    let initial_balance = rpc_client.get_balance(&runner.pubkey).await?;
    let initial_balance_sol = initial_balance as f64 / 1_000_000_000.0;
    info!("Initial wallet balance: {} SOL", initial_balance_sol);

    // Calculate the expected max transferable amount (with 0.001 SOL gas reserve)
    let expected_max_transferable = query_handler
        .calculate_max_transferable(&runner.pubkey, Some(1_000_000))
        .await?;
    let expected_max_transferable_sol = expected_max_transferable as f64 / 1_000_000_000.0;
    info!(
        "Expected max transferable amount: {} SOL",
        expected_max_transferable_sol
    );

    // Process the query with "all" keyword
    let result = query_handler.process_query(prompt, &runner.pubkey).await?;

    // Verify the query was processed successfully
    assert!(
        result.success,
        "Query processing failed: {:?}",
        result.error_message
    );
    assert!(
        result.transaction_signature.is_some(),
        "No transaction signature returned"
    );

    // Verify transfer was successful
    info!(
        "✅ Transfer completed with signature: {:?}",
        result.transaction_signature
    );

    // Verify the final balance directly from blockchain
    let final_balance = rpc_client.get_balance(&runner.pubkey).await?;
    let final_balance_sol = final_balance as f64 / 1_000_000_000.0;
    let transferred_amount = initial_balance_sol - final_balance_sol;
    info!("Amount transferred: {} SOL", transferred_amount);

    // Verify that the transferred amount is close to the expected max transferable amount
    // Allow for a small variance due to transaction fees and rounding
    let variance = 0.01; // 0.01 SOL variance
    assert!(
        (transferred_amount - expected_max_transferable_sol).abs() < variance,
        "Transferred amount {transferred_amount} SOL is not close to expected {expected_max_transferable_sol} SOL"
    );

    Ok(())
}
