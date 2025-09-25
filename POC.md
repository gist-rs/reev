# Proof of Concept: Mock `reev-agent` for End-to-End Testing

This document outlines the plan to create a standalone `reev-agent` crate that serves a mock endpoint. The primary goal is to establish a stable, end-to-end testing environment for the `reev-runner` and its scoring logic before integrating a real Large Language Model.

## Plan

1.  **Create `reev-agent` Crate**:
    *   Initialize a new binary crate named `reev-agent` within the `reev/crates` directory.
    *   Add it to the main workspace `Cargo.toml`.

2.  **Implement Mock HTTP Server with Axum**:
    *   Add `axum`, `tokio`, `serde`, and `serde_json` as dependencies to `reev-agent`.
    *   Create a simple `main` function to start an `axum` server.
    *   The server will listen on `http://localhost:9090`.

3.  **Implement the `/gen/tx` Endpoint**:
    *   Create a handler function for the `POST /gen/tx` route.
    *   This handler will be designed to always return a hardcoded, valid JSON response for the `001-sol-transfer.yml` benchmark.
    *   This removes any variability from the LLM and allows us to focus on the runner's integration and scoring pipeline.

4.  **Define Request and Response Structures**:
    *   Define Rust structs that match the JSON payload sent by `reev-lib`'s `LlmAgent` and the expected response structure. This ensures type safety.
    *   The response will be a JSON object containing a `result` field, which in turn contains a `text` field with the mocked `RawInstruction`.

5.  **Integrate with `reev-runner`**:
    *   Confirm that the `LLM_API_URL` in `reev-lib`'s `LlmAgent` defaults to `http://localhost:9090/gen/tx`. This ensures that when the runner is started without a custom URL, it will automatically connect to our new mock agent.

6.  **End-to-End Test Execution**:
    *   Start the new `reev-agent` server in one terminal.
    *   In another terminal, run the `reev-runner` with the `benchmarks/001-sol-transfer.yml` file.
    *   The expected outcome is a successful run, with the final score being `1.0`. This will validate that the entire pipeline, from benchmark parsing to agent communication, transaction execution, and scoring, is working correctly.