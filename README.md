# reev 🪸

**Re-Eval: A Rust-native framework for the reproducible evaluation of Solana LLM agents.**

---

## Summary

`reev` is a framework for the reproducible evaluation of Large Language Model (LLM) agents designed to operate on the Solana blockchain. Traditional LLM benchmarks are insufficient for agents that perceive, plan, and act within dynamic, high-stakes environments. This framework provides the necessary tools and methodologies to assess agent behavior in a rigorous, verifiable, and standardized manner.

The architecture is grounded in the principles of the Gymnasium API but implemented as a native Rust framework to ensure performance, type safety, and seamless integration with the Solana ecosystem.

### Core Principles

-   **Reproducibility**: The primary goal. Every test run is hermetic, guaranteeing that a given benchmark will produce the exact same result every time.
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is treated as a managed, external service that the environment spawns, configures via RPC, and terminates for each test. This ensures a clean architectural boundary and prevents dependency conflicts.
-   **Gymnasium-Inspired API**: The agent-environment interaction is modeled via a standard Rust `trait` (`GymEnv`) inspired by the Gymnasium API, promoting a clear separation of concerns.

### Key Components

1.  **`reev-lib` (Core Library)**:
    *   **`SolanaEnv`**: A custom, hermetic evaluation environment that manages an external `surfpool` process. It handles state setup, transaction execution, and observation generation.
    *   **Agent Interface**: Defines a simple `Agent` trait and provides a `DummyAgent` that executes a pre-defined sequence of actions from a benchmark file.
    *   **Action Handlers**: A modular system for building different types of Solana transactions (e.g., `sol_transfer`, `spl_transfer`).
    *   **Benchmark Structs**: Rust types that define the structure of a `SolanaBench` YAML file, enabling strongly-typed parsing.

2.  **`reev-runner` (CLI Orchestrator)**:
    *   The command-line tool for loading and running benchmarks.
    *   Orchestrates the entire evaluation loop, from setting up the environment to calculating metrics and reporting results.

3.  **`SolanaBench` (Benchmark Suite)**:
    *   A suite of evaluation tasks defined in YAML files located in the `benchmarks/` directory.
    *   Each test case includes a declarative `initial_state`, a natural language `prompt`, and `ground_truth` criteria for success.

## Usage

To run a specific benchmark, use the `reev-runner` crate with the `--benchmark` flag.

### Prerequisites

You must have the `surfpool` local validator installed.

```bash
brew install txtx/taps/surfpool
```

### Example: Running the SPL-Token Transfer Benchmark

1.  **Navigate to the project root.**
2.  **Run the command:**

    ```bash
    cargo run -p reev-runner -- --benchmark benchmarks/001-sol-transfer.yml
    ```

3.  The runner will execute the following steps:
    *   Load and parse the `001-sol-transfer.yml` file.
    *   Instantiate the `DummyAgent` and `SolanaEnv`.
    *   Spawn a new, clean `surfpool start` process in the background.
    *   Wait for the validator to become responsive.
    *   Use RPC "cheatcodes" to set up the initial on-chain state (wallets, USDC mint, token accounts) as defined in the benchmark.
    *   Execute the agent-environment loop, where the `DummyAgent` performs the `spl_transfer` action.
    *   Calculate and display the final performance metrics (e.g., Task Success Rate).
    *   Print a detailed JSON execution trace for analysis.
    *   Cleanly shut down the `surfpool` process.
