//! End-to-end lend test using the standardized test framework
//!
//! This test uses the new test framework to eliminate duplication and
//! provide consistent behavior across all e2e tests.

mod common;

use anyhow::Result;
use common::{framework::TestRunner, operations::LendOperation};
use rstest::rstest;

/// Test end-to-end lend flow with different parameters
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
#[case(100.0)]
#[case(200.0)]
#[case(500.0)]
async fn test_lend_operations(#[case] lend_amount: f64) -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create lend operation
    let operation = LendOperation::new(lend_amount);

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Lend {lend_amount} USDC completed with signature: {signature}"
    );
    Ok(())
}

/// Test end-to-end lend all USDC flow
///
/// This is a special case of the lend test with fixed parameters
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn test_lend_all_usdc() -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create operation to lend all USDC
    let operation = LendOperation::lend_all();

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!("✅ Lend all USDC completed with signature: {signature}");
    Ok(())
}

/// Test end-to-end lend with minimum amount
///
/// This test verifies that the system can handle small amount lending
#[tokio::test(flavor = "multi_thread")]
async fn test_small_amount_lend() -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create operation to lend a very small amount
    let operation = LendOperation::new(1.0);

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Small amount lend completed with signature: {signature}"
    );
    Ok(())
}
