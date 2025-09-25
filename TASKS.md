# TASKS.md: Development Roadmap

This document provides a detailed, actionable checklist for the development of the `reev` framework, based on the high-level phases outlined in `PLAN.md`.

---

## Completed Work (Phases 1-7)

The foundational framework, agent integration, and UI are complete. The next phase focuses on implementing scoring and persisting results.

-   [x] **Workspace and Core Primitives**: Initialized `reev-lib`, `reev-runner`, and `reev-tui` crates.
-   [x] **Hermetic Solana Environment**: Implemented `SolanaEnv` to manage the `surfpool` lifecycle.
-   [x] **Benchmark Specification**: Defined the `reev-benchmarks` YAML format for test cases.
-   [x] **Reporting & UI**: Implemented YAML/ASCII output and a full `ratatui` TUI cockpit.
-   [x] **Observability**: Added OpenTelemetry tracing for performance analysis.
-   [x] **LLM Integration**: Reworked the agent model to support raw instruction generation from a third-party API.

---

## Phase 7: LLM Integration - Instruction Generation Model (Completed)

**Goal:** Evaluate an LLM's ability to act as a raw instruction generator.

-   [x] **Task 7.1: Redefine `AgentAction`**
    -   [x] Refactor the `AgentAction` struct to wrap a native `solana_sdk::instruction::Instruction`.
    -   [x] Create helper structs to deserialize the specific JSON response from the third-party API.
-   [x] **Task 7.2: Implement `LlmAgent` for Instruction Generation**
    -   [x] The agent now sends a prompt and receives a JSON object containing a raw instruction.
    -   [x] Implemented logic to parse the nested JSON (`{"result": {"text": {...}}}`) and decode the Base58 data string.
-   [x] **Task 7.3: Adapt `SolanaEnv` to Process Raw Instructions**
    -   [x] Refactor the `SolanaEnv::step` function to accept the new `AgentAction`.
    -   [x] The environment now dynamically finds the required signer from its keymap and signs the agent-generated transaction before submission.
    -   [x] The old, tool-based `actions` module has been removed.

---

## Phase 8: Scoring and Persistence (Completed)

**Goal:** Implement a robust system for scoring evaluation runs and persisting the results in a local database.

-   [x] **Task 8.1: Add Database Dependency**
    -   [x] Add the `turso` crate to the `reev-runner`'s `Cargo.toml`.
-   [x] **Task 8.2: Implement Database Manager**
    -   [x] Create a new module in `reev-runner` (`db.rs`) to handle all database interactions.
    -   [x] Implement a function to initialize the database connection and create the necessary tables if they don't exist.
-   [x] **Task 8.3: Implement Scoring Logic**
    -   [x] Create a function that takes the `final_observation` and the `ground_truth.final_state_assertions` from the benchmark.
    -   [x] This function iterates through the assertions, compares them against the actual on-chain state, and returns a final score (1.0 for pass, 0.0 for fail).
-   [x] **Task 8.4: Persist Results**
    -   [x] In `reev-runner/src/main.rs`, after a run completes, call the scoring function.
    -   [x] Call the database manager to insert a new record containing the benchmark ID, timestamp, prompt, the generated instruction (serialized to JSON), the final state, and the score.

---

## Phase 9: Integration Testing and Environment Fixes (Completed)

**Goal:** Build a robust integration test suite to validate the scoring logic and fix environment setup issues.

-   [x] **Task 9.1: Fix `SolanaEnv` Account Initialization**
    -   [x] Modify the `SolanaEnv::reset` function in `reev-lib` to correctly parse and apply the `data` field from a benchmark's `initial_state` for SPL Token accounts.
-   [x] **Task 9.2: Stabilize SPL-Token Scoring Tests**
    -   [x] Confirm that `test_scoring_pass_case` passes with a score of `1.0`.
    -   [x] Confirm that `test_scoring_fail_case` passes with a score of `0.0`.
-   [x] **Task 9.3: Create SOL Transfer Scoring Tests**
    -   [x] Create `crates/reev-runner/tests/benchmarks/001-sol-transfer-pass.yml`.
    -   [x] Create `crates/reev-runner/tests/benchmarks/002-sol-transfer-fail.yml`.
-   [x] **Task 9.4: Implement SOL Transfer Scoring Test Logic**
    -   [x] Add `test_sol_transfer_pass_case` and `test_sol_transfer_fail_case` to `scoring_test.rs`.
    -   [x] Validate `SolBalance` assertion logic by asserting correct scores.
