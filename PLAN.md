# PLAN.md: Master Development Plan for `reev`

This document outlines the high-level, phased development plan for the `reev` project. The goal is to build the comprehensive LLM agent evaluation framework as specified in `IDEA.md`.

## Guiding Principles

-   **Iterative Development**: Build the framework in layers, starting with the core environment and progressively adding the agent interface, metrics, and advanced features. Each phase should result in a testable, partially functional system.
-   **Test-Driven**: Each component, especially the `SolanaEnv`, should be accompanied by unit and integration tests to ensure its behavior is correct and reproducible.
-   **Clear Interfaces**: Define clean `trait`-based interfaces between major components (Environment, Agent, Runner) to ensure modularity and separation of concerns.

---

## Phase 1: Foundational Scaffolding & Core Types

This phase focuses on setting up the project structure and defining the fundamental data types that will be used throughout the framework. The goal is to establish a solid, well-typed foundation.

1.  **Initialize Cargo Workspace**:
    -   Set up a new Rust workspace.
    -   Create the initial crates: `reev-lib` for the core framework logic and `reev-runner` for the binary that will orchestrate the evaluations.

2.  **Define Core Traits and Structs (`reev-lib`)**:
    -   Create a `src/env.rs` module.
    -   Define the central `GymEnv` trait, which will be the Rust equivalent of the Gymnasium `Env` class.
    -   Define the primary data structures:
        -   `Step<Observation>`: The standard return type for the `step` method.
        -   `AgentAction`: A struct representing a tool call from the agent (e.g., tool name and parameters).
        -   `AgentObservation`: A struct representing the state of the world returned by the environment.

3.  **Define Benchmark Specification (`reev-lib`)**:
    -   Create a `src/benchmark.rs` module.
    -   Define the Rust structs that represent a `SolanaBench` test case (`TestCase`, `InitialState`, `GroundTruth`, etc.).
    -   Use `serde` to enable deserialization from YAML, which will be the format for benchmark files.

## Phase 2: Hermetic `SolanaEnv` Implementation

This phase is dedicated to building the heart of the framework: the reproducible Solana environment. The focus is on correctly managing the `solana-test-validator` process and simulating the on-chain world.

1.  **Implement `SolanaEnv` Struct (`reev-lib`)**:
    -   Create the `SolanaEnv` struct, which will hold the state for the environment (e.g., validator process handle, RPC client).
    -   Implement the `GymEnv` trait for `SolanaEnv`.

2.  **Implement `reset` Logic**:
    -   Write the logic to programmatically start and stop `solana-test-validator` using `std::process::Command`.
    -   Implement functionality to load a specific on-chain state based on the `initial_state` definition from a `TestCase`. This involves programmatically creating accounts, deploying programs, and setting balances.

3.  **Implement `step` Logic**:
    -   Write the logic to receive an `AgentAction`, translate it into a Solana transaction, sign it, and send it to the test validator.
    -   Wait for transaction confirmation and query the result (success/failure, logs, etc.).
    -   Format the result into the `Step<AgentObservation>` return type.

4.  **Implement `close` Logic**:
    -   Ensure the `solana-test-validator` process is cleanly terminated when the environment is closed.

## Phase 3: The Evaluation Runner & Agent Interface

This phase focuses on the orchestrator application that runs the benchmarks and provides a way to plug in an agent.

1.  **Benchmark Loading (`reev-runner`)**:
    -   Implement the logic to scan a directory for `SolanaBench` `.yml` files.
    -   Parse the YAML files into the `TestCase` structs defined in Phase 1.

2.  **Create the Main Evaluation Loop (`reev-runner`)**:
    -   Loop through each loaded `TestCase`.
    -   For each case, instantiate and `reset` the `SolanaEnv`.
    -   Run the agent-environment interaction loop until the `terminated` or `truncated` flag is set.
    -   `close` the environment after each test case.

3.  **Define the `Agent` Trait (`reev-lib`)**:
    -   Define a simple `trait Agent` with a single method: `get_action(observation: &AgentObservation) -> AgentAction`.

4.  **Create a Dummy Agent**:
    -   Create a simple struct `DummyAgent` that implements the `Agent` trait.
    -   This agent will return hardcoded actions, allowing for end-to-end testing of the runner-environment loop without needing a real LLM.

## Phase 4: Metrics, Tracing, and Reporting

With the core loop functional, this phase adds the ability to measure performance and understand what the agent did.

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

## Phase 5: LLM Integration and Advanced Evaluation

This final phase integrates a real LLM and adds the more nuanced qualitative evaluation.

1.  **Implement an LLM-Powered Agent**:
    -   Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   Use `reqwest` to communicate with an external LLM API (e.g., OpenAI, Anthropic).
    -   Implement the logic for prompt engineering: serialize the observation and conversation history into a prompt for the LLM.
    -   Parse the LLM's response (likely JSON for function calling) back into an `AgentAction`.

2.  **Implement LLM-as-a-Judge**:
    -   Write the functionality to take a completed execution trace and submit it to a powerful "judge" LLM.
    -   Develop the rubric and prompt templates for the judge to score aspects like reasoning, efficiency, and robustness.
    -   Integrate these qualitative scores into the final report.

3.  **Expand `SolanaBench`**:
    -   Curate a comprehensive suite of test cases covering all the capability areas (T1-T5) defined in `IDEA.md`.
    -   Include simple "unit tests," multi-step "integration tests," and "adversarial" edge cases.