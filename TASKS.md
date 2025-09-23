# TASKS.md: Development Roadmap

This document outlines the development plan for `reev`, focusing on a mock-first, modular approach to building the evaluation framework.

---

## Phase 1: Foundational Scaffolding & Core Types (Complete)

-   [x] Initialize Cargo Workspace (`reev-lib`, `reev-runner`).
-   [x] Define Core Traits and Structs (`GymEnv`, `AgentAction`, `AgentObservation`, `Step`).
-   [x] Define Benchmark Specification (`TestCase` and related structs).
-   [x] Implement a unit test to load and parse a benchmark YAML file.

---

## Phase 2: Mocked Solana Environment & Core Action

**Goal:** Create a fully functional, in-memory simulation of the Solana environment that can execute a basic SOL transfer.

-   [ ] **Task 2.1: Refactor `SolanaEnv` to be In-Memory**
    -   [ ] Remove `solana-sdk`, `solana-client`, and `solana-program` dependencies from `reev-lib/Cargo.toml`.
    -   [ ] Remove all `solana-test-validator` process management code from `solana_env.rs`.
    -   [ ] Add an in-memory `HashMap` to `SolanaEnv` to store the mocked state of on-chain accounts (e.g., `HashMap<String, AccountState>`).
    -   [ ] Update the `reset` function to populate this in-memory store directly from the `initial_state` of a `TestCase`.

-   [ ] **Task 2.2: Create the Actions Module**
    -   [ ] Create a new module `reev-lib/src/actions/mod.rs`.
    -   [ ] Define a common `Action` trait that all specific transaction handlers will implement (e.g., `trait Action { fn execute(&self, state: &mut MockedState, params: &Value) -> Result<()>; }`).

-   [ ] **Task 2.3: Implement Mocked SOL Transfer**
    -   [ ] Create a new file `reev-lib/src/actions/sol_transfer.rs`.
    -   [ ] Implement the `sol_transfer` action logic. It should take parameters (from, to, amount), validate them, and update the balances in the in-memory state map.
    -   [ ] The function should return appropriate errors for failures (e.g., insufficient funds).

-   [ ] **Task 2.4: Implement the `step` Function as a Dispatcher**
    -   [ ] Modify `SolanaEnv::step` to read the `tool_name` from the incoming `AgentAction`.
    -   [ ] Based on the `tool_name` (e.g., "sol_transfer"), call the corresponding action handler from the `actions` module.
    -   [ ] Update the `AgentObservation` with the new state from the in-memory map and return it.

---

## Phase 3: End-to-End Evaluation & Metrics

**Goal:** Achieve a complete, end-to-end run of the evaluation harness using the mocked environment and calculate a meaningful result.

-   [ ] **Task 3.1: Update `DummyAgent` for SOL Transfer**
    -   [ ] Modify `DummyAgent` to return a hardcoded `sol_transfer` `AgentAction` with correct parameters when it sees the initial observation from the `transfer-simple-001` benchmark.

-   [ ] **Task 3.2: Verify Metrics Calculation**
    -   [ ] Run the `reev-runner`.
    -   [ ] Confirm that the `sol_transfer` is executed by the mock environment.
    -   [ ] Confirm that the final `AgentObservation` reflects the new balances.
    -   [ ] Confirm that the `calculate_task_success_rate` function now passes, yielding a TSR of 1.0.

---

## Phase 4: Expanding Mocked Actions (Future Work)

**Goal:** Increase the capability of the mocked environment to support a wide range of Solana interactions.

-   [ ] **Task 4.1: Implement Mocked SPL-Token Transfer**
-   [ ] **Task 4.2: Implement Mocked Token2022 Transfer**
-   [ ] **Task 4.3: Implement Mocked NFT Transfer**
-   [ ] **Task 4.4: Implement Mocked Swap**
-   [ ] **Task 4.5: Implement Mocked Deposit (e.g., to a lending protocol)**
-   [ ] **Task 4.6: Implement Mocked Withdraw**

---

## Phase 5: Real Environment Implementation (Deferred)

**Goal:** Re-implement the environment actions to interact with a real `solana-test-validator` using the `solana-sdk`.

-   [ ] **Task 5.1: Resolve Solana SDK Dependencies and Imports**
-   [ ] **Task 5.2: Re-implement `sol_transfer` with Real Transactions**
-   [ ] **Task 5.3: Re-implement `reset` to Create Real On-Chain Accounts**