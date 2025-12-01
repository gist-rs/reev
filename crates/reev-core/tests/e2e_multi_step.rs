//! End-to-end multi-step test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the common test
//! framework to test multi-step operations like swapping and then lending in a single query.
//!
//! ## Multi-step Process Flow
//!
//! 1. User query with multiple operations is passed to QueryHandler
//! 2. QueryHandler resolves wallet context
//! 3. QueryHandler's LLM refines prompt (handling multiple operations)
//! 4. QueryHandler generates and executes a multi-step flow
//! 5. Multiple transactions are performed on-chain in sequence
//!
//! For multi-step operations, QueryHandler's LLM-based planner will:
//! - Detect multiple operations in user query
//! - Plan a sequence of operations in the correct order
//! - Execute each step with appropriate parameters
//! - Handle dependencies between operations (e.g., using output of one step in the next)
//!
//! ## Jupiter Transaction Behavior
//!
//! Jupiter transactions can sometimes fail with the 0xffff error due to market conditions.
//! This is a transient error that happens randomly due to:
//! - Jupiter swap routes being based on current market conditions and liquidity
//! - Solana transactions being tied to specific blockhashes that expire
//! - Market volatility affecting slippage and liquidity
//!
//! When the 0xffff error occurs, the test will fail with an informative message.
//! To retry, simply run the test again manually from the command line:
//!
//! ```bash
//! # First run
//! RUST_LOG=error cargo test -p reev-core --test e2e_multi_step --quiet
//!
//! # If it fails with 0xffff error, run it again
//! RUST_LOG=error cargo test -p reev-core --test e2e_multi_step --quiet
//! ```
//!
//! Each retry will get a fresh quote from Jupiter API with current blockhash.
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test e2e_multi_step -- --nocapture > test_output.log 2>&1
//! ```

mod common;

use anyhow::Result;

use reev_core::QueryHandler;
use rstest::*;
use serial_test::serial;

use tracing::info;

/// Consolidated multi-step test that handles multiple operations in a single query
/// The QueryHandler's LLM-based planner should handle multi-step scenarios appropriately
#[rstest]
#[case("swap 0.1 sol to usdc then lend 10 usdc")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_multi_step(#[case] prompt: &str) -> Result<()> {
    info!("Testing prompt: {prompt}");
    info!("=====================================");

    // Initialize test runner
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize query handler
    let mut query_handler = QueryHandler::new().await?;

    // Reset wallet balance to ensure we have enough SOL for tests
    runner.reset_wallet_balance().await?;

    // Process the multi-step query through QueryHandler's LLM-based pipeline
    // The LLM will handle the sequence of operations appropriately
    let result = query_handler.process_query(prompt, &runner.pubkey).await?;

    // Verify the query was processed successfully
    if result.success {
        info!(
            "✅ Multi-step operation completed successfully with signature: {:?}",
            result.transaction_signature
        );
    } else {
        // Handle the case where the swap failed
        let error_msg = result.error_message.unwrap_or("Unknown error".to_string());

        // Check if it's a Jupiter 0xffff error
        if error_msg.contains("custom program error: 0xffff") {
            // This is a Jupiter program error, which can happen due to market conditions
            tracing::error!(
                "❌ Multi-step operation encountered Jupiter program error (0xffff): {}",
                error_msg
            );
            tracing::info!("💡 This error can occur due to market conditions, slippage too tight, or liquidity issues");
            tracing::info!("ℹ️ Run the test again manually to retry with fresh market data");
            return Err(anyhow::anyhow!("Jupiter program error (0xffff): {error_msg}\nRun test again to retry with fresh market data"));
        } else {
            // For other errors, we expect operations to succeed
            tracing::error!(
                "❌ Multi-step operation encountered an error: {}",
                error_msg
            );
            return Err(anyhow::anyhow!("Multi-step operation failed: {error_msg}"));
        }
    }

    info!("=============================");

    Ok(())
}
