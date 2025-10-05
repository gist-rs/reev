# reev 🪸

**Re-Eval: A Rust-native framework for the reproducible evaluation of Solana LLM agents.**

---

## Summary

`reev` is a framework for the reproducible evaluation of Large Language Model (LLM) agents designed to operate on the Solana blockchain. Traditional LLM benchmarks are insufficient for agents that perceive, plan, and act within dynamic, high-stakes environments. This framework provides the necessary tools and methodologies to assess agent behavior in a rigorous, verifiable, and standardized manner.

The architecture is grounded in the principles of the Gymnasium API but implemented as a native Rust framework to ensure performance, type safety, and seamless integration with the Solana ecosystem.

### Core Methodology: Real Programs, Controlled State

The entire framework is built on **`surfpool`**, a high-speed, in-memory fork of the Solana mainnet. This provides the best of both worlds:
- **Real-World Logic**: Agents interact with the actual, deployed mainnet versions of programs like Jupiter, Kamino, or the SPL Token Program. There is no program mocking, so a successful action is a strong indicator of real-world viability.
- **Controlled Environment**: While program logic is real, the *state* (e.g., account balances, token ownership) is precisely controlled. Tests use RPC "cheat codes" to set up a specific initial on-chain state, ensuring every evaluation run is hermetic and reproducible.

### Core Principles

-   **Reproducibility**: The primary goal. Every test run is hermetic, guaranteeing that a given benchmark will produce the exact same result every time.
-   **Service-Oriented Environment**: The Solana test validator (`surfpool`) is treated as a managed, external service that the environment connects to and configures via RPC. This ensures a clean architectural boundary and prevents dependency conflicts.
-   **Gymnasium-Inspired API**: The agent-environment interaction is modeled via a standard Rust `trait` (`GymEnv`) inspired by the Gymnasium API, promoting a clear separation of concerns.

### Key Components

1.  **`reev-lib` (Core Library)**:
    *   **`SolanaEnv`**: A custom, hermetic evaluation environment that connects to an external `surfpool` process. It handles state setup, transaction execution, and observation generation.
    *   **Agent Interface**: Defines a simple `Agent` trait and provides an `LlmAgent` that can reason about prompts.
    *   **Benchmark Structs**: Rust types that define the structure of a benchmark YAML file, enabling strongly-typed parsing.

2.  **`reev-runner` (CLI Orchestrator)**:
    *   The command-line tool for loading and running benchmarks.
    *   Orchestrates the entire evaluation loop, from setting up the environment to calculating metrics and reporting results.

3.  **`reev-agent` (LLM Service)**:
    *   A standalone server that exposes an LLM's reasoning capabilities over an API.
    *   Can be configured to use different models (local, Gemini, etc.) and includes a deterministic agent for generating ground-truth instructions.

4.  **Benchmark Suite**:
    *   A suite of evaluation tasks defined in YAML files located in the `benchmarks/` directory.
    *   Each test case includes a declarative `initial_state`, a natural language `prompt`, and `ground_truth` criteria for success.

## Usage

The project has two primary command-line tools: `reev-runner` for running full benchmarks and `reev-agent` examples for making quick, direct API calls. Both tools use a consistent `--agent` flag to select the agent.

### Prerequisites

1.  **Install `surfpool`:**
    You must have the `surfpool` local validator installed and running in a separate terminal.
    ```bash
    brew install txtx/taps/surfpool
    surfpool
    ```

2.  **Install a Local LLM (Optional):**
    If you want to use the local agent, install [LM Studio](https://lmstudio.ai/) or another compatible server and download a model (e.g., `qwen`).

3.  **Setup `.env` File:**
    Create a `.env` file in the project root. This file configures the `reev-agent`.
    ```bash
    # The endpoint for the reev-agent server.
    LLM_API_URL="http://localhost:9090/gen/tx"

    # For AI agents, you may need an API key.
    # e.g., for Gemini
    # GOOGLE_API_KEY="your-google-api-key"
    ```

### 1. Running Benchmarks (`reev-runner`)

This is the standard tool for evaluating agent performance against a benchmark file.

**Command Structure:**
```sh
cargo run -p reev-runner -- <PATH_TO_BENCHMARK> [--agent <AGENT_NAME>]
```

**Examples:**

*   **Deterministic Agent (Default):**
    ```sh
    # The --agent flag is omitted, so it defaults to 'deterministic'
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml
    ```

*   **Gemini Agent:**
    ```sh
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent gemini-2.5-pro
    ```

*   **Local Agent:**
    ```sh
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent local
    ```

### 2. Testing Scenarios (`reev-agent` Examples)

This is a tool for a quick, isolated test of the agent server for a specific scenario, without the full benchmark environment.

**Command Structure:**
```sh
cargo run -p reev-agent --example <EXAMPLE_NAME> -- [--agent <AGENT_NAME>]
```

**Examples:**

*   **Deterministic Agent (Default):**
    ```sh
    cargo run -p reev-agent --example 001-sol-transfer
    ```

*   **Gemini Agent:**
    ```sh
    cargo run -p reev-agent --example 001-sol-transfer -- --agent gemini-2.5-pro
    ```

*   **Local Agent:**
    ```sh
    cargo run -p reev-agent --example 001-sol-transfer -- --agent local
    ```

### 3. AI Agent Integration Testing

**Phase 14 - End-to-End AI Agent Integration Test**: The ultimate validation that demonstrates the complete `reev` framework can successfully evaluate real, capable on-chain AI agents.

**Prerequisites:**
```sh
# Install and start surfpool
brew install txtx/taps/surfpool
surfpool

# Configure .env file for AI models
# GOOGLE_API_KEY="your-google-api-key"  # For Gemini
# or start local LLM server on localhost:1234 (e.g., LM Studio)
```

**Running AI Agent Integration Tests:**

The test suite is now split into two specialized test files for better organization and maintainability:

#### Deterministic Agent Tests
Tests that validate core framework functionality without LLM dependencies:
```sh
# Run all deterministic tests (loops through all benchmarks)
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test -- --nocapture

# Run specific deterministic test for Jupiter integration
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test test_deterministic_agent_jupiter_swap_integration -- --nocapture
```

#### LLM Agent Tests  
Tests that evaluate actual AI agents with external LLM services:
```sh
# Run all LLM tests (automatically loops through all benchmarks)
RUST_LOG=info cargo test -p reev-runner --test llm_agent_test -- --nocapture
```

**✅ Validation Results:**
- **Complete Pipeline**: Runner → Environment → Agent → LLM → Scoring loop working end-to-end
- **Real AI Integration**: Successfully tested with local models and cloud APIs
- **Complex DeFi Operations**: Jupiter Swap benchmark with sophisticated multi-instruction transactions
- **Robust Infrastructure**: Automatic port cleanup, service orchestration, and graceful error handling
- **Production Ready**: Framework proven to evaluate AI agents on complex on-chain tasks
- **DRY Architecture**: Single comprehensive test functions that automatically handle all benchmark files
- **Intelligent Scoring**: Different score thresholds based on benchmark complexity

This integration test serves as **the definitive proof** that the `reev` framework can successfully evaluate AI agents in production environments.
