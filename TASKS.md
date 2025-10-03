# TASKS.md: Actionable To-Do List

This file breaks down high-level goals from `PLAN.md` into specific, actionable tasks.

---

## Completed: Phase 13 - Composite Scoring Model

**Goal:** Refactor the scoring system to a composite model that separately evaluates instruction quality and on-chain state correctness.

-   [x] **Refactor Scoring Logic (`reev-lib`):**
    -   [x] The main function `calculate_final_score` was created in `score.rs`.
    -   [x] It combines two scores with a 75/25 weighting: `instruction_score` and `onchain_score`.
    -   [x] The `onchain_score` is a simple binary check for transaction `Success` or `Failure`.
    -   [x] `final_state_assertions` are no longer used for scoring and are for diagnostics only.

-   [x] **Implement Instruction Scoring (`reev-lib`):**
    -   [x] A new module, `instruction_score.rs`, was created.
    -   [x] The `calculate_instruction_score` function provides granular, partial credit by comparing the generated instruction against the benchmark's `expected_instructions`.
    -   [x] The logic correctly resolves placeholder pubkeys (e.g., "USER_WALLET_PUBKEY") against the test's `key_map`.

-   [x] **Update Benchmark Format (`reev-lib`):**
    -   [x] The `BenchmarkInstruction` and `BenchmarkAccountMeta` structs were updated to include `weight` fields.
    -   [x] The `data` and `data_weight` fields were made optional to handle cases where data scoring is not applicable.

-   [x] **Update All Benchmarks (`benchmarks/`):**
    -   [x] All existing benchmarks (`001`, `002`, `003`, `100`, `110`) were reviewed and updated to the new format.
    -   [x] Incorrect or non-deterministic `expected_instructions` were removed from complex DeFi benchmarks (`100`, `110`).
    -   [x] Descriptions were updated to reflect the new scoring reality.

-   [x] **Fix and Validate All Tests (`reev-runner`):**
    -   [x] `scoring_test.rs` and `benchmarks_test.rs` were updated to use the new `calculate_final_score` function.
    -   [x] All test helpers and mock instruction generators were fixed to produce truly "perfect" instructions.
    -   [x] All tests are now passing.

---

## Completed: Phase 15 - Expand Jupiter Lend Benchmarks

**Goal:** Add benchmarks for lending and withdrawing USDC to increase test coverage of Jupiter's Lend functionality.

-   [x] **1. Create `111-jup-lend-deposit-usdc.yml` Benchmark:**
    -   [x] Defined a new benchmark file for depositing USDC into a lending market.
    -   [x] The `initial_state` gives the user a starting USDC balance.
    -   [x] The `prompt` is "Lend 50 USDC using Jupiter."
    -   [x] The `final_state_assertions` verify the change in USDC and L-Token balances.

-   [x] **2. Create `112-jup-lend-withdraw-usdc.yml` Benchmark:**
    -   [x] Defined a new benchmark file for withdrawing USDC.
    -   [x] The `initial_state` gives the user a starting L-Token balance.
    -   [x] The `prompt` is "Withdraw 50 USDC using Jupiter."
    -   [x] The `final_state_assertions` verify the change in L-Token and USDC balances.

-   [x] **3. Implement "Smart Test" Helpers (`reev-runner/tests/common/helpers.rs`):**
    -   [x] Created the `prepare_jupiter_lend_deposit_usdc` async function.
    -   [x] Created a placeholder `prepare_jupiter_lend_withdraw_usdc` function. The test for this benchmark currently fails as the `jup-sdk` does not yet support withdrawal operations.

-   [x] **4. Update `benchmarks_test.rs`:**
    -   [x] Added the new benchmark IDs to the logic that selects the correct "smart test" helper.

---

## Next Up: Phase 16 - Implement Jupiter Withdraw in SDK

**Goal:** Add support for Jupiter Lend withdrawals to the `jup-sdk` to enable the `112-jup-lend-withdraw-usdc.yml` benchmark.

-   [ ] **1. Research Jupiter Lend Withdrawal:**
    -   [ ] Investigate the necessary on-chain instructions for withdrawing from a Jupiter lending pool.
    -   [ ] Identify the required accounts and data.

-   [ ] **2. Implement `withdraw` in `jup-sdk`:**
    -   [ ] Add a `withdraw` method to the `Jupiter` client in the `jup-sdk` crate.
    -   [ ] Implement the logic to fetch and construct the correct withdrawal transaction.

-   [ ] **3. Update `reev-runner` Test Helper:**
    -   [ ] Remove the placeholder logic in `prepare_jupiter_lend_withdraw_usdc`.
    -   [ ] Integrate the new `jup-sdk::withdraw` method.

-   [ ] **4. Validate and Fix Tests:**
    -   [ ] Run the `benchmarks_test` suite and confirm that the `112-jup-lend-withdraw-usdc.yml` test now passes with a perfect score.