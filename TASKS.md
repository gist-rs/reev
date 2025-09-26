# TASKS.md: Development Roadmap

This document provides a detailed, actionable checklist for the development of the `reev` project.

---

## Phase 10: TUI Interaction and `reev-runner` Integration [COMPLETED]

**Goal:** Transform the TUI from a static prototype into a fully interactive tool that can discover, run, and display the results of benchmarks.

-   [x] **Task 10.1: Dynamic Benchmark Discovery**
-   [x] **Task 10.2: `reev-runner` as a Library**
-   [x] **Task 10.3: Execute Benchmarks from TUI**
-   [x] **Task 10.4: Display Live Results**
-   [x] **Enhancements:** Added "Run All" feature and improved UI layout.

---

## Phase 11: Comparative AI Agent Integration [COMPLETED]

**Goal:** Integrate a `rig`-based AI agent into the `reev-agent` service to enable comparative evaluation against the deterministic agent.

-   [x] **Task 11.1: Dual-Agent Routing**: Implemented routing in `reev-agent` to switch between the deterministic agent (`?mock=true`) and the AI agent.
-   [x] **Task 11.2: Define `rig` Tools**: Created `SolTransferTool` and `SplTransferTool` that implement the `rig::Tool` trait.
-   [x] **Task 11.3: AI Agent Implementation**: Used `rig` to build an agent that can query an LLM and use the defined tools to generate instructions.
-   [x] **Task 11.4: Runner Integration**: Updated `reev-runner` and `reev-tui` to select between `deterministic` and `ai` agents.

---

## Phase 12: Advanced TUI Controls [CURRENT]

**Goal:** Enhance the TUI to provide granular control over agent selection and execution.

-   [x] **Task 12.1: Agent Selection Tabs**:
    -   [x] Implemented a tabbed interface in the TUI header for selecting the agent (`Deterministic`, `Gemini`, `Local`).
    -   [x] The selected tab now determines which agent configuration is passed to the runner.
-   [x] **Task 12.2: Concurrency Management**:
    -   [x] Disabled the agent selection tabs and run controls while a benchmark is executing.
    -   [x] Prevented new benchmark runs from being started if one is already in progress.
-   [x] **Task 12.3: Agent Log Viewer**:
    -   [x] Added a new panel to the TUI to display the live output from the `reev-agent.log` file.
    -   [x] Added a keybinding (`l`) to toggle the visibility of the log panel.
-   [x] **Task 12.4: Enhanced Keybindings**:
    -   [x] Added `Enter` as a key to run benchmarks.
    -   [x] Resolved all keybinding conflicts.