# TASKS.md: Development Roadmap

This document provides a detailed, actionable checklist for the development of the `reev` framework, based on the high-level phases outlined in `PLAN.md`.

---

## Completed Work (Phases 1-4)

The foundational framework and the initial reporting layer are complete.

-   [x] **Workspace and Core Primitives**:
    -   [x] Initialized `reev-lib` and `reev-runner` crates.
    -   [x] Defined core traits (`GymEnv`, `Agent`) and structs (`TestCase`, `AgentAction`, `ExecutionTrace`).
-   [x] **Hermetic Solana Environment**:
    -   [x] Implemented `SolanaEnv` to manage the `surfpool` lifecycle.
    -   [x] Implemented state setup via RPC "cheatcodes".
-   [x] **Action Handling**:
    -   [x] Implemented `sol_transfer` and `spl_transfer` action handlers.
    -   [x] Implemented dynamic SPL Token Mint initialization.
-   [x] **Foundational Reporting**:
    -   [x] Defined the canonical `TestResult` struct for all reporting.
    -   [x] `reev-runner` serializes the `TestResult` to a structured YAML output.
    -   [x] Implemented advanced metrics: Task Success Rate (TSR), Tool Selection Accuracy (TSA), and Parameterization Accuracy (PA).
    -   [x] Implemented an ASCII tree renderer for human-readable console output.

---

## Phase 5: Advanced Observability (OpenTelemetry) (Next Up)

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces for advanced performance analysis.

-   [x] **Task 5.1: Add Dependencies**
    -   [x] Add `tracing`, `opentelemetry`, `opentelemetry-sdk`, and an exporter crate (e.g., `opentelemetry-stdout`) to the `Cargo.toml` of `reev-runner`.

-   [x] **Task 5.2: Initialize OTel Pipeline**
    -   [x] In `reev-runner/src/main.rs`, create a function to set up the global tracer provider.
    -   [x] Configure a simple "stdout" exporter that prints traces to the console in a machine-readable format for initial verification.
    -   [x] Ensure the pipeline is properly shut down at the end of the `main` function.

-   [x] **Task 5.3: Instrument Code with Spans**
    -   [x] Add `#[tracing::instrument]` macros to key functions to create spans.
    -   [x] Target functions:
        -   `run_evaluation_loop` (the root span for a test case).
        -   `SolanaEnv::reset`.
        -   `SolanaEnv::step`.
        -   `DummyAgent::get_action`.
    -   [x] Add relevant context (e.g., `benchmark.id`, `step_number`, `tool_name`) to spans as attributes.
    -   [x] Record errors and important logs as `events` on the appropriate spans.

---

## Phase 6: Interactive TUI Cockpit (Future)

**Goal:** Build the interactive `Ratatui`-based TUI for a "mission control" experience.

-   [x] **Task 6.1: Create `reev-tui` Crate**
    -   [x] Initialize the new binary crate and add `ratatui` and `crossterm` as dependencies.
-   [x] **Task 6.2: Build the TUI Layout**
    -   [x] Implement the static three-panel layout and the header bar as specified in `UI.md`.
-   [x] **Task 6.3: Implement State Management**
    -   [x] Develop the application state struct that will hold the list of benchmarks, the results of runs, and the current UI focus.
-   [ ] **Task 6.4: Implement Interactive Functionality**
    -   [ ] Wire up the `[RUN]` button to trigger the evaluation logic from `reev-lib`.
    -   [ ] Implement the logic to load and display YAML results in the UI panels.

---

## Phase 7: LLM Integration (Future)

**Goal:** Replace the `DummyAgent` with a real LLM-powered agent.

-   [ ] **Task 7.1: Implement `LlmAgent`**
    -   [ ] Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   [ ] Use `reqwest` to build a client for an external LLM API.
-   [ ] **Task 7.2: Implement Prompt Engineering & Parsing**
    -   [ ] Develop the logic to serialize an `AgentObservation` and conversation history into a prompt.
    -   [ ] Develop the logic to parse the LLM's JSON tool-call response back into an `AgentAction`.
-   [ ] **Task 7.3: Expand `reev-benchmarks` Suite**
    -   [ ] Create new, more complex benchmarks that test multi-step reasoning, error handling, and optimization.