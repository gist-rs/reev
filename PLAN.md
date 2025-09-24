# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. The goal is to build the comprehensive LLM agent evaluation framework as specified in `IDEA.md`.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with the core environment and progressively adding the agent interface, metrics, and advanced features. Each phase should result in a testable, partially functional system.
-   **Test-Driven**: Each component, especially the `SolanaEnv`, should be accompanied by unit and integration tests to ensure its behavior is correct and reproducible.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between major components (Environment, Agent, Runner) to ensure modularity and separation of concerns.
-   **Service-Oriented Environment**: The Solana test environment (`surfpool`) will be treated as an external, ephemeral service managed by the evaluation runner. Interaction will occur exclusively through its public JSON-RPC API, ensuring a clean architectural boundary.

---

## Phase 1: Foundational Scaffolding (Completed)

This phase focused on setting up the project structure and defining the fundamental data types that will be used throughout the framework.

-   **Cargo Workspace**: Established a workspace with two primary crates: `reev-lib` for the core framework logic and `reev-runner` for the command-line orchestrator.
-   **Core Traits and Structs**: Defined the central `GymEnv` and `Agent` traits, along with the primary data structures (`Step`, `AgentAction`, `AgentObservation`).
-   **Benchmark Specification**: Created the Rust structs (`TestCase`, `InitialAccountState`, `GroundTruth`, etc.) to represent `reev-benchmarks` test cases, with `serde` support for YAML deserialization.

## Phase 2: Hermetic `SolanaEnv` with External Process Management (Completed)

This phase was dedicated to building the reproducible `SolanaEnv`. The implementation treats the `surfpool` validator as a managed, black-box service, ensuring hermetic test execution.

-   **`SolanaEnv` Implementation**: Implemented the `GymEnv` trait for `SolanaEnv`, which manages the `surfpool` child process and communicates via an `RpcClient`.
-   **`reset` Logic**: The `reset` function successfully spawns a new `surfpool` instance, waits for it to become responsive, generates ephemeral keypairs for the test case, and uses the `surfnet_setAccount` RPC to configure the initial on-chain state.
-   **`step` Logic**: The `step` function correctly translates an `AgentAction` into a signed Solana transaction, sends it to the validator, and returns the result. Initial implementation covered basic system program instructions.
-   **`close` Logic**: The `close` function ensures the `surfpool` child process is cleanly terminated after each test run.

## Phase 3: Expanding Action Space & Agent Capabilities (Completed)

This phase expanded the environment's capabilities to handle more complex on-chain interactions and made the `DummyAgent` more generic.

-   **SPL-Token Transfer Action**: Implemented a robust action handler for `spl_token::instruction::transfer`.
-   **Mint Account Initialization**: Enhanced the `SolanaEnv::reset` logic and benchmark specification to support the on-the-fly creation and initialization of SPL Token mint accounts, enabling tests with custom tokens like the real USDC.
-   **Generic Dummy Agent**: Refactored the `DummyAgent` to dynamically execute the `expected_tool_calls` from the loaded benchmark's `ground_truth`. This allows the agent to run any benchmark without requiring code changes.
-   **NFT Transfer Capability**: Verified that the existing `spl_transfer` logic correctly handles NFT transfers (as they are a subset of SPL token transfers).

## Phase 4: Metrics, Tracing, and Reporting (In Progress)

With the core interaction loop functional, this phase adds the ability to measure performance and understand what the agent did.

1.  **Implement Trace Capture**:
    -   Create a `Trace` or `ExecutionLog` struct.
    -   Modify the evaluation loop to record every `thought`, `action`, `observation`, and `info` dictionary into a hierarchical trace.

2.  **Implement Quantitative Metrics**:
    -   Implement the logic to calculate the core metrics based on the captured trace and the test case's `ground_truth`.
    -   Start with **Task Success Rate (TSR)** by evaluating the `final_state_assertions`.
    -   Follow with **Tool Selection Accuracy (TSA)** and **Gas Consumption Efficiency (GCE)**.

3.  **Implement ASCII Trace Visualization**:
    -   Write a renderer that traverses the captured `Trace` and generates a human-readable ASCII tree, as specified in `IDEA.md`.

4.  **Generate a Summary Report**:
    -   At the end of a benchmark run, aggregate all metrics and generate a summary report (e.g., in Markdown or JSON format) that provides a high-level overview of the agent's performance.

## Phase 5: UI/UX for Visualization and Reporting (Next Up)

This phase focuses on building a rich user interface for visualizing and analyzing test results, as detailed in `UI.md`.

1.  **Implement Structured YAML Output**:
    -   The `reev-runner` will serialize the `ExecutionTrace` for each test case into a structured YAML file. This serves as the data foundation for all UI layers.

2.  **Implement ASCII Tree Rendering**:
    -   Create a renderer that parses the YAML trace and prints a human-readable ASCII tree to the console, providing an immediate summary of the agent's execution flow.

3.  **Develop Interactive TUI with `Ratatui`**:
    -   Create a new crate, `reev-tui`, to house a full-featured Terminal User Interface.
    -   The TUI will feature a multi-panel layout allowing users to navigate a list of test cases, view the corresponding execution tree, and drill down into the details of any specific action or observation.

## Phase 6: LLM Integration and Advanced Evaluation

This final phase will integrate a real LLM and add more nuanced qualitative evaluation.

1.  **Implement an LLM-Powered Agent**:
    -   Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   Use `reqwest` to communicate with an external LLM API (e.g., OpenAI, Anthropic).
    -   Implement the logic for prompt engineering: serialize the observation and conversation history into a prompt for the LLM.
    -   Parse the LLM's response (likely JSON for function calling) back into an `AgentAction`.

2.  **Implement LLM-as-a-Judge**:
    -   Write the functionality to take a completed execution trace and submit it to a powerful "judge" LLM.
    -   Develop the rubric and prompt templates for the judge to score aspects like reasoning, efficiency, and robustness.
    -   Integrate these qualitative scores into the final report.

3.  **Expand `reev-benchmarks`**:
    -   Curate a comprehensive suite of test cases covering all the capability areas (T1-T5) defined in `IDEA.md`.
    -   Include simple "unit tests," multi-step "integration tests," and "adversarial" edge cases.