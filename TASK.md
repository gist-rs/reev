# TASK: Implement a Two-Transaction Approach for Jupiter Lend Benchmarks

## Problem

The `110-JUP-LEND-DEPOSIT-SOL` benchmark is unstable. The root cause is a race condition and a state consistency issue within the `surfpool` test validator.

1.  **Simulation Failure:** When setup instructions (e.g., creating a WSOL account) and the main action (e.g., Jupiter lend) are bundled into a single transaction, the simulation fails with a `0x1776` error. This is because the state change from the setup instructions is not visible to the main instruction within the same simulation.
2.  **Observation Failure:** When the transaction succeeds, a race condition often occurs where the final observation is made before the newly created L-Token account is visible via the RPC endpoint, causing the scoring assertion to fail.

## Solution: Two-Transaction Approach

To make the test stable and accurately model the real-world process, the setup and the main action must be separated into two distinct, sequential transactions.

This ensures that the prerequisite accounts (like the WSOL ATA) are fully created and confirmed on-chain before the main program logic (Jupiter Lend) is simulated and executed.

## Implementation Plan

### 1. Modify `crates/reev-runner/tests/common/helpers.rs`

-   **Create a new `async` function: `execute_jupiter_lend_setup`**.
    -   This function will be responsible for creating and executing the prerequisite transaction to wrap SOL.
    -   It will generate the `create_associated_token_account`, `system_instruction::transfer`, and `spl_token::instruction::sync_native` instructions.
    -   It will sign and send this as a transaction using `env.rpc_client.send_and_confirm_transaction`.
    -   This function will directly modify the on-chain state and will not return any instructions.

-   **Refactor `prepare_jupiter_lend_deposit`**.
    -   Remove the logic for creating the SOL-wrapping instructions.
    -   This function's sole responsibility will now be to prepare the final Jupiter Lend instruction(s) and run `jup_sdk::surfpool::preload_accounts`.
    -   It will correctly assume that the WSOL account already exists when it runs.

### 2. Modify `crates/reev-runner/tests/benchmarks_test.rs`

-   Inside the main test loop, within the `if test_case.id.starts_with("110-JUP")` block:
    1.  First, `await` the new `execute_jupiter_lend_setup` helper function. This will set up the required on-chain state.
    2.  Then, `await` the refactored `prepare_jupiter_lend_deposit` function to get the final Jupiter instruction(s).
    3.  Pass **only** the Jupiter instruction(s) to the `env.step()` function for execution and scoring.

## Expected Outcome

The test for benchmark `110-JUP-LEND-DEPOSIT-SOL` will be robust and pass consistently. The simulation error will be resolved, and the observation race condition will be eliminated because the final observation will correctly find the L-Token ATA created by the now-confirmed transaction.