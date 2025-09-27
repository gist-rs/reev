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

The tests for this crate, particularly for the scoring logic, can be run from the workspace root.

### Run All Tests

To execute all tests within the `reev-runner` package, use the following command:

```sh
cargo test -p reev-runner
```

### Run a Specific Test File

If you want to run a specific test file (e.g., to verify the scoring logic), you can target it by its filename:

```sh
cargo test -p reev-runner --test scoring_test
```

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).