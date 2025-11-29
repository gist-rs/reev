//! End-to-end SOL swap test using the standardized test framework
//!
//! This test uses the new test framework to eliminate duplication and
//! provide consistent behavior across all e2e tests.

mod common;

use anyhow::Result;
use common::{framework::TestRunner, operations::SwapOperation};
use rstest::rstest;

/// Test end-to-end swap flow with different parameters
///
/// This test follows the 6-step process:
/// 1. Create YML prompt with wallet context
/// 2. Send prompt to LLM
/// 3. Generate flow from prompt
/// 4. Execute flow with tools
/// 5. Extract transaction signature
/// 6. Verify transaction completion
#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[case("SOL", "USDC", "0.1")]
#[case("SOL", "USDC", "0.5")]
#[case("SOL", "USDC", "1.0")]
#[case("SOL", "USDC", "all")]
async fn test_swap_operations(
    #[case] from_token: &str,
    #[case] to_token: &str,
    #[case] amount: &str,
) -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create swap operation
    let operation = SwapOperation::new(from_token, to_token, amount);

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Swap {amount} {from_token} for {to_token} completed with signature: {signature}"
    );
    Ok(())
}

/// Test end-to-end sell all SOL flow
///
/// This is a special case of the swap test with fixed parameters
#[tokio::test(flavor = "multi_thread")]
async fn test_sell_all_sol_for_usdc() -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create operation to sell all SOL for USDC
    let operation = SwapOperation::new("SOL", "USDC", "all");

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Sell all SOL for USDC completed with signature: {signature}"
    );
    Ok(())
}

/// Test end-to-end swap with minimum amount
///
/// This test verifies that the system can handle small amount swaps
#[tokio::test(flavor = "multi_thread")]
async fn test_small_amount_swap() -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create operation to swap a very small amount
    let operation = SwapOperation::new("SOL", "USDC", "0.01");

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Small amount swap completed with signature: {signature}"
    );
    Ok(())
}
