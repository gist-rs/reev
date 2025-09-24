# NOW: Foundational Reporting & Visualization

**Main Goal:** Implement the foundational layer for reporting and visualization as defined in Phase 4 of `PLAN.md`. The focus is on transforming the captured `ExecutionTrace` into structured, human-readable artifacts.

**Immediate Tasks:**

1.  **Define the Canonical `TestResult` Struct**:
    *   Create a new module `reev-lib/src/results.rs`.
    *   Define the top-level `TestResult` struct that will serve as the data model for all evaluation outputs. It will aggregate the test case definition, the final metrics (`QuantitativeScores`), and the full `ExecutionTrace`.

2.  **Implement Structured YAML Output**:
    *   In `reev-runner`, modify the main loop to construct the `TestResult` struct upon completion of a benchmark.
    *   Serialize this struct into a well-formatted YAML string and print it to the console. This creates the foundational machine-readable artifact for all future UI work.

3.  **Implement ASCII Tree Renderer**:
    *   Create a new "renderer" module within `reev-runner`.
    *   Implement the logic to traverse the `ExecutionTrace` within the `TestResult` struct.
    *   Render the trace as a human-readable ASCII tree to the console, providing immediate, intuitive feedback on the agent's performance.