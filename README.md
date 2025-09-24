# reev
🪸 Re-Eval: A Framework for the Reproducible Evaluation of LLM Agents

## Summary

`reev` is a framework for the reproducible evaluation of Large Language Model (LLM) agents, specifically those designed to operate on the Solana blockchain. Traditional LLM benchmarks are insufficient for agents that perceive, plan, and act within dynamic environments. This framework provides the necessary tools and methodologies to assess agent behavior in a rigorous, verifiable, and standardized manner.

The core of the project is based on the Gymnasium (a fork of OpenAI's Gym) API, which provides a standard interface for agent-environment interaction. This ensures that evaluations are reproducible and align with established AI research community standards.

### Key Components

1.  **Solana-Gym Environment (`SolanaEnv`)**:
    *   A custom, hermetic evaluation environment that simulates interaction with the Solana blockchain.
    *   Uses a local, ephemeral `solana-test-validator` for each test run to guarantee reproducibility and isolation from external network factors.
    *   Provides a standardized `step` and `reset` interface for the agent to submit actions (transactions) and receive observations (on-chain state changes).

2.  **Solana Agent Benchmark (`SolanaBench`)**:
    *   A suite of curated test cases defined in a machine-readable format (YAML/JSON).
    *   Each test case includes an initial on-chain state, a natural language prompt for the agent, and ground-truth assertions for verifying success.
    *   Tasks are designed to test a taxonomy of agent capabilities, from simple state comprehension to complex, multi-step reasoning and error handling.

3.  **Multi-Faceted Evaluation Harness**:
    *   An automated runner that orchestrates the entire evaluation process.
    *   Calculates a suite of quantitative metrics:
        *   **Task Success Rate (TSR):** Did the agent achieve the goal?
        *   **Tool Selection & Parameterization Accuracy:** Did the agent use the correct tools with the right parameters?
        *   **Gas Consumption Efficiency (GCE):** How economically did the agent operate?
    *   Incorporates a qualitative "LLM-as-a-Judge" assessment to score the agent's reasoning, planning, and adaptability.

4.  **Execution Trace Visualization**:
    *   Captures a detailed, hierarchical trace of the agent's entire thought process and interaction loop.
    *   Renders this trace as a human-readable ASCII tree, providing an "Explainable AI" (XAI) view for debugging, analysis, and building trust.

5.  **Rust-Native Implementation**:
    *   The entire framework is specified to be built in Rust for performance, type safety, and seamless integration with the Solana ecosystem and its tooling. Core concepts from Gymnasium are translated into idiomatic Rust traits and structs.

This framework aims to provide a comprehensive, rigorous, and transparent methodology for developing and verifying the capabilities of sophisticated LLM agents in high-stakes blockchain environments.

## Usage

To run a specific benchmark, use the `reev-runner` crate with the `--benchmark` flag.

### Example: Running the Simple SOL Transfer Benchmark

1.  **Navigate to the project root.**
2.  **Run the command:**

    ```bash
    surfpool start
    cargo run -p reev-runner -- --benchmark benchmarks/transfer-simple-001.yml
    ```

3.  The runner will:
    *   Load the specified benchmark file.
    *   Instantiate the `DummyAgent` and `SolanaEnv`.
    *   Start a local `surfpool` validator process.
    *   Set up the initial on-chain state via RPC.
    *   Execute the agent-environment loop until the task is complete.
    *   Calculate and display the final performance metrics.
    *   Cleanly shut down the validator process.
