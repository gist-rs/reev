# NOW: TUI Interactivity

This document outlines the immediate development focus for the `reev` framework. The current goal is to transform the `reev-tui` from a static proof-of-concept into a fully interactive evaluation cockpit.

## Current Objective

The primary objective is to enable the TUI to discover and execute benchmarks using the `reev-runner`'s core logic, and then display the results in real-time. This involves a significant refactoring of the `reev-runner` to expose its functionality as a library.

## Action Plan

The detailed plan for this phase is broken down in `TASKS.md` and covers the following key areas:

1.  **Dynamic Benchmark Discovery**: The TUI must find all available benchmark files at startup.
2.  **`reev-runner` as a Library**: The runner's execution logic needs to be callable from other crates.
3.  **Asynchronous Execution**: Benchmarks must be run in a separate thread to keep the TUI responsive.
4.  **Live Result Display**: The TUI must update dynamically as benchmarks complete, showing the final status and detailed trace information.