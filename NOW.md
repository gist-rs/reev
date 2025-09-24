# NOW: Implementing Advanced Metrics and Reporting

**Main Goal:** With the core environment and action handlers in place, the focus now shifts to building out the framework's ability to measure and report on agent performance in detail. This phase is critical for moving beyond simple pass/fail checks to a nuanced understanding of agent behavior.

**Immediate Tasks:**

1.  **Implement Full Execution Tracing**:
    *   Enhance the `ExecutionTrace` struct and the runner loop to meticulously record every `AgentAction`, the resulting `AgentObservation`, and the `info` dictionary from each step. This creates a complete, auditable log of the agent-environment interaction.

2.  **Implement Advanced Quantitative Metrics**:
    *   **Tool Selection Accuracy (TSA)**: Implement the logic in `metrics.rs` to compare the agent's sequence of tool calls against the `expected_tool_calls` defined in the benchmark's ground truth.
    *   **Parameterization Accuracy (PA)**: For tool calls that were correctly selected, implement checks to verify that the parameters provided by the agent match the ground truth.

3.  **Implement ASCII Trace Visualization**:
    *   Create a new module or function responsible for rendering the `ExecutionTrace`.
    *   This renderer will traverse the trace and generate a human-readable ASCII tree that clearly visualizes the agent's actions and the environment's responses, as specified in `IDEA.md`.

4.  **Generate a Final Summary Report**:
    *   Modify the `reev-runner` to aggregate the metrics (TSR, TSA, PA, etc.) from all executed test cases.
    *   At the end of a run, print a comprehensive summary report to the console or a file, providing a high-level overview of the agent's performance.