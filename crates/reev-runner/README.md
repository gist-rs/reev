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

### Available Agents

| Agent Name | Description | Environment Variables |
|------------|-------------|----------------------|
| `deterministic` | Default agent with predefined actions | None |
| `local` | Local LLM agent for custom models | None |
| `glm-4.6` | GLM 4.6 general purpose model | `GLM_API_KEY`, `GLM_API_URL` |
| `glm-4.6-coding` | GLM 4.6 specialized for coding tasks | `GLM_CODING_API_KEY`, `GLM_CODING_API_URL` |
| `gemini-2.5-flash-lite` | Google's Gemini 2.5 Flash Lite model | `GEMINI_API_KEY` |

### Command Structure

```sh
RUST_LOG=info cargo run -p reev-runner -- <PATH_TO_BENCHMARK> [--agent <AGENT_NAME>] [--shared-surfpool]
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
    To run using GLM 4.6 model with OpenAI-compatible API, set the required environment variables and use the `glm-4.6` agent name.
    ```sh
    export GLM_API_KEY="your-glm-api-key"
    export GLM_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
    ```
    > **Note:** Both `GLM_API_KEY` and `GLM_API_URL` environment variables must be set for GLM 4.6 to work.

*   **GLM 4.6 Coding Agent:**
    For coding-specific tasks, use the GLM 4.6 Coding variant:
    ```sh
    export GLM_CODING_API_KEY="your-glm-coding-api-key"
    export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6-coding
    ```

*   **Gemini Agent:**
    To run using Google's Gemini model:
    ```sh
    export GEMINI_API_KEY="your-gemini-api-key"
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent gemini-2.5-flash-lite
    ```

*   **Shared vs Fresh Surfpool Mode:**
    Control whether to reuse existing service instances or create fresh ones for each run.
    ```sh
    # Use existing instances (faster, shared state)
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic --shared-surfpool
    
    # Create fresh instances (isolated, clean state)
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
    ```

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

*   **Run Tests for Specific Agents:**
    
    **E2E Run All Test (now accepts --agent parameter):**
    ```sh
    # Test all default agents (deterministic, local, glm-4.6)
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --nocapture
    
    # Test specific agent
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent deterministic -- --nocapture
    
    # Test local agent (requires reev-agent service running)
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent local -- --nocapture
    
    # Test GLM 4.6 agent (requires GLM_API_KEY and GLM_API_URL)
    export GLM_API_KEY="your-glm-api-key"
    export GLM_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent glm-4.6 -- --nocapture
    
    # Test GLM 4.6 Coding agent (requires GLM_CODING_API_KEY and GLM_CODING_API_URL)
    export GLM_CODING_API_KEY="your-glm-coding-api-key"
    export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent glm-4.6-coding -- --nocapture
    
    # Test Gemini agent (requires GEMINI_API_KEY)
    export GEMINI_API_KEY="your-gemini-api-key"
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent gemini-2.5-flash-lite -- --nocapture
    ```
    
    **Single Agent Tests (use --agent flag):**
    ```sh
    # Test with deterministic agent (default)
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --agent deterministic -- --nocapture
    
    # Test with local agent (requires reev-agent service)
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --agent local -- --nocapture
    
    # Test with GLM 4.6 agent (requires GLM_API_KEY and GLM_API_URL)
    export GLM_API_KEY="your-glm-api-key"
    export GLM_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --agent glm-4.6 -- --nocapture
    
    # Test with GLM 4.6 Coding agent (requires GLM_CODING_API_KEY and GLM_CODING_API_URL)
    export GLM_CODING_API_KEY="your-glm-coding-api-key"
    export GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --agent glm-4.6-coding -- --nocapture
    
    # Test with Gemini agent (requires GEMINI_API_KEY)
    export GEMINI_API_KEY="your-gemini-api-key"
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --agent gemini-2.5-flash-lite -- --nocapture
    ```
    
    **Single Benchmark Consistency Test:**
    ```sh
    # Test single benchmark consistency across agents
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test test_single_benchmark_consistency -- --nocapture
    ```

### Current Test Files (10 tests)
- `benchmarks_test.rs` - Comprehensive benchmark testing with surfpool integration
- `deterministic_agent_test.rs` - Deterministic agent validation
- `llm_agent_test.rs` - LLM agent integration tests
- `scoring_test.rs` - Scoring logic unit tests
- `surfpool_rpc_test.rs` - RPC connectivity validation
- `dependency_management_test.rs` - Service lifecycle management
- `database_ordering_test.rs` - Database consistency tests
- `shared_flow_converter_test.rs` - Flow serialization tests
- `e2e_run_all_test.rs` - End-to-end validation of "Run All" functionality

*   **Benchmark Sanity-Check Test (`benchmarks_test.rs`):**
    Ensures ALL benchmarks are solvable by different agents with surfpool integration.
    ```sh
    RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --nocapture
    
    # Run with specific agent
    cargo test -p reev-runner --test benchmarks_test -- --agent local -- --nocapture
    ```

*   **End-to-End Run All Test (`e2e_run_all_test.rs`):**
    Validates the "Run All" functionality by testing multiple agents in both shared and fresh surfpool modes.
    **Now accepts `--agent` parameter!** If no agent specified, defaults to `["deterministic", "local", "glm-4.6"]`.
    
    ```sh
    # Test all default agents (deterministic, local, glm-4.6)
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --nocapture
    
    # Test single benchmark consistency (uses deterministic agent)
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test test_single_benchmark_consistency -- --nocapture
    
    # Test specific agents (supports all available agents)
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent glm-4.6-coding -- --nocapture
    RUST_LOG=info cargo test -p reev-runner --test e2e_run_all_test -- --agent gemini-2.5-flash-lite -- --nocapture
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
For master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).
