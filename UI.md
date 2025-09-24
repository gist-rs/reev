# UI.md: Plan for Reporting and Visualization

This document outlines the phased development plan for creating a rich user interface for visualizing and analyzing the results of `reev-benchmarks` runs. The goal is to provide multi-layered feedback, catering to different needs from quick command-line summaries to a fully interactive TUI cockpit.

---

## Phase 1: Data Foundation - Structured YAML Output

**Goal:** The `reev-runner` will produce a structured, machine-readable YAML file for each test case execution. This is the foundational artifact for all subsequent UI work.

**Rationale:**
-   **Single Source of Truth:** The YAML file serves as the canonical, versionable representation of a test result.
-   **Machine Readability:** Provides a stable format for programmatic analysis, CI/CD integration, and as a context source for LLM-driven analysis (RAG).

**Example YAML Output (`result-spl-transfer-001.yml`):**
```yaml
id: SPL-TRANSFER-SIMPLE-001
prompt: "Please send 15 USDC from my token account..."
final_status: Succeeded
metrics:
  task_success_rate: 1.0
trace:
  prompt: "Please send 15 USDC from my token account..."
  steps:
    - action:
        tool_name: spl_transfer
        parameters: { from_pubkey: "...", to_pubkey: "...", amount: 15000000 }
      info: { error: null }
      observation:
        last_transaction_status: Success
        last_transaction_error: null
        last_transaction_logs: ["Transaction confirmed: 3fLx..."]
        account_states: { ... } # A map of final account states
      thought: null
```

---

## Phase 2: Human-Readable CLI Output - ASCII Tree View

**Goal:** Implement a renderer that transforms the structured YAML trace into a human-readable ASCII tree, similar to `cargo tree`, for immediate CLI feedback.

**Rationale:**
-   **Quick Analysis:** Allows developers to quickly understand the agent's actions and the environment's responses without leaving the terminal.
-   **Debugging:** Makes it easy to spot where an agent's logic deviated or which tool call failed.

**Example ASCII Output:**
```
✓ SPL-TRANSFER-SIMPLE-001: SUCCEEDED
|
└─ Step 1:
   ├─ ACTION: spl_transfer(from_pubkey: "...", to_pubkey: "...", amount: 15000000)
   └─ OBSERVATION: Success
      └─ INFO: Transaction confirmed: 3fLx...
```

---

## Phase 3: Interactive Deep-Dive - `Ratatui` TUI Cockpit

**Goal:** Build a full-featured, interactive Terminal User Interface (TUI) for orchestrating runs and analyzing results.

**High-Level Layout:**
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Gemini 1.5 Pro                                       [R]UN  [S]ETTINGS │
├───────────────────────────┬──────────────────────────────────────────────┤
│ A: Benchmark Navigator    │ B: Execution Trace View                      │
│                           │                                              │
│ [✔] SPL-TRANSFER-001      │ ✔ SPL-TRANSFER-SIMPLE-001: SUCCEEDED         │
│ [✗] NFT-TRANSFER-001      │ │                                            │
│ [ ] TRANSFER-SIMPLE-001   │ └─ Step 1:                                  │
│                           │    ├─ ACTION: spl_transfer(...)             │
│                           │    └─ OBSERVATION: Failure                  │
├───────────────────────────┴──────────────────────────────────────────────┤
│ C: Details Pane                                                          │
│ > Error: Transaction failed: RPC response error -32002                   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Component Functionality:**
-   **Header:** Displays the active agent/model. The `[RUN]` button executes the selected benchmarks. The `[SETTINGS]` button will manage configurations.
-   **Panel A (Benchmark Navigator):** A scrollable list of all available benchmarks, with checkboxes for selecting which tests to run. Status icons (`✔`, `✗`) are updated live.
-   **Panel B (Execution Trace View):** Displays the ASCII tree (from Phase 2) for the result selected in Panel A.
-   **Panel C (Details Pane):** A context-aware panel showing detailed, un-truncated information (e.g., full error messages, transaction logs) for the line selected in Panel B.

**Implementation:** This will be built in a new `reev-tui` crate using the `ratatui` library.

---

## Phase 4: Advanced Observability - OpenTelemetry Integration

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces, enabling advanced performance analysis.

**Rationale:**
-   **Performance Analysis:** OTel is the industry standard for visualizing latency and identifying performance bottlenecks (e.g., agent "thinking" time vs. RPC latency).
-   **Standardization:** OTel traces can be ingested by any compatible backend (Jaeger, Honeycomb, Datadog), integrating `reev` into professional observability ecosystems.

**Mapping `reev` Concepts to OpenTelemetry:**
| `reev` Concept         | OpenTelemetry (OTel) Concept         | OTel Attributes & Events                               |
| ---------------------- | ------------------------------------ | ------------------------------------------------------ |
| An entire benchmark run| **Root Span** (`reev.run`)           | `benchmark.id`, `benchmark.file_path`                  |
| A single `TraceStep`   | A **Child Span** (`reev.step`)       | `step.number`                                          |
| Agent's decision logic | A **Span** within the step (`agent.decision`) | `agent.tool_name`, `agent.parameters`                  |
| RPC call to validator  | A **Span** within the step (`solana.rpc`) | `rpc.method`, `transaction.signature`                  |
| A transaction error    | An **Event** on the span (`exception`) | `exception.type`, `exception.message`                  |