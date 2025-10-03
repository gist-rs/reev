# TASKS.md: Granular To-Do List

This file breaks down high-level goals from `PLAN.md` into specific, actionable tasks.

---

## Phase 13: Granular Scoring Model

**Goal:** Refactor the scoring system from a binary 0/1 to a cumulative score based on weighted assertions.

### 1. Update Benchmark Data Model (`reev-lib`)
-   [x] In `reev-lib/src/benchmark.rs`, modify the `StateAssertion` enum or its container struct.
-   [x] Add an optional `weight` field (e.g., `f64` or `u32`) to each assertion variant.
-   [x] Ensure `serde` can still correctly parse benchmarks that *don't* have the `weight` field, defaulting it to `1.0`.
-   [x] Update the `110-jup-lend-deposit-sol.yml` benchmark to include weights for its assertions as a test case.

### 2. Refactor Scoring Logic (`reev-lib`)
-   [x] In `reev-lib/src/score.rs`, update the `calculate_score` function.
-   [x] The function should first calculate the `total_possible_score` by summing the weights of all `final_state_assertions` in the benchmark.
-   [x] Initialize an `achieved_score` to `0.0`.
-   [x] The check for transaction success must remain. If it fails, return `0.0` immediately.
-   [x] Iterate through each assertion. If an assertion passes, add its `weight` to the `achieved_score`.
-   [x] After checking all assertions, calculate the final score: `achieved_score / total_possible_score`. Handle the case where `total_possible_score` is zero to avoid division by zero.
-   [x] Update the function's documentation to reflect the new cumulative logic.

### 3. Update Database Schema (`reev-runner`)
-   [x] In `reev-runner/src/db.rs`, locate the `CREATE TABLE` statement for the results table.
-   [x] Change the data type of the `score` column from `INTEGER` to `REAL` to accommodate floating-point values. (Verified it was already `REAL`).
-   [x] Ensure the insertion logic correctly handles the new `f64` score.

### 4. Update UI and Reporting (`reev-runner` & `reev-tui`)
-   [x] In `reev-runner/src/renderer.rs` (or wherever the console output is generated), format the score to be displayed as a percentage (e.g., `84.5%`).
-   [ ] In `reev-tui`, update the relevant view to format and display the float score correctly, likely as a percentage.
-   [x] In `reev-lib/src/results.rs`, ensure the `score` field in `TestResult` is changed to `f64`.

### 5. Validation and Testing
-   [ ] Create a new benchmark file specifically designed to test the weighted scoring. It should have multiple assertions with different weights.
-   [ ] Create a test scenario that fails one assertion but passes others.
-   [ ] Run the test and verify that the calculated score is correct (e.g., if a weight `1` assertion passes and a weight `1` assertion fails, the score should be `0.5`).
-   [ ] Run an existing test (like the SOL transfer) and verify its score is `1.0` (or `100%`) since the default weight is `1`.