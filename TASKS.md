# TASKS.md: TUI Development Roadmap

This document provides a detailed, actionable checklist for the development of the interactive `reev-tui` cockpit.

---

## Phase 10: TUI Interaction and `reev-runner` Integration

**Goal:** Transform the TUI from a static prototype into a fully interactive tool that can discover, run, and display the results of benchmarks.

-   [ ] **Task 10.1: Dynamic Benchmark Discovery**
    -   [ ] Implement logic in `reev-tui/src/main.rs` to scan the `benchmarks/` directory at startup.
    -   [ ] Populate the "Benchmark Navigator" list with the discovered `.yml` files.
    -   [ ] The initial state for all discovered benchmarks should be "[ ] PENDING".

-   [ ] **Task 10.2: `reev-runner` as a Library**
    -   [ ] Refactor `reev-runner/src/main.rs` to move the core benchmark execution loop into a public function (e.g., `pub fn run_benchmark(path: &str) -> Result<ExecutionTrace>`).
    -   [ ] This will allow the `reev-tui` to call the runner's logic directly, capturing the structured output.
    -   [ ] **Note:** This is a significant refactoring. `main` should become a thin wrapper around this new library function.

-   [ ] **Task 10.3: Execute Benchmarks from TUI**
    -   [ ] In the `on_run` function in `reev-tui`, get the file path of the currently selected benchmark.
    -   [ ] Spawn a new thread to call the `reev_runner::run_benchmark` function.
    -   [ ] **Crucially**, use a channel (e.g., `std::sync::mpsc`) to send updates from the execution thread back to the TUI's main event loop to avoid blocking the UI.
    -   [ ] While a benchmark is running, its status in the navigator should change to `[...] RUNNING`.

-   [ ] **Task 10.4: Display Live Results**
    -   [ ] When the `run_benchmark` function completes, it will send the final `ExecutionTrace` back to the TUI.
    -   [ ] The TUI will receive this trace and update the application state for the completed benchmark.
    -   [ ] The benchmark's status in the navigator will update to `[✔] SUCCEEDED` or `[✗] FAILED`.
    -   [ ] The "Execution Trace View" and "Details Pane" will be populated with the data from the received trace.