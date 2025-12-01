//! End-to-end swap test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the common test
//! framework to reduce duplication. It loads the wallet from ~/.config/solana/id.json,
//! uses the planner to process the swap prompt, lets the LLM handle tool calling via rig,
//! signs the transaction with the default keypair, and verifies completion.
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
//! RUST_LOG=error cargo test -p reev-core --test e2e_swap --quiet
//!
//! # If it fails with 0xffff error, run it again
//! RUST_LOG=error cargo test -p reev-core --test e2e_swap --quiet
//! ```
//!
//! Each retry will get a fresh quote from Jupiter API with current blockhash.
//!
//! ## Running the Test with Proper Logging
//!
//! To run this test with the recommended logging filters to reduce noise:
//!
//! ```bash
//! RUST_LOG=info cargo test -p reev-core --test e2e_swap -- --nocapture > test_output.log 2>&1
//! ```

mod common;

use anyhow::Result;

use common::pubkeys;
use reev_core::QueryHandler;
use rstest::*;
use serial_test::serial;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::info;

/// Test fixture for the target public key
#[fixture]
fn target_pubkey() -> Pubkey {
    Pubkey::from_str(pubkeys::TARGET).expect("Invalid target public key")
}

/// Async test fixture for the target public key
#[fixture]
async fn async_target_pubkey() -> Pubkey {
    Pubkey::from_str(pubkeys::TARGET).expect("Invalid target public key")
}

/// Consolidated swap test that handles both specific amount and "all" keyword cases
/// The QueryHandler's LLM-based planner should handle both scenarios appropriately
#[rstest]
#[case("swap 1 sol for usdc")]
#[case("swap all sol for usdc")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_swap(#[case] prompt: &str, _target_pubkey: Pubkey) -> Result<()> {
    info!("Testing prompt: {prompt}");
    info!("=====================================");

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
    if result.success {
        info!(
            "✅ Swap completed successfully with signature: {:?}",
            result.transaction_signature
        );
    } else {
        // Handle the case where the swap failed
        let error_msg = result.error_message.unwrap_or("Unknown error".to_string());

        // For any swap failure, check if it's the Jupiter 0xffff error
        if error_msg.contains("custom program error: 0xffff") {
            // This is a Jupiter program error, which can happen due to market conditions
            tracing::error!(
                "❌ Swap encountered Jupiter program error (0xffff): {}",
                error_msg
            );
            tracing::info!("💡 This error can occur due to market conditions, slippage too tight, or liquidity issues");
            tracing::info!("ℹ️ Run the test again manually to retry with fresh market data");
            return Err(anyhow::anyhow!("Jupiter program error (0xffff): {error_msg}\nRun test again to retry with fresh market data"));
        } else {
            // For other errors, we expect swaps to succeed
            tracing::error!("❌ Swap encountered an error: {}", error_msg);
            return Err(anyhow::anyhow!("Swap failed: {error_msg}"));
        }
    }

    info!("=============================");

    Ok(())
}
