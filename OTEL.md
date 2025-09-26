# OTEL.md: Advanced Observability Plan

This document outlines the plan to instrument the `reev` framework with standardized OpenTelemetry (OTel) traces, enabling advanced performance analysis. This work is considered a separate, advanced feature to be implemented after the core TUI functionality is complete.

---

## OpenTelemetry Integration

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