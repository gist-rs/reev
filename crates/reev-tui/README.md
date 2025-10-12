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
-   **Switch Agents**: `Left`/`Right` arrow keys (Deterministic | Local | GLM 4.6 | Gemini)
-   **Run Selected Benchmark**: `r`
-   **Run All Benchmarks**: `a`
-   **Cycle Through Panels**: `Tab`
-   **Toggle Log Panel**: `l`
-   **Quit**: `q` or `Esc`

### Agent Selection

The TUI supports four different agent types:
- **Deterministic**: Ground truth agent with perfect instructions
- **Local**: Local LLM model (requires `reev-agent` service)
- **GLM 4.6**: OpenAI-compatible GLM API (requires `GLM_API_KEY` and `GLM_API_URL`)
- **Gemini**: Google Gemini model (requires `GEMINI_API_KEY`)

Note: GLM 4.6 tab appears grayed out when environment variables are not configured.
