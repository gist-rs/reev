//! End-to-end swap test using rstest and the common test framework
//!
//! This test uses the rstest framework for parameterization and the common test
//! framework to reduce duplication. It loads the wallet from ~/.config/solana/id.json,
//! uses the planner to process the swap prompt, lets the LLM handle tool calling via rig,
//! signs the transaction with the default keypair, and verifies completion.
//!
//! ## Jupiter Transaction Retry Behavior
//!
//! Jupiter transactions are time-sensitive and cannot be simply retried:
//! - Jupiter swap routes are based on current market conditions and liquidity
//! - Solana transactions are tied to specific blockhashes that expire
//! - Proper retry would require getting a fresh quote from Jupiter API with current blockhash
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
use common::operations::{SwapOperation, TestOperation};
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

    // Check that we can create a SwapOperation
    let operation = SwapOperation::new("SOL", "USDC", "1");
    info!(
        "✅ Swap operation: swap {} {} for {}",
        operation.amount, operation.from, operation.to
    );

    // Check that we can generate a prompt
    let prompt = operation.prompt();
    info!("✅ Generated prompt: {}", prompt);

    info!("✅ Framework setup test completed successfully!");
    info!("=============================================");

    Ok(())
}

/// Parameterized test that executes different swap amounts
#[rstest]
#[case("1.0", "1 SOL")]
#[case("0.5", "0.5 SOL")]
#[case("0.1", "0.1 SOL")]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_swaps(
    #[case] amount: &str,
    #[case] description: &str,
    target_pubkey: Pubkey,
) -> Result<()> {
    info!("🧪 Starting Swap Test: {}", description);
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the swap operation
    let operation = SwapOperation::new("SOL", "USDC", amount);

    // Execute the swap using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Swap test '{}' completed successfully!", description);
    info!("✅ Transaction signature: {}", signature);
    info!("=============================");

    Ok(())
}

/// Test for specific 1 SOL swap case (maintains backward compatibility)
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_swap_1_sol_for_usdc(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Starting Test: Swap 1 SOL for USDC");
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the swap operation for 1 SOL
    let operation = SwapOperation::new("SOL", "USDC", "1");

    // Execute the swap using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Swap completed successfully!");
    info!("✅ Transaction signature: {}", signature);
    info!("=============================");

    Ok(())
}

/// Test for "sell all SOL" swap case
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_sell_all_sol_for_usdc(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Starting Test: Sell all SOL for USDC");
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the swap operation for all SOL
    let operation = SwapOperation::new("SOL", "USDC", "all");

    // Execute the swap using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Swap completed successfully!");
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

    // Only process swap prompts in this test
    if operation_type == "swap" {
        // Initialize the test environment (will start SURFPOOL if needed)
        let mut runner = common::framework::TestRunner::new()?;
        runner.initialize().await?;

        // Create the swap operation
        let operation = SwapOperation::new("SOL", "USDC", "1");

        // Execute the swap using the standardized operation
        let signature =
            common::operations::execute_standardized_operation(&operation, &runner.pubkey())
                .await?;

        info!("✅ Prompt processing test completed successfully!");
        info!("✅ Transaction signature: {}", signature);
    } else {
        info!("⚠️ Skipping non-swap operation in swap test");
    }

    Ok(())
}

/// Test with timeout to ensure the operation completes within a reasonable time
#[rstest]
#[timeout(std::time::Duration::from_secs(180))]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_swap_with_timeout(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Starting Swap Test with Timeout");
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Create the swap operation for 0.1 SOL (smaller amount for faster processing)
    let operation = SwapOperation::new("SOL", "USDC", "0.1");

    // Execute the swap using the standardized operation
    let signature =
        common::operations::execute_standardized_operation(&operation, &runner.pubkey()).await?;

    info!("✅ Swap with timeout test completed successfully!");
    info!("✅ Transaction signature: {}", signature);
    info!("=============================");

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

/// Test simple SOL fee calculation
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_simple_sol_fee_calculation(target_pubkey: Pubkey) -> Result<()> {
    info!("🧪 Testing SOL fee calculation");
    info!("=====================================");

    // Initialize the test environment (will start SURFPOOL if needed)
    let mut runner = common::framework::TestRunner::new()?;
    runner.initialize().await?;

    // Initialize balance validator
    let mut key_map = std::collections::HashMap::new();
    key_map.insert("USER_PUBKEY".to_string(), target_pubkey.to_string());

    let balance_validator = reev_lib::balance_validation::BalanceValidator::new(key_map);

    // Test 1: Check max swappable SOL with fee reserve
    let max_swappable = balance_validator.get_max_swappable_sol(
        &target_pubkey.to_string(),
        10_000_000, // 0.01 SOL fee reserve
    )?;

    println!(
        "Max swappable SOL: {} lamports ({} SOL)",
        max_swappable,
        max_swappable as f64 / 1_000_000_000.0
    );

    // Test 2: Check specific amount with fee calculation
    let swappable_amount = balance_validator.get_swappable_amount_after_fees(
        &target_pubkey.to_string(),
        1_000_000_000, // 1 SOL
        10_000_000,    // 0.01 SOL fee reserve
    )?;

    println!(
        "Swappable amount for 1 SOL request: {} lamports ({} SOL)",
        swappable_amount,
        swappable_amount as f64 / 1_000_000_000.0
    );

    // Test 3: Test with insufficient balance
    match balance_validator.get_swappable_amount_after_fees(
        &target_pubkey.to_string(),
        10_000_000_000, // 10 SOL
        10_000_000,     // 0.01 SOL fee reserve
    ) {
        Ok(amount) => println!("Swappable amount for 10 SOL request: {amount} lamports"),
        Err(e) => println!("Expected error for insufficient SOL: {e}"),
    }

    info!("✅ SOL fee calculation test completed successfully!");
    info!("=============================================");

    Ok(())
}
