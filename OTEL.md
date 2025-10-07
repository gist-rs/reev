# OTEL.md: Advanced Observability Plan

## 📋 Current Status: Future Enhancement

This document outlines the plan to instrument the `reev` framework with standardized OpenTelemetry (OTel) traces, enabling advanced performance analysis. The framework is currently production-ready with comprehensive logging and monitoring. OpenTelemetry integration is planned as a future enhancement to provide deeper insights into agent behavior and system performance.

**Current State**: The framework uses `tracing` for structured logging and has basic health monitoring. OpenTelemetry integration is planned for Phase 17+ development.

---

## OpenTelemetry Integration

**Goal:** Instrument the framework to emit standardized OpenTelemetry (OTel) traces for advanced performance analysis and professional observability integration.

**Rationale:**
-   **Performance Analysis**: OTel provides industry-standard visualization for latency analysis, performance bottleneck identification, and system behavior insights.
-   **Professional Integration**: OTel traces can be ingested by any compatible backend (Jaeger, Honeycomb, Datadog), integrating `reev` into enterprise observability ecosystems.
-   **Agent Behavior Analysis**: Deep insights into agent decision-making processes, tool selection patterns, and execution performance.
-   **System Optimization**: Identify bottlenecks in surfpool simulation, instruction generation, and scoring algorithms.

**Current Observability Stack:**
- **Structured Logging**: Comprehensive `tracing` integration with log levels (debug, info, warn, error)
- **Health Monitoring**: Surfpool health checks and service status monitoring
- **Performance Metrics**: Benchmark execution times, scoring results, and agent performance data
- **Database Persistence**: SQLite database with detailed execution traces and results

**Planned OpenTelemetry Integration:**
| `reev` Concept         | OpenTelemetry (OTel) Concept         | OTel Attributes & Events                               |
| ---------------------- | ------------------------------------ | ------------------------------------------------------ |
| An entire benchmark run| **Root Span** (`reev.run`)           | `benchmark.id`, `benchmark.type`, `agent.model`       |
| A single `TraceStep`   | A **Child Span** (`reev.step`)       | `step.number`, `step.type`                           |
| Agent's decision logic | A **Span** within the step (`agent.decision`) | `agent.tool_name`, `agent.parameters`, `agent.reasoning` |
| Protocol execution     | A **Span** within the step (`protocol.execute`) | `protocol.name`, `instruction.count`, `execution.time` |
| RPC call to validator  | A **Span** within the step (`solana.rpc`) | `rpc.method`, `transaction.signature`, `simulation.time` |
| Scoring calculation    | A **Span** within the step (`reev.score`)   | `score.instruction`, `score.execution`, `score.final` |
| Transaction error    | An **Event** on the span (`exception`) | `exception.type`, `exception.message`, `error.details`   |