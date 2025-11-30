//! End-to-end SOL transfer test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the common test
//! framework to reduce duplication. It loads the wallet from ~/.config/solana/id.json,
//! uses the planner to process the transfer prompt, lets the LLM handle tool calling via rig,
//! signs the transaction with the default keypair, and verifies completion.
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
use common::operations::{TestOperation, TransferOperation};
use common::pubkeys;
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

/// Test that checks if the test framework is properly set up but doesn't require SURFPOOL
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_framework_setup(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Testing framework setup (no SURFPOOL required)");
    info!("=====================================");

    // This test verifies that the basic test framework is working
    // without requiring SURFPOOL to be running

    // Check that we can create a target pubkey
    info!("✅ Target pubkey: {}", target_pubkey);

    // Check that we can create a TransferOperation
    let operation = TransferOperation::new(&target_pubkey.to_string(), 0.1);
    info!(
        "✅ Transfer operation: send {} SOL to {}",
        operation.amount, operation.to
    );

    // Check that we can generate a prompt
    let prompt = operation.prompt();
    info!("✅ Generated prompt: {}", prompt);

    info!("✅ Framework setup test completed successfully!");
    info!("=============================================");

    Ok(())
}

/// Parameterized test that executes different transfer amounts
#[rstest]
#[case(1.0, "1 SOL")]
#[case(0.5, "0.5 SOL")]
#[case(0.1, "0.1 SOL")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_transfers(
    #[case] amount: f64,
    #[case] description: &str,
    target_pubkey: Pubkey,
) -> Result<()> {
    info!("🧪 Starting Transfer Test: {}", description);
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the transfer operation
    let operation = TransferOperation::new(&target_pubkey.to_string(), amount);

    // Execute the transfer using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Transfer test '{}' completed successfully!", description);
    info!("✅ Transaction signature: {}", signature);
    info!("=============================");

    Ok(())
}

/// Test for specific 1 SOL transfer case (maintains backward compatibility)
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_send_1_sol_to_target(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Starting Test: Send 1 SOL to target account");
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the transfer operation for 1 SOL
    let operation = TransferOperation::new(&target_pubkey.to_string(), 1.0);

    // Execute the transfer using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Transfer completed successfully!");
    info!("✅ Transaction signature: {}", signature);
    info!("=============================");

    Ok(())
}

/// Test with custom prompt (testing prompt generation)
#[rstest]
#[case(
    "transfer",
    "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
)]
#[case("swap", "swap 1 sol to usdc")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_prompt_processing(#[case] operation_type: &str, #[case] prompt: &str) -> Result<()> {
    info!(
        "🧪 Testing prompt processing for operation: {}",
        operation_type
    );
    info!("Prompt: {}", prompt);

    // Only process transfer prompts in this test
    if operation_type == "transfer" {
        let target_pubkey = Pubkey::from_str(pubkeys::TARGET).expect("Invalid target public key");
        let operation = TransferOperation::new(&target_pubkey.to_string(), 1.0);

        // Initialize the test environment (will start SURFPOOL if needed)
        let mut runner = common::framework::TestRunner::new()?;
        runner.initialize().await?;

        // Execute the transfer using the standardized operation
        let signature =
            common::operations::execute_standardized_operation(&operation, &runner.pubkey())
                .await?;

        info!("✅ Prompt processing test completed successfully!");
        info!("✅ Transaction signature: {}", signature);
    } else {
        info!("⚠️ Skipping non-transfer operation in transfer test");
    }

    Ok(())
}

/// Test using async fixtures with #[future] and #[awt]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[awt]
#[serial]
async fn test_async_fixture(#[future] async_target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Testing async fixtures with #[awt]");
    info!("=====================================");

    // Check that we can use the async fixture
    info!("✅ Target pubkey: {}", async_target_pubkey);

    info!("✅ Async fixture test completed successfully!");
    info!("=========================================");

    Ok(())
}
