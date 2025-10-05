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

### AI Agent Integration Test (`ai_agent_test.rs`)

**Phase 14 - End-to-End AI Agent Integration Test**: This is the ultimate validation test that demonstrates the complete `reev` framework can successfully evaluate real, capable on-chain AI agents. It validates the entire loop from runner to environment to agent to LLM and back.

**Key Features:**
- **Complete Lifecycle**: Orchestrates the full evaluation pipeline including service startup, environment setup, AI agent execution, and scoring
- **Real AI Integration**: Tests against actual AI models (Gemini, local models) with real token usage and tool execution
- **Complex Benchmark**: Uses the Jupiter Swap benchmark (`100-jup-swap-sol-usdc.yml`) which represents a sophisticated DeFi operation
- **Robust Error Handling**: Gracefully handles AI agent tool execution issues and provides detailed feedback
- **Infrastructure Validation**: Proves the framework can evaluate AI agents on complex on-chain tasks

**Prerequisites:**
```sh
# Install and start surfpool
brew install txtx/taps/surfpool
surfpool

# Configure .env file for AI models
# GOOGLE_API_KEY="your-google-api-key"  # For Gemini
# or start local LLM server on localhost:1234
```

**Running the AI Agent Tests:**

To run the AI agent integration test:
```sh
RUST_LOG=info cargo test -p reev-runner --test ai_agent_test -- --nocapture
```

To run only the AI agent test:
```sh
RUST_LOG=info cargo test -p reev-runner --test ai_agent_test test_ai_agent_jupiter_swap_integration -- --nocapture
```

To run only the deterministic agent comparison test:
```sh
RUST_LOG=info cargo test -p reev-runner --test ai_agent_test test_deterministic_agent_jupiter_swap_integration -- --nocapture
```

**Expected Output:**
The test demonstrates:
- AI agent service startup and health checks
- Real AI model processing (Gemini 2.0 Flash with ~1,800 tokens)
- Tool recognition and execution attempts
- Complete evaluation pipeline validation
- Graceful handling of AI agent execution issues

This test serves as **the definitive proof** that the `reev` framework can successfully evaluate AI agents on complex on-chain tasks, making it ready for production AI agent evaluation workflows.

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).
