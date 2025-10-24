# `reev-runner`

This crate is the command-line interface (CLI) and orchestrator for the `reev` evaluation framework. Its primary responsibility is to provide a simple, scriptable way to run benchmarks from the terminal.

## Role in the Workspace

`reev-runner` is a "thin binary" that acts as the entrypoint for non-interactive evaluation runs. It is designed for simplicity and is ideal for use in automated environments like CI/CD pipelines.

Its responsibilities are:
1.  Parsing command-line arguments to identify the benchmark path and the selected agent.
2.  Instantiating the `SolanaEnv` and the appropriate agent from `reev-lib`.
3.  Orchestrating the main evaluation loop and capturing the results.
4.  Printing a summary report and a detailed execution trace to the console.

It contains no core evaluation logic itself; all of that resides in the `reev-lib` crate.

## Usage

To run a specific benchmark, provide the path to the benchmark YAML file. You can select the agent to use with the `--agent` flag.

### Command Structure

```sh
RUST_LOG=info cargo run -p reev-runner -- <PATH_TO_BENCHMARK> [--agent <AGENT_NAME>]
```

### Examples

*   **Deterministic Agent (Default):**
    If the `--agent` flag is omitted, the runner defaults to the `deterministic` agent.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml
    ```

*   **Cloud LLM Agent (e.g., Gemini):**
    To run the benchmark using a specific model, provide its name.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
    ```

*   **Local LLM Agent:**
    To run against a locally-served model, use the `local` agent name.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/115-jup-lend-mint-usdc.yml --agent local
    ```

*   **GLM 4.6 Agent:**
    To run using GLM 4.6 model with OpenAI-compatible API, set the required environment variables and use the `glm` agent name.
    ```sh
    export GLM_API_KEY="your-glm-api-key"
    export GLM_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm
    ```
    > **Note:** Both `GLM_API_KEY` and `GLM_API_URL` environment variables must be set for GLM 4.6 to work.

## Testing

**Core Principle:** All tests in this crate run against a `surfpool` instance, which is a high-speed, in-memory fork of the Solana mainnet. This allows tests to interact with the *real, deployed* versions of on-chain programs.

### Prerequisites

1.  **Install and run `surfpool`:**
    ```sh
    brew install txtx/taps/surfpool
    surfpool
    ```
2.  **For LLM Agent Tests:**
    *   Configure your `.env` file with the appropriate API keys (e.g., `GEMINI_API_KEY`).
    *   For GLM 4.6 tests, set both `GLM_API_KEY` and `GLM_API_URL` environment variables.
    *   Build and run the `reev-agent` service in a separate terminal:
        ```sh
        cargo run -p reev-agent
        ```

### Running the Test Suites

To see detailed log output for any test, add the `-- --nocapture` flag.

*   **Run All Tests:**
    ```sh
    RUST_LOG=info cargo test -p reev-runner
    ```

### Current Test Files (8 tests)
- `benchmarks_test.rs` - Comprehensive benchmark testing with surfpool integration
- `deterministic_agent_test.rs` - Deterministic agent validation
- `llm_agent_test.rs` - LLM agent integration tests
- `scoring_test.rs` - Scoring logic unit tests
- `surfpool_rpc_test.rs` - RPC connectivity validation
- `dependency_management_test.rs` - Service lifecycle management
- `database_ordering_test.rs` - Database consistency tests
- `shared_flow_converter_test.rs` - Flow serialization tests

*   **Benchmark Sanity-Check Test (`benchmarks_test.rs`):**
    Ensures ALL benchmarks are solvable by different agents with surfpool integration.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --nocapture
    
    # Run with specific agent
    cargo test -p reev-runner --test benchmarks_test -- --agent local -- --nocapture
    ```

*   **Deterministic Agent Test (`deterministic_agent_test.rs`):**
    Validates core framework functionality using predefined instructions.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test -- --nocapture
    ```

*   **LLM Agent Test (`llm_agent_test.rs`):**
    Validates the full AI agent pipeline by calling an external LLM service.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test llm_agent_test -- --nocapture
    ```

*   **Scoring Logic Unit Test (`scoring_test.rs`):**
    A focused unit test for the `calculate_score` function.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test scoring_test
    ```

*   **Surfpool RPC Test (`surfpool_rpc_test.rs`):**
    Validates basic RPC connectivity with the `surfpool` instance.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test surfpool_rpc_test -- --nocapture
    ```

---
For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).
