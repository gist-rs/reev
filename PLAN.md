# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap, ensuring each development phase builds logically on the last.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features. Each phase results in a testable, functional system.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic, a non-negotiable principle for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner) to ensure modularity and separation of concerns.
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework, not a linked library.

---

## Completed Work (Phases 1-3)

The foundational work for the `reev` framework is complete. This includes:

-   **Workspace Scaffolding**: The `reev-lib` and `reev-runner` crates are set up with a clear separation of concerns.
-   **Hermetic `SolanaEnv`**: The core `SolanaEnv` is implemented, successfully managing the `surfpool` lifecycle and configuring on-chain state via RPC "cheatcodes".
-   **Core Action Handlers**: Handlers for `sol_transfer` and `spl_transfer` (including mint initialization) are fully functional.
-   **Benchmark Specification**: The Rust structs representing the `reev-benchmarks` format are defined and integrated with `serde` for YAML parsing.
-   **Generic `DummyAgent`**: A generic test agent that executes the `expected_tool_calls` from any benchmark file is implemented.

---

## Phase 4: Metrics, Tracing, and Reporting (Next Up)

**Goal:** Build the essential infrastructure for capturing and reporting detailed evaluation results. This phase transforms the runner from a simple executor into a powerful analysis tool.

1.  **Define `TestResult` Struct**:
    -   In a new `reev-lib/src/results.rs` module, create the canonical `TestResult` struct. This will aggregate the `TestCase` info, the final `QuantitativeScores`, and the complete `ExecutionTrace`.

2.  **Implement YAML Trace Serialization**:
    -   Modify the `reev-runner` to construct the `TestResult` struct at the end of an evaluation.
    -   Serialize this struct into a structured YAML file (e.g., `results/spl-transfer-001.yml`). This file becomes the foundational artifact for all subsequent UI work.

3.  **Implement Advanced Quantitative Metrics**:
    -   In `reev-lib/src/metrics.rs`, add the logic to calculate **Tool Selection Accuracy (TSA)** and **Parameterization Accuracy (PA)** by comparing the `ExecutionTrace` against the `ground_truth.expected_tool_calls`.

4.  **Implement ASCII Tree Visualization**:
    -   Create a "renderer" in `reev-runner` that parses the final `TestResult` struct.
    -   This renderer will print a human-readable ASCII tree of the execution flow to the console, providing an immediate summary of the agent's performance.

## Phase 5: Interactive TUI Cockpit

**Goal:** Create a rich, interactive Terminal User Interface for running benchmarks and analyzing results, as detailed in `UI.md`.

1.  **Create `reev-tui` Crate**:
    -   Initialize a new binary crate and add `ratatui` as a dependency.
    -   This crate will be the main entry point for interactive sessions.

2.  **Build the TUI Layout**:
    -   Implement the three-panel layout (Benchmark Navigator, Trace View, Details Pane) and the top-level control bar (`[RUN]`, `[SETTINGS]`) as specified in `UI.md`.

3.  **Implement Interactive Functionality**:
    -   The TUI will manage the full evaluation lifecycle:
        -   Scan and display available benchmarks in Panel A.
        -   Allow the user to select tests and a model.
        -   Trigger the evaluation run via the `[RUN]` button, calling `reev-lib` functions in a non-blocking way.
        -   Load the resulting YAML trace files to populate the result views dynamically.

## Phase 6: Advanced Observability (OpenTelemetry)

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces for advanced performance analysis, as detailed in `UI.md`.

1.  **Instrument Core Logic**:
    -   Integrate the `tracing` and `opentelemetry` crates into the `reev-lib` and `reev-runner`.
    -   Add `spans` to key operations (`reset`, `step`, `agent.get_action`, RPC calls) to measure latency and capture contextual attributes.

2.  **Configure OTel Exporter**:
    -   In `reev-runner`, implement the logic to initialize an OTel pipeline.
    -   Provide a default exporter that prints traces to the console or a file, with clear instructions on how to swap it for an exporter that sends data to Jaeger, Honeycomb, etc.

## Phase 7: LLM Integration

**Goal:** Replace the `DummyAgent` with a true LLM-powered agent and evaluate its performance on the `reev-benchmarks` suite.

1.  **Implement `LlmAgent`**:
    -   Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   It will use `reqwest` to communicate with an external LLM API.
    -   Implement robust prompt engineering to serialize the `AgentObservation` into the LLM's context.
    -   Implement response parsing to safely convert the LLM's tool-call output into an `AgentAction`.

2.  **Expand `reev-benchmarks` Suite**:
    -   Curate a comprehensive suite of test cases covering all capability areas (T1-T5) defined in `IDEA.md`. Include multi-step reasoning, error handling, and optimization challenges.