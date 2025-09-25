# NOW: Integration Testing and Environment Fixes

This document outlines the immediate development focus for the `reev` framework. With Phase 8 complete, the current goal is to build a robust integration test suite to validate the scoring and persistence logic, and to fix the underlying environment bugs that the initial tests have uncovered.

## Current Objective

The primary objective is to ensure the scoring mechanism is reliable and accurate for all assertion types. This involves fixing the environment's account initialization process and creating dedicated "pass" and "fail" test cases for both SPL-Token and SOL transfers.

## Action Plan

1.  **Task 9.1: Fix `SolanaEnv` Account Initialization**:
    -   Modify the `SolanaEnv::reset` function in `reev-lib` to correctly parse and apply the `data` field from a benchmark's `initial_state`.
    -   This is critical for setting up SPL Token accounts with the correct mint, owner, and amount, which is the root cause of the `InvalidAccountData` error in the current tests.

2.  **Task 9.2: Stabilize SPL-Token Scoring Tests**:
    -   With the environment fix in place, re-run the `scoring_test.rs` integration test.
    -   Confirm that `test_scoring_pass_case` now passes with a score of `1.0`.
    -   Confirm that `test_scoring_fail_case` still passes with a score of `0.0`.

3.  **Task 9.3: Create SOL Transfer Scoring Tests**:
    -   Create a new test benchmark file: `crates/reev-runner/tests/benchmarks/001-sol-transfer-pass.yml`. This test should have correct `SolBalance` assertions.
    -   Create a new test benchmark file: `crates/reev-runner/tests/benchmarks/002-sol-transfer-fail.yml`. This test should have deliberately incorrect `SolBalance` assertions.

4.  **Task 9.4: Implement SOL Transfer Scoring Test Logic**:
    -   Add two new test functions to `crates/reev-runner/tests/scoring_test.rs`: `test_sol_transfer_pass_case` and `test_sol_transfer_fail_case`.
    -   These tests will load and run the new SOL transfer benchmarks, asserting scores of `1.0` and `0.0` respectively.
    -   This will validate that the `SolBalance` assertion logic in `calculate_score` is working correctly.