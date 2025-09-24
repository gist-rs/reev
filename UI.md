# UI.md: Plan for Interactive TUI Cockpit

This document outlines the plan for creating a comprehensive and interactive user interface for running evaluations and visualizing the results of the `reev-benchmarks` suite. The goal is to evolve the framework from a simple command-line runner into a powerful, interactive "cockpit" for agent development and analysis.

## Core Concept: The Evaluation Cockpit

The TUI will serve as a mission control dashboard. It's not just a passive viewer for results but an interactive environment where a user can select an agent model, choose which tests to run, execute the evaluation, and then immediately dive into the results.

## Phased Implementation Plan

The development will proceed in three distinct phases, progressively enhancing the user experience from a machine-readable format to a fully interactive TUI.

### Phase 1: Data Foundation - Structured YAML Output

**Goal:** The `reev-runner` will produce a structured, machine-readable YAML file for each test case execution. This remains the foundational layer.

*   **Rationale:** This YAML file serves as the canonical, versionable representation of a test result. It's the single source of truth for all subsequent visualization layers and is ideal for CI/CD integration or programmatic analysis.

**Example YAML Output (`result-spl-transfer-001.yml`):**

```yaml
id: SPL-TRANSFER-SIMPLE-001
prompt: "Please send 15 USDC from my token account (USER_USDC_ATA) to the recipient's token account (RECIPIENT_USDC_ATA)."
final_status: Succeeded
metrics:
  task_success_rate: 1.0
trace:
  prompt: "Please send 15 USDC from my token account (USER_USDC_ATA) to the recipient's token account (RECIPIENT_USDC_ATA)."
  steps:
    - action:
        tool_name: spl_transfer
        parameters:
          from_pubkey: "USER_USDC_ATA"
          to_pubkey: "RECIPIENT_USDC_ATA"
          authority_pubkey: "USER_WALLET_PUBKEY"
          amount: 15000000
      info:
        error: null
      observation:
        last_transaction_status: Success
        last_transaction_error: null
        last_transaction_logs:
          - "Transaction confirmed: 3fLxJ9...eWtcm4X"
        account_states:
          USER_USDC_ATA:
            lamports: 2039280
            owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            # ... other account fields
          RECIPIENT_USDC_ATA:
            lamports: 2039280
            owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            # ... other account fields
      thought: null
```

### Phase 2: Human-Readable CLI Output - ASCII Tree View

**Goal:** Implement a renderer that transforms the structured YAML trace into a human-readable ASCII tree, similar to `cargo tree`.

*   **Rationale:** This provides immediate, intuitive feedback directly in the command line. This renderer will be used for both the simple CLI output and as a component within the TUI itself in Panel B.

### Phase 3: Interactive TUI Cockpit with `Ratatui`

**Goal:** Build a full-featured, interactive Terminal User Interface (TUI) for orchestrating runs and analyzing results.

*   **Rationale:** A TUI provides a much richer experience than static text, allowing for interactive exploration, deep inspection of failures, and a more professional and efficient development workflow.

---

### TUI Cockpit Detailed Design

The TUI will be built using the `ratatui` Rust library and will feature a dynamic layout composed of a header and three main panels.

**High-Level Layout:**

```
+-----------------------------------------------------------------------------+
| Gemini  Modelle [RUN] [SETTINGS]                                           |
+------------------------------------+----------------------------------------+
| Panel A: Test Case Navigator       | Panel B: Execution Trace View          |
|                                    |                                        |
| [✓] SPL-TRANSFER-SIMPLE-001        | ✓ SPL-TRANSFER-SIMPLE-001: SUCCEEDED   |
| [✗] NFT-TRANSFER-001               | |                                      |
| [ ] TRANSFER-SIMPLE-001            | +-- Step 1:                            |
|                                    | |   +-- ACTION: spl_transfer(...)      |
|                                    | |       +-- OBSERVATION: Failure       |
|                                    | |           +-- INFO: tx_sig: 5YF...   |
+------------------------------------+----------------------------------------+
| Panel C: Details Pane                                                       |
|                                                                             |
| Error: Transaction failed: RPC response error -32002:                       |
|        Instruction #1 Failed: custom program error: 0x1                     |
| Logs:                                                                       |
|   > Program log: "Error: Insufficient funds for transfer"                   |
|                                                                             |
+-----------------------------------------------------------------------------+
```

**Component Functionality:**

1.  **Header:**
    *   **Model Display:** Shows the name of the currently configured agent/LLM model (e.g., "Gemini", "DummyAgent").
    *   **`[RUN]` Button:** This is the primary action button. When triggered, it will:
        *   Identify the test cases selected in Panel A.
        *   Invoke the core `reev-lib` evaluation logic for the selected model against the selected tests.
        *   Stream results back to the UI, updating the status of tests in Panel A as they complete.
    *   **`[SETTINGS]` Button:** A placeholder for future functionality. This would open a new view or popup for configuring:
        *   Different agent models and their API keys.
        *   Environment settings.
        *   (Future) Adding new data sources or plugins.

2.  **Panel A (Test Case Navigator):**
    *   A scrollable list of all available benchmarks found in the `benchmarks/` directory.
    *   **Checkboxes:** Each test case will have a checkbox (`[ ]`, `[✓]`) allowing the user to select which tests to include in the next run.
    *   **Status Indicators:** After a run, this panel will update to show the final status of each test (e.g., `✔ SUCCEEDED`, `✗ FAILED`, `● RUNNING`).
    *   Selecting a completed test case in this panel populates Panels B and C with its results.

3.  **Panel B (Execution Trace View):**
    *   Displays the detailed ASCII tree (from Phase 2) for the test case selected in Panel A.
    *   This panel will be scrollable and allow the user to select individual lines (e.g., an `ACTION` or `OBSERVATION`).

4.  **Panel C (Details Pane):**
    *   A context-aware panel that displays detailed, un-truncated information for the item selected in Panel B.
    *   If an `ACTION` is selected, it shows the full parameters.
    *   If an `OBSERVATION` indicates a failure, it shows the full `last_transaction_error` and `last_transaction_logs`.

### Implementation Strategy

*   A new crate, `reev-tui`, will be created to house the `Ratatui` application.
*   The TUI will become the primary interactive entry point. It will directly call the `reev-lib` functions to execute evaluations and will manage the parsing of the resulting YAML traces to update its state. This provides a seamless, integrated experience.

---

### Phase 4: Observability - OpenTelemetry (OTel) Integration

**Goal:** Instrument the `reev-runner` to generate standardized OpenTelemetry traces for each benchmark run, complementing the semantic YAML logs.

**Rationale:**
While the YAML trace is excellent for human and LLM readability, OpenTelemetry is the industry standard for machine-analyzable observability. Integrating OTel provides:
-   **Performance Analysis:** Precisely measure the duration of every step, from agent "thinking" time to RPC call latency, allowing for the diagnosis of performance bottlenecks.
-   **Standardization:** OTel traces can be ingested by a wide array of open-source (Jaeger, Grafana) and commercial (Honeycomb, Datadog) observability platforms.
-   **Dependency Analysis:** Automatically visualizes the causal relationships between operations (e.g., this `step` was caused by this `agent_decision`).

**Implementation Strategy:**

1.  **Add Dependencies:** Introduce the `tracing`, `opentelemetry`, and related crates to `reev-runner`.
2.  **Setup OTel Exporter:** Configure the runner to export traces. A simple starting point is to export to the console in OTLP format or directly to a local Jaeger instance.
3.  **Instrument the Code:** Use `#[tracing::instrument]` macros and manual span creation to map the evaluation flow to OTel concepts.

**Mapping `reev` Concepts to OpenTelemetry:**

| `reev` Concept         | OpenTelemetry (OTel) Concept         | OTel Attributes & Events                                                                  |
| ---------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------- |
| An entire benchmark run | **Root Span** (`reev.run`)           | `benchmark.id`, `benchmark.file_path`                                                     |
| A single `TraceStep`   | A **Child Span** (`reev.step`)       | `step.number`                                                                             |
| Agent's decision logic | A **Span** within the step (`agent.decision`) | `agent.tool_name`, `agent.parameters`                                                     |
| RPC call to validator  | A **Span** within the step (`solana.rpc`) | `rpc.method`, `transaction.signature`                                                     |
| A transaction error    | An **Event** on the span (`exception`) | `exception.type`, `exception.message`                                                     |
| Transaction logs       | **Events** on the span (`log`)       | `log.message`                                                                             |

By generating both the semantic YAML and the operational OTel trace, `reev` will provide a best-in-class solution for both high-level analysis and deep, performance-oriented debugging.