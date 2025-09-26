# reev-tui: The Reev Interactive Cockpit

`reev-tui` provides an interactive Terminal User Interface (TUI) for discovering, running, and analyzing `reev` evaluation benchmarks. It acts as a visual frontend for the `reev-runner` engine.

## Features

-   **Automatic Benchmark Discovery**: Scans the `benchmarks/` directory at startup and lists all available test cases.
-   **Interactive Execution**: Run any selected benchmark directly from the UI by pressing a key.
-   **Asynchronous Operations**: Benchmarks run in the background, ensuring the UI remains responsive.
-   **Live Status Updates**: See the status of each benchmark (`PENDING`, `RUNNING`, `SUCCEEDED`, `FAILED`) update in real-time.
-   **Detailed Trace Views**: Once a benchmark completes, its full execution trace is displayed for immediate analysis.

## Running the TUI

To run the TUI, execute the following command from the **root of the `reev` workspace**:

```sh
RUST_LOG=info cargo run -p reev-tui
```

It is important to run this from the workspace root so that the TUI can correctly locate the `benchmarks/` directory.

### Keybindings

-   **Navigate Benchmarks**: `Up`/`Down` arrow keys or `k`/`j`
-   **Run Selected Benchmark**: `r`
-   **Cycle Through Panels**: `Tab`
-   **Quit**: `q` or `Esc`
