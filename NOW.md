# NOW: Advanced Observability (OpenTelemetry)

**Main Goal:** Implement Phase 5 of the master plan. The focus is on instrumenting the framework with OpenTelemetry to enable deep performance analysis and standardized observability.

**Immediate Tasks:**

1.  **Add Dependencies (`reev-runner`):**
    *   Integrate `tracing`, `opentelemetry`, `opentelemetry_sdk`, and a console exporter like `opentelemetry-stdout` into `Cargo.toml`.

2.  **Initialize Tracing Pipeline (`reev-runner`):**
    *   In `main.rs`, create a new function responsible for setting up the global OTel tracer and `tracing` subscriber.
    *   This pipeline will be configured to export traces directly to the console for immediate verification.
    *   Ensure the pipeline is shut down gracefully on application exit.

3.  **Instrument Codebase (`reev-lib`, `reev-runner`):**
    *   Add `#[tracing::instrument]` macros to key functions (`run_evaluation_loop`, `SolanaEnv::reset`, `SolanaEnv::step`).
    *   Add contextual attributes to the spans (e.g., `benchmark.id`, `step_number`, `tool_name`).
    *   Record important outcomes like errors or transaction signatures as `events` on the spans.

4.  **Verify OTel Output:**
    *   Run a benchmark and confirm that structured, machine-readable trace data is printed to the console, showing the hierarchy and duration of operations.