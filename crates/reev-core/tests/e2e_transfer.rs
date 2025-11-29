//! End-to-end SOL transfer test using the standardized test framework
//!
//! This test uses the new test framework to eliminate duplication and
//! provide consistent behavior across all e2e tests.

mod common;

use anyhow::Result;
use common::{framework::TestRunner, operations::TransferOperation, pubkeys::target};
use rstest::rstest;

/// Test end-to-end transfer flow with different parameters
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
#[case(1.0)]
#[case(0.5)]
#[case(0.1)]
async fn test_transfer_operations(#[case] amount: f64) -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create transfer operation
    let operation = TransferOperation::new(&target().to_string(), amount);

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Transfer operation completed with signature: {signature}"
    );
    Ok(())
}

/// Test end-to-end transfer flow with specific recipient
///
/// This is a specialized test case for transferring to a known recipient
#[tokio::test(flavor = "multi_thread")]
async fn test_transfer_to_specific_recipient() -> Result<()> {
    // Initialize test runner
    let mut runner = TestRunner::new()?;
    runner.initialize().await?;

    // Create transfer operation with a specific amount
    let operation = TransferOperation::new(&target().to_string(), 0.75);

    // Execute the operation using the standardized flow
    let signature = runner.execute_operation(&operation).await?;

    println!(
        "✅ Transfer to specific recipient completed with signature: {signature}"
    );
    Ok(())
}
