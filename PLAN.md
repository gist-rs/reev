# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner).
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework.

---

## Completed Work (Phases 1-9)

The foundational work for the framework, reporting, interactive TUI, result persistence, and integration testing is complete.

-   **Workspace Scaffolding**: `reev-lib`, `reev-runner`, and `reev-tui` crates are set up.
-   **Hermetic `SolanaEnv`**: Manages the `surfpool` lifecycle and on-chain state.
-   **Benchmark Specification**: YAML-based test case definitions.
-   **Reporting Primitives**: Structured YAML output and console tree rendering.
-   **Interactive TUI Cockpit**: A `ratatui`-based TUI for running benchmarks and analyzing results.
-   **Observability**: The framework is instrumented with OpenTelemetry for tracing.
-   **LLM Integration**: The `LlmAgent` is implemented to generate raw Solana instructions from natural language prompts, and the `SolanaEnv` is equipped to execute them.
-   **Scoring and Persistence**: The `reev-runner` now calculates a final score based on on-chain state assertions and persists the complete evaluation record to a local SQLite database.
-   **Integration Testing**: A full integration test suite validates the scoring and persistence logic, ensuring reliable evaluations.

---

## Project Status: All Planned Phases Complete

All phases outlined in this plan are now complete. The `reev` framework is operational and ready for use. Future work will be defined in new planning documents.