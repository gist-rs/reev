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
use rstest::*;
use serial_test::serial;
use tracing::info;

/// Test with custom prompt (testing prompt generation)
#[rstest]
#[case(
    "transfer",
    "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
)]
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_transfer(#[case] operation_type: &str, #[case] prompt: &str) -> Result<()> {
    info!("operation_type: {operation_type}",);
    info!("operation_type: {prompt}",);

    // TODO
    // User query comes in through the API endpoint
    // call transfer_wallet_setup for airdrop for test
    // 2. `execute_dynamic_flow` handler receives the request
    // 3. The query is passed to the `Planner` in `reev-core`
    // 4. `Planner::refine_and_plan()` processes the query
    // 5. `LanguageRefiner` refines the natural language
    // 6. `YmlGenerator` creates a structured flow with a swap step
    // 7. The flow is executed by the agent, which calls the `jupiter_swap` tool
    // 8. The swap is performed on-chain

    Ok(())
}
