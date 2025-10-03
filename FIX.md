# Plan to Fix Benchmark Scoring Logic

Based on the insightful analysis of the logs, the root cause of the test failure has been identified. This document outlines the plan to correct it.

## 1. The Core Problem

The current logic in `handle_step` fetches the final on-chain state for scoring *before* the transaction has been fully confirmed and its state changes are visible.

As correctly pointed out, the log sequence reveals the flaw:
1. `Transaction simulation successful. Executing transaction...`
2. `Fetching accounts for observation...` (This is the observation for scoring)
3. `Missing 2 accounts, retrying...` (The account doesn't exist yet)
4. `✅ Transaction executed for 110-JUP-LEND-DEPOSIT-SOL. Status: Success` (Too late, the observation is already done)
5. `Scoring assertion failed...` (The score is calculated on the stale observation)

The simulation does not create the account on the forked ledger; only the executed transaction does. We are trying to observe the result of the transaction before it has actually landed.

## 2. The Solution

The solution is to restructure the `handle_step` function in `reev/crates/reev-lib/src/solana_env/step.rs` to ensure there is **only one observation at the very end of the step**, after the transaction has been confirmed.

This makes the flow logical: `Simulate -> Execute -> Observe -> Score`.

## 3. Action Plan

### Step 1: Revert Unnecessary Changes

The previous attempts to fix this involved adding complex retry logic to `observation.rs`. This was a patch for a symptom, not a fix for the root cause. This logic will be removed to simplify the code.

-   **File:** `reev/crates/reev-lib/src/solana_env/observation.rs`
-   **Action:** Revert the `get_observation` function to its original, simple implementation that performs a single fetch without retries.

### Step 2: Refactor `handle_step` for a Single, Final Observation

The `handle_step` function will be refactored to separate transaction execution from state observation.

-   **File:** `reev/crates/reev-lib/src/solana_env/step.rs`
-   **Action:**
    1.  Create local variables before the transaction execution block to store the outcome (e.g., `tx_status`, `tx_error`, `tx_logs`, `info_json`).
    2.  The `match env.rpc_client.send_and_confirm_transaction(...)` block will now *only* be responsible for executing the transaction and populating these local variables.
    3.  Move the call to `observation::get_observation(...)` to be the very last thing that happens before returning the `Step` result. This guarantees it runs *after* execution is complete.
    4.  Construct the final `Step` object using the populated variables and the final, correct observation.

### Step 3: Verify the Fix

-   **Action:** Run the test suite again: `cargo test --test benchmarks_test -- --nocapture`.
-   **Expected Outcome:** The `110-JUP-LEND-DEPOSIT-SOL` test will now pass consistently with a score of `1.0`. The logs will show the "Fetching accounts" message *after* the "Transaction executed" message.