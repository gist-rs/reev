```# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner).
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework.

---

## Completed Work (Phases 1-9)

The foundational work for the `reev` framework is complete and has been validated through a full integration test suite. This includes:

-   **Workspace Scaffolding**: `reev-lib`, `reev-runner`, and `reev-tui` crates are set up.
-   **Hermetic `SolanaEnv`**: The core environment manages the `surfpool` validator lifecycle, ensuring reproducible on-chain state for every run.
-   **Benchmark Specification**: A clear, YAML-based format (`reev-benchmarks`) defines test cases, including initial state, prompts, and final state assertions.
-   **Code-Based Agent**: The `reev-agent` can parse natural language prompts and generate correct, raw Solana instructions for both SOL and SPL-Token transfers using code, not LLM generation.
-   **Scoring and Persistence**: The `reev-runner` calculates a final score based on on-chain assertions and persists the complete evaluation record to a local SQLite database.
-   **Reporting Primitives**: The runner can output results as structured YAML and a human-readable console tree.
-   **Static TUI Prototype**: A non-interactive `ratatui`-based TUI cockpit was built to serve as the visual blueprint for the interactive tool.
-   **Integration Testing**: A full integration test suite validates the end-to-end scoring and persistence logic, ensuring reliable evaluations.

---

## Current Phase: Phase 10 - TUI Interactivity

**Goal:** Transform the static TUI prototype into a fully interactive evaluation cockpit capable of orchestrating benchmark runs and displaying live results.

-   **Dynamic Benchmark Discovery**: The TUI will automatically discover all available benchmark files from the `benchmarks/` directory at startup.
-   **`reev-runner` as a Library**: The `reev-runner`'s core execution logic will be refactored into a callable library function. This will allow the TUI (and other future tools) to programmatically run benchmarks.
-   **Asynchronous Execution & Live Updates**: The TUI will run benchmarks in a background thread to keep the interface responsive. It will use channels to receive live updates, displaying the status of each benchmark (`PENDING`, `RUNNING`, `SUCCEEDED`, `FAILED`) and populating the trace and details panes upon completion.

---

## Future Work

The following are planned features to be implemented after the completion of the interactive TUI.

### Advanced Observability

As outlined in `OTEL.md`, the framework will be instrumented with OpenTelemetry to provide deep insights into performance and agent behavior, enabling analysis in professional observability platforms.

### Advanced Agent Logic: Tool Calling

The current `reev-agent` uses a direct mapping from prompt intent to a specific code function (e.g., "send sol" maps to a `create_sol_transfer` function). The next evolution of the agent will implement a more flexible **tool-calling** architecture.

**Agent Decision Mapping Table:**

| Prompt Intent                | Current Implementation    | Future "Tool Calling" Model                |
| ---------------------------- | ------------------------- | ------------------------------------------ |
| Native SOL Transfer          | `handle_sol_transfer()`   | `[call_tool('sol_transfer', amount, to)]`  |
| SPL Token Transfer           | `handle_spl_transfer()`   | `[call_tool('spl_transfer', amount, to)]`  |
| Swap on Orca                 | (Not implemented)         | `[call_tool('orca_swap', token_in, token_out)]` |
| Create Token Account         | (Not implemented)         | `[call_tool('create_token_account', owner, mint)]` |

This will decouple the agent's intent recognition from the execution logic, allowing it to dynamically select from a list of available "tools" (functions) and their parameters. This is a crucial step towards building more general-purpose and extensible on-chain agents.

---

## TUI and Mocking Strategy

The interactive TUI for running and viewing benchmarks is now complete. This provides a rich, interactive way to engage with the `reev` framework. Additionally, we have implemented a mocking strategy for the `reev-agent`. All calls to the transaction generation endpoint (`/gen/tx`) are now routed to a `mock_generate_transaction` function by default. This is controlled by a `?mock=true` query parameter, which is now the default. This allows us to develop the frontend and runner without relying on a live LLM, while providing a clear path for future integration.
