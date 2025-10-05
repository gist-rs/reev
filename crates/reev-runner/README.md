# `reev-runner`

This crate is the command-line interface (CLI) and orchestrator for the `reev` evaluation framework. Its primary responsibility is to provide a simple, scriptable way to run benchmarks from the terminal.

## Role in the Workspace

`reev-runner` is a "thin binary" that acts as the entrypoint for non-interactive evaluation runs. It is designed for simplicity and is ideal for use in automated environments like CI/CD pipelines.

Its responsibilities are:
1.  Parsing command-line arguments using the `clap` crate to identify the benchmark path and the selected agent.
2.  Instantiating the `SolanaEnv` and `LlmAgent` from the `reev-lib` crate.
3.  Orchestrating the main evaluation loop by calling the core library functions.
4.  Capturing the `ExecutionTrace` and calculating the final metrics.
5.  Printing a summary report and the detailed trace to the console.

It contains no core evaluation logic itself; all of that resides in the `reev-lib` crate.

## Usage

To run a specific benchmark, provide the path to the benchmark file. You can select the agent to use with the `--agent` flag.

### Command Structure

```sh
RUST_LOG=info cargo run -p reev-runner -- <PATH_TO_BENCHMARK> [--agent <AGENT_NAME>]
```

### Examples

*   **Deterministic Agent (Default):**
    If the `--agent` flag is omitted, the runner defaults to the `deterministic` agent, which provides the ground truth.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml
    ```

*   **Gemini Agent:**
    To run the benchmark using a specific model like Gemini, provide its name.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent gemini-2.5-pro
    ```

*   **Local Agent:**
    To run against a locally-served model, use the `local` agent name.
    ```sh
    RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent local
    ```

## Testing

**Core Principle:** All tests in this crate run against a `surfpool` instance, which is a high-speed, in-memory fork of the Solana mainnet. This allows tests to interact with the *real, deployed* versions of on-chain programs (like the SPL Token program or Jupiter) without any program logic mocking. This ensures that a passing test is a strong signal of real-world viability.

The tests for this crate are split into three main categories to ensure both the correctness of the scoring logic, the validity of the benchmark files, and the end-to-end AI agent integration.

### Running All Tests

To execute all tests within the `reev-runner` package, use the following command from the workspace root:

```sh
RUST_LOG=info cargo test -p reev-runner
```

To see detailed log output for each test case as it runs, add the `--nocapture` flag (note the extra `--`):

```sh
RUST_LOG=info cargo test -p reev-runner -- --nocapture
```

### Scoring Logic Unit Test (`scoring_test.rs`)

This is a focused unit test to verify that the `calculate_score` function works as expected. It uses a small, fixed set of benchmarks to check for clear pass (`1.0`) and fail (`0.0`) scenarios.

To run only this test:
```sh
RUST_LOG=info cargo test -p reev-runner --test scoring_test
```

### Benchmark Integration Test (`benchmarks_test.rs`)

This is a sanity-check test that dynamically discovers and runs **every solvable benchmark file** in the `benchmarks/` directory. For each benchmark, it simulates a "perfect agent" and asserts that the final score is `1.0`.

This test is crucial for ensuring that all benchmarks are correctly configured and are "winnable." If this test fails, it indicates a problem with the benchmark's definition, not an agent's performance.

To run only this test:
```sh
RUST_LOG=info cargo test -p reev-runner --test benchmarks_test
```

To see the detailed log output for each benchmark case, which is very useful for debugging, add the `--nocapture` flag:
```sh
RUST_LOG=info cargo test -p reev-runner --test benchmarks_test -- --nocapture
```

### Deterministic Agent Tests (`deterministic_agent_test.rs`)

**Phase 14 - End-to-End Deterministic Agent Tests**: These tests validate the core framework functionality without external LLM dependencies. They use predefined instructions to ensure the system works correctly end-to-end.

**Key Features:**
- **Complete Lifecycle**: Orchestrates the full evaluation pipeline including environment setup, instruction execution, and scoring
- **Perfect Score Validation**: All deterministic tests should achieve a perfect score of 1.0
- **Dynamic Test Generation**: Uses `rstest` to automatically loop through all benchmark files
- **Match-Based Logic**: Clean, idiomatic Rust code using `match` expressions instead of if-else chains
- **Port Cleanup**: Automatically cleans up reev-agent processes before starting tests

**Running Deterministic Agent Tests:**

To run all deterministic tests (automatically tests all benchmarks):
```sh
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test -- --nocapture
```

To run only the Jupiter integration test:
```sh
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test test_deterministic_agent_jupiter_swap_integration -- --nocapture
```

### LLM Agent Tests (`llm_agent_test.rs`)

**Phase 14 - End-to-End LLM Agent Tests**: These tests validate the complete framework's ability to evaluate real AI agents that call external LLM services. They demonstrate the full loop from runner to environment to AI agent to LLM and back.

**Key Features:**
- **Real AI Integration**: Tests against actual AI models (local models, Gemini, etc.) with real token usage
- **Smart Model Selection**: Automatically detects API keys and falls back to local models when unavailable
- **Dynamic Test Generation**: Uses `rstest` to automatically loop through all benchmark files  
- **Intelligent Scoring**: Different score thresholds based on benchmark complexity
- **Port Cleanup**: Automatically cleans up reev-agent processes to ensure clean test environment
- **DRY Architecture**: Single comprehensive test functions eliminate code duplication

**Prerequisites:**
```sh
# Install and start surfpool
brew install txtx/taps/surfpool
surfpool

# Configure .env file for AI models
# GOOGLE_API_KEY="your-google-api-key"  # For Gemini
# or start local LLM server on localhost:1234 (e.g., LM Studio)
```

**Running LLM Agent Tests:**

To run all LLM tests (automatically tests all benchmarks with intelligent scoring):
```sh
RUST_LOG=info cargo test -p reev-runner --test llm_agent_test -- --nocapture
```

**Expected Output:**
The tests demonstrate:
- Automatic service startup and health checks with port cleanup
- Smart model selection (local vs cloud based on API key availability)
- Real AI model processing with token usage tracking
- Tool recognition and execution (sol_transfer, spl_transfer, jupiter_swap, jupiter_lend_deposit)
- Intelligent scoring thresholds based on benchmark complexity:
  - Simple transfers: threshold 0.8
  - Jupiter swap: threshold 0.3
  - Jupiter lend operations: threshold 0.4
  - Complex 3-step operations: threshold 0.2
- Graceful handling of AI agent execution issues
- Complete evaluation pipeline validation

This comprehensive test suite serves as **the definitive proof** that the `reev` framework can successfully evaluate AI agents on a wide range of on-chain tasks, from simple transfers to complex DeFi operations, with automatic port management and intelligent scoring that adapts to benchmark complexity.

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).
