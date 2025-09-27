# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner).
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework.

---

## Completed Work (Phases 1-11)

The foundational work for the `reev` framework is complete and has been validated through a full integration test suite. This includes:

-   **Workspace Scaffolding**: `reev-lib`, `reev-runner`, and `reev-tui` crates are set up.
-   **Hermetic `SolanaEnv`**: The core environment manages the `surfpool` validator lifecycle, ensuring reproducible on-chain state for every run.
-   **Benchmark Specification**: A clear, YAML-based format (`reev-benchmarks`) defines test cases, including initial state, prompts, and final state assertions.
-   **Scoring and Persistence**: The `reev-runner` calculates a final score based on on-chain assertions and persists the complete evaluation record to a local SQLite database.
-   **Reporting Primitives**: The runner can output results as structured YAML and a human-readable console tree.
-   **Integration Testing**: A full integration test suite validates the end-to-end scoring and persistence logic, ensuring reliable evaluations.
-   **Interactive TUI Cockpit (Phase 10)**: The static TUI prototype was transformed into a fully interactive evaluation cockpit capable of discovering benchmarks, orchestrating asynchronous runs, and displaying live results and detailed execution traces.
-   **Comparative AI Agent (Phase 11)**: A dual-agent architecture was implemented within the `reev-agent` service. It can now operate in two modes: a `deterministic` mode (the oracle) and an `ai` mode that uses the `rig` crate to query an LLM. This enables direct, comparative evaluation of AI performance against a ground truth baseline.

---

## Current Phase: Phase 12 - Advanced TUI Controls

**Goal:** Enhance the TUI with agent selection and better execution control.

1.  **Agent Selection Tabs**:
    *   Add a tabbed interface to the TUI header to allow the user to select the agent for the benchmark run. The tabs will be: `Deterministic`, `Gemini`, and `Local`.
    *   The selection will control which agent is used when a benchmark is executed.

2.  **Concurrency Management**:
    *   Implement logic to disable the other agent tabs while a benchmark is running.
    *   Ensure that only one benchmark task can be executed at a time, preventing multiple runs from being queued or started simultaneously.

---

## Future Work

With the core framework and TUI now complete, future work will focus on expanding the benchmark suite and exploring more advanced agent capabilities.

### Advanced Observability
As outlined in `OTEL.md`, the framework can be instrumented with OpenTelemetry to provide deep insights into performance and agent behavior, enabling analysis in professional observability platforms.