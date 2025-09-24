# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap, ensuring each development phase builds logically on the last.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features. Each phase results in a testable, functional system.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic, a non-negotiable principle for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner) to ensure modularity and separation of concerns.
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework, not a linked library.

---

## Completed Work (Phases 1-4)

The foundational work for the `reev` framework and its initial reporting layer is complete. This includes:

-   **Workspace Scaffolding**: The `reev-lib` and `reev-runner` crates are set up with a clear separation of concerns.
-   **Hermetic `SolanaEnv`**: The core `SolanaEnv` is implemented, successfully managing the `surfpool` lifecycle and configuring on-chain state via RPC "cheatcodes".
-   **Core Action Handlers**: Handlers for `sol_transfer` and `spl_transfer` (including mint initialization) are fully functional.
-   **Benchmark Specification**: The Rust structs representing the `reev-benchmarks` format are defined and integrated with `serde` for YAML parsing.
-   **Generic `DummyAgent`**: A generic test agent that executes the `expected_tool_calls` from any benchmark file is implemented.
-   **Reporting Primitives**:
    -   The canonical `TestResult` struct is defined to aggregate all evaluation data.
    -   The `reev-runner` serializes this `TestResult` into a structured YAML output.
    -   Advanced metrics (Task Success Rate, Tool Selection Accuracy, Parameterization Accuracy) are calculated.
    -   A human-readable ASCII tree of the execution trace is rendered to the console.

---

## Phase 5: Advanced Observability (OpenTelemetry) (Next Up)

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces for advanced performance analysis, as detailed in `UI.md`.

1.  **Instrument Core Logic**:
    -   Integrate the `tracing` and `opentelemetry` crates into `reev-lib` and `reev-runner`.
    -   Add `spans` to key operations (`reset`, `step`, `agent.get_action`, RPC calls) to measure latency and capture contextual attributes.

2.  **Configure OTel Exporter**:
    -   In `reev-runner`, implement the logic to initialize an OTel pipeline.
    -   Provide a default exporter that prints traces to the console or a file, with clear instructions on how to swap it for an exporter that sends data to Jaeger, Honeycomb, etc.

## Phase 6: Interactive TUI Cockpit

**Goal:** Create a rich, interactive Terminal User Interface for running benchmarks and analyzing results, as detailed in `UI.md`.

1.  **Create `reev-tui` Crate**:
    -   Initialize a new binary crate and add `ratatui` as a dependency.

2.  **Build the TUI Layout**:
    -   Implement the three-panel layout (Benchmark Navigator, Trace View, Details Pane) and the top-level control bar (`[RUN]`, `[SETTINGS]`) as specified in `UI.md`.

3.  **Implement Interactive Functionality**:
    -   The TUI will manage the full evaluation lifecycle: scanning for benchmarks, selecting tests, triggering runs, and loading the resulting YAML trace files to populate the UI dynamically.

## Phase 7: LLM Integration

**Goal:** Replace the `DummyAgent` with a true LLM-powered agent and evaluate its performance on the `reev-benchmarks` suite.

1.  **Implement `LlmAgent`**:
    -   Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   It will use `reqwest` to communicate with an external LLM API.
    -   Implement robust prompt engineering to serialize the `AgentObservation` into the LLM's context and parse the tool-call response back into an `AgentAction`.

2.  **Expand `reev-benchmarks` Suite**:
    -   Curate a comprehensive suite of test cases covering all capability areas (T1-T5) defined in `IDEA.md`, including multi-step reasoning, error handling, and optimization challenges.