# `reev-runner`

This crate is the command-line interface (CLI) and orchestrator for the `reev` evaluation framework. Its primary responsibility is to provide a simple, scriptable way to run benchmarks from the terminal.

## Role in the Workspace

`reev-runner` is a "thin binary" that acts as the entrypoint for non-interactive evaluation runs. It is designed for simplicity and is ideal for use in automated environments like CI/CD pipelines.

Its responsibilities are:
1.  Parsing command-line arguments using the `clap` crate to identify which benchmark file to run.
2.  Instantiating the `SolanaEnv` and `DummyAgent` from the `reev-lib` crate.
3.  Orchestrating the main evaluation loop by calling the core library functions.
4.  Capturing the `ExecutionTrace` and calculating the final metrics.
5.  Printing a summary report and the detailed trace to the console.

It contains no core evaluation logic itself; all of that resides in the `reev-lib` crate.

## Usage

To run a specific benchmark, provide the path to the benchmark YAML file as a positional argument.

**Note:** For now, you must manually start the `surfpool` validator in a separate terminal before running the runner. Automatic spawning is temporarily disabled to address a stability issue.

### Example

First, start the validator in a separate terminal:
```bash
surfpool start
```

Then, from the root of the workspace, run the simple SOL transfer benchmark:
```bash
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml
```

For the master project plan and more detailed architectural documentation, please see the main [repository `README.md`](../../README.md).