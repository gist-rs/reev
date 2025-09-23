# TASKS.md: Phase 1 - Foundational Scaffolding & Core Types

This file lists the specific, actionable tasks required to complete Phase 1 of the development plan. The goal is to establish a solid, well-typed foundation for the entire framework.

## Workspace and Crate Setup

-   [ ] Initialize a new Cargo workspace named `reev`.
-   [ ] Create a library crate `reev-lib` within the workspace. This crate will contain all the core logic, traits, and types for the framework.
-   [ ] Create a binary crate `reev-runner` within the workspace. This will be the entry point for the evaluation harness application.
-   [ ] Configure the root `Cargo.toml` to correctly reference both workspace members.
-   [ ] Add initial dependencies to `reev-lib`:
    -   `serde` (with `derive` feature) for serialization.
    -   `serde_json` for flexible data structures.
    -   `anyhow` for error handling.

## Core Environment Traits and Types (`reev-lib`)

-   [ ] Create a new module `reev-lib/src/env.rs`.
-   [ ] In `env.rs`, define the `Step<Obs>` struct to represent the output of an environment step. It should contain `observation`, `reward`, `terminated`, `truncated`, and `info`.
-   [ ] In `env.rs`, define the core `GymEnv` trait with the following methods:
    -   `reset(&mut self, seed: Option<u64>, options: Option<serde_json::Value>) -> anyhow::Result<Self::Observation>`
    -   `step(&mut self, action: Self::Action) -> anyhow::Result<Step<Self::Observation>>`
    -   `render(&self)`
    -   `close(&mut self)`
-   [ ] Create a new module `reev-lib/src/agent.rs`.
-   [ ] In `agent.rs`, define the `AgentAction` struct. It should contain `tool_name` (String) and `parameters` (e.g., `HashMap<String, serde_json::Value>`).
-   [ ] In `agent.rs`, define the `AgentObservation` struct. It should contain fields like `last_transaction_status`, `last_transaction_error`, etc., as specified in the IDEA.

## Benchmark Specification Types (`reev-lib`)

-   [ ] Add `serde_yaml` as a dependency to `reev-lib`.
-   [ ] Create a new module `reev-lib/src/benchmark.rs`.
-   [ ] In `benchmark.rs`, define a `TestCase` struct that represents a single test case from a YAML file.
-   [ ] Define the necessary sub-structs for `TestCase`, including:
    -   `InitialState` (e.g., list of accounts to create).
    -   `GroundTruth` (containing `final_state_assertions`).
    -   `StateAssertion` (e.g., an enum for `SolBalance`, `TokenAccountBalance`).
-   [ ] Add `#[derive(Deserialize)]` to all benchmark structs to enable parsing from YAML files.
-   [ ] Create a dummy `solana-bench-001.yml` file in a new `benchmarks/` directory at the root of the workspace to test the deserialization.
-   [ ] Write a unit test in `reev-lib` that successfully loads and parses the dummy benchmark file.