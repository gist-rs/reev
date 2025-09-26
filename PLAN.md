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

## Future Work: The Dual-Agent Evaluation Strategy

The next major phase of the project is to build a true, LLM-based agent and evaluate its performance against our existing `MockAgent`. This comparative approach is the core of the `reev` framework's evaluation philosophy.

### Phase 11: Building a Tool-Calling LLM Agent

**Goal:** Implement a new `LlmAgent` that uses a tool-calling architecture to interact with a real LLM. This agent will be the "subject" of our evaluations.

1.  **The Dual-Agent Strategy:** We will maintain two distinct agents:
    *   **`MockAgent` (The Oracle):** The existing code-based agent. It serves as our ground truth, deterministically generating the perfect instruction for a given prompt. Its performance is the baseline for a perfect score.
    *   **`LlmAgent` (The Subject):** A new agent built with the `rig` crate. It will query a real LLM, asking it to choose the correct on-chain action ("tool") and parameters to fulfill a user's prompt.

2.  **Define On-Chain Actions as `rig` Tools:** The core on-chain actions (`sol_transfer`, `spl_transfer`) will be defined as structs that implement the `rig::Tool` trait. This will allow the `LlmAgent` to present them to the LLM as callable functions, securely separating the LLM's reasoning from the instruction's execution.

3.  **Implement Agent Selection:** The `reev-runner` and `reev-tui` will be updated to allow the user to select which agent to run a benchmark against (e.g., `--agent mock` vs. `--agent llm`). This enables direct, side-by-side comparison of the LLM's performance against the perfect oracle.

### Advanced Observability

As outlined in `OTEL.md`, the framework will be instrumented with OpenTelemetry to provide deep insights into performance and agent behavior, enabling analysis in professional observability platforms.
