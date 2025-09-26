# `reev-runner`

This crate is the command-line interface (CLI) and orchestrator for the `reev` evaluation framework. Its primary responsibility is to provide a simple, scriptable way to run benchmarks from the terminal.

## Role in the Workspace

`reev-runner` is a "thin binary" that acts as the entrypoint for non-interactive evaluation runs. It is designed for simplicity and is ideal for use in automated environments like CI/CD pipelines.

Its responsibilities are:
1.  Parsing command-line arguments using the `clap` crate to identify which benchmark file to run.
2.  Instantiating the `SolanaEnv` and `LlmAgent` from the `reev-lib` crate.
3.  Orchestrating the main evaluation loop by calling the core library functions.
4.  Capturing the `ExecutionTrace` and calculating the final metrics.
5.  Printing a summary report and the detailed trace to the console.

It contains no core evaluation logic itself; all of that resides in the `reev-lib` crate.

## Usage

To run a specific benchmark, provide the path to the benchmark file.

### Example

```bash
# Run the benchmark using the default deterministic agent (ground truth)
RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml

# Run the benchmark using the AI agent (e.g., Gemini)
RUST_LOG=info cargo run -p reev-runner -- --agent ai benchmarks/001-sol-transfer.yml
```

### Verifying Agent Execution

The `reev-runner` automatically starts the `reev-agent` service in the background and directs its logs to `logs/reev-agent.log`. You can verify which agent logic was executed by checking this file.

-   When running with `--agent ai`, the log will contain:
    ```
    INFO reev_agent::main: [reev-agent] Routing to AI Agent.
    INFO reev_agent::main: [reev-agent] Running AI agent with Gemini...
    ```

-   When running with the default `deterministic` agent, the log will contain:
    ```
    INFO reev_agent::main: [reev-agent] Routing to Deterministic Agent (mock=true).
    ```

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).
