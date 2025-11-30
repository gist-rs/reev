//! End-to-end SOL transfer test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the new QueryHandler
//! to test the complete flow from user prompt to transaction execution.
//!
//! ## Transfer Process Flow
//!
//! 1. User query is passed to QueryHandler
//! 2. QueryHandler resolves wallet context
//! 3. QueryHandler uses LLM to refine the prompt (handling "all" keyword)
//! 4. QueryHandler generates and executes a flow
//! 5. Transaction is performed on-chain
//!
//! For "send all sol" transfers, the QueryHandler's LLM planner will:
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
use tracing::info;

/// Consolidated transfer test that handles both specific amount and "all" keyword cases
/// The QueryHandler's LLM-based planner should handle both scenarios appropriately
#[rstest]
#[case("send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[case("send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_transfer(#[case] prompt: &str) -> Result<()> {
    info!("Testing prompt: {prompt}");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let mut query_handler = QueryHandler::new().await?;

    // Reset wallet balance to ensure we have enough SOL for tests
    runner.reset_wallet_balance().await?;

    // Process the query through QueryHandler's LLM-based pipeline
    // The LLM will handle both specific amounts and "all" keyword cases
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
