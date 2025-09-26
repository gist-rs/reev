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

## Future Work: The Comparative Agent Framework

The next major phase is to integrate a true AI-powered agent into the framework and evaluate it against our existing deterministic agent.

### Phase 11: Implementing a Comparative AI Agent

**Goal:** Integrate a new, `rig`-based **AI Agent** directly into the existing `reev-agent` crate. This will allow for direct, comparative evaluation against the **Deterministic Agent** using a simple query parameter for routing.

1.  **The Comparative Agent Framework:** We will use a single agent service (`reev-agent`) that can operate in two modes:
    *   **The Deterministic Agent (Ground Truth):** Triggered by a `?mock=true` query parameter, this mode uses the existing, code-based logic to produce the perfect instruction. It is our baseline for correctness.
    *   **The AI Agent (Subject):** Triggered by the absence of the `mock` parameter, this mode will use the `rig` crate to query a real AI model (LLM, VLM, etc.), asking it to choose the correct tool and parameters to solve the user's request.

2.  **Define On-Chain Actions as `rig` Tools:** The core on-chain actions (`sol_transfer`, `spl_transfer`) will be defined as structs within `reev-agent` that implement the `rig::Tool` trait. This will allow the AI Agent to present them to the model as callable functions, securely separating the model's reasoning from the secure execution of our Rust code.

3.  **Implement Agent Selection in Runner:** The `reev-runner` CLI will be updated so the `--agent` flag controls the URL used to call the `reev-agent` service:
    *   `--agent deterministic`: The runner calls `.../gen/tx?mock=true`.
    *   `--agent ai`: The runner calls `.../gen/tx`.

### Advanced Observability

As outlined in `OTEL.md`, the framework will be instrumented with OpenTelemetry to provide deep insights into performance and agent behavior, enabling analysis in professional observability platforms.
