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
use common::operations::TransferOperation;
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
