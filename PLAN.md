# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. It serves as the single source of truth for the project's roadmap.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with a solid foundation and progressively adding features.
-   **Hermetic & Reproducible**: The core environment must be completely isolated and deterministic for verifiable evaluations.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between components (Environment, Agent, Runner).
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is an external, ephemeral service managed by the framework.

---

## Completed Work (Phases 1-12)

The foundational work for the `reev` framework is complete and has been validated through a full integration test suite. This includes:

-   **Workspace Scaffolding**: `reev-lib`, `reev-runner`, and `reev-tui` crates are set up.
-   **Hermetic `SolanaEnv`**: The core environment manages the `surfpool` validator lifecycle, ensuring reproducible on-chain state for every run.
-   **Benchmark Specification**: A clear, YAML-based format (`reev-benchmarks`) defines test cases, including initial state, prompts, and final state assertions.
-   **Scoring and Persistence**: The `reev-runner` calculates a final score based on on-chain assertions and persists the complete evaluation record to a local SQLite database.
-   **Reporting Primitives**: The runner can output results as structured YAML and a human-readable console tree.
-   **Integration Testing**: A full integration test suite validates the end-to-end scoring and persistence logic, ensuring reliable evaluations.
-   **Interactive TUI Cockpit (Phase 10)**: The static TUI prototype was transformed into a fully interactive evaluation cockpit capable of discovering benchmarks, orchestrating asynchronous runs, and displaying live results and detailed execution traces.
-   **Comparative AI Agent (Phase 11)**: A dual-agent architecture was implemented within the `reev-agent` service. It can now operate in two modes: a `deterministic` mode (the oracle) and an `ai` mode that uses the `rig` crate to query an LLM. This enables direct, comparative evaluation of AI performance against a ground truth baseline.
-   **Advanced TUI Controls (Phase 12)**: Enhanced the TUI with agent selection tabs (`Deterministic`, `Gemini`, `Local`) and implemented concurrency management to ensure only one benchmark runs at a time.

---

## Next Up: Phase 13 - Granular Scoring Model

**Goal:** Transition from a binary (0/1) success/failure score to a more nuanced, cumulative scoring system.

**Rationale:** A simple pass/fail score doesn't capture the subtleties of an agent's performance. An agent might get most of the steps right but fail on a single assertion. A granular score provides a more accurate picture of progress and helps pinpoint specific areas for improvement.

1.  **Weighted Assertions**:
    *   Modify the benchmark YAML format to allow an optional `weight` for each `final_state_assertions`. The default weight will be `1` if not specified.

2.  **Cumulative Score Calculation**:
    *   Refactor the `reev-lib/src/score.rs` module. The new logic will sum the weights of all passed assertions.
    *   The final score will be the ratio of the accumulated score to the total possible score (sum of all weights), represented as a float between 0.0 and 1.0.
    *   Transaction success remains a prerequisite. If the transaction fails, the score is `0.0`, as no on-chain state change occurred to evaluate.

3.  **Database and UI Updates**:
    *   Update the database schema in `reev-runner` to store the score as a `REAL` or `FLOAT` type.
    *   Update the `reev-tui` and console runner to display the score as a percentage (e.g., `84%`).

---

## Current Phase: Phase 13 - Composite Scoring Model (Final)

**Goal:** Implement the definitive scoring model that evaluates instruction quality separately from on-chain execution, providing a fair and nuanced assessment of agent performance.

**Rationale:** The agent's primary job is to generate a high-quality transaction based on the prompt (its "homework"). The on-chain outcome is a secondary, binary success signal. This model separates these concerns. `final_state_assertions` are now purely for diagnostics, not scoring.

1.  **Instruction Score (75% Weight):**
    *   Provides granular, partial credit for the agent's reasoning.
    *   Compares the generated instruction's `program_id`, `accounts`, and `data` against the `expected_instructions` ground truth, using weights defined in the benchmark.

2.  **On-Chain Execution Score (25% Weight):**
    *   A simple, binary score reflecting the transaction's outcome on `surfpool`.
    *   `1.0` for a `Success` status, `0.0` for a `Failure` status.

3.  **Correctly Evaluating Scenarios like `003-spl-transfer-fail.yml`:**
    *   In this benchmark, the agent is asked to send 15 USDC but only has 10.
    *   A good agent generates a **perfect instruction**, earning the full **75%** from the Instruction Score.
    *   The transaction **fails on-chain**, earning **0%** from the On-Chain Score.
    *   The **final score is 75%**, correctly reflecting a high-quality agent whose transaction failed due to environmental constraints, not its own error.

---

## Next Up: Phase 14 - End-to-End AI Agent Integration Test

**Goal:** Create an integration test that validates the full lifecycle of an AI agent solving a complex benchmark that the deterministic agent cannot.

**Rationale:** This test is the ultimate proof that the `reev` framework can successfully evaluate a real, capable on-chain AI agent. It validates the entire loop from runner to environment to agent to LLM and back.

1.  **Create New Test:** A new test file (`crates/reev-runner/tests/ai_agent_test.rs`).
2.  **Orchestrate Services:** The test will spawn the `reev-agent` in AI mode.
3.  **Run Complex Benchmark:** Execute the Jupiter Swap benchmark (`100-jup-swap-sol-usdc.yml`), which is unsolvable by the deterministic agent.
4.  **Assert Success:** Assert that the final score is `1.0`, proving the AI agent successfully generated and executed a valid transaction.

---

## Future Work

With the core framework and TUI now complete, future work will focus on expanding the benchmark suite and exploring more advanced agent capabilities.

### Advanced Observability
As outlined in `OTEL.md`, the framework can be instrumented with OpenTelemetry to provide deep insights into performance and agent behavior, enabling analysis in professional observability platforms.