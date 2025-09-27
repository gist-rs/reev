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
cargo run -p reev-runner -- <PATH_TO_BENCHMARK> [--agent <AGENT_NAME>]
```

### Examples

*   **Deterministic Agent (Default):**
    If the `--agent` flag is omitted, the runner defaults to the `deterministic` agent, which provides the ground truth.
    ```sh
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml
    ```

*   **Gemini Agent:**
    To run the benchmark using a specific model like Gemini, provide its name.
    ```sh
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent gemini-2.5-pro
    ```

*   **Local Agent:**
    To run against a locally-served model, use the `local` agent name.
    ```sh
    cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent local
    ```

## Testing

The tests for this crate are split into two main categories to ensure both the correctness of the scoring logic and the validity of the benchmark files themselves.

### Running All Tests

To execute all tests within the `reev-runner` package, use the following command from the workspace root:

```sh
cargo test -p reev-runner
```

### Scoring Logic Unit Test (`scoring_test.rs`)

This is a focused unit test to verify that the `calculate_score` function works as expected. It uses a small, fixed set of benchmarks to check for clear pass (`1.0`) and fail (`0.0`) scenarios.

To run only this test:
```sh
cargo test -p reev-runner --test scoring_test
```

### Benchmark Integration Test (`benchmarks_test.rs`)

This is a sanity-check test that dynamically discovers and runs **every benchmark file** in the `benchmarks/` directory. For each benchmark, it simulates a "perfect agent" and asserts that the final score is `1.0`.

This test is crucial for ensuring that all benchmarks are correctly configured and are "winnable." If this test fails, it indicates a problem with the benchmark's definition, not an agent's performance.

To run only this test:
```sh
cargo test -p reev-runner --test benchmarks_test
```

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).