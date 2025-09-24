# NOW: Refactoring to a Service-Oriented `surfpool` Architecture

**Main Goal:** Pivot away from library integration and refactor `SolanaEnv` to treat `surfpool` as a managed, external service. The environment will be responsible for spawning, managing, and communicating with the `surfpool start` process via its JSON-RPC API. This ensures a robust, clean separation of concerns.

**Immediate Tasks:**

1.  **Update `PLAN.md`**: Revise the master plan to reflect the new service-oriented architecture (Completed).
2.  **Clean Up Dependencies**: Remove all local `path` dependencies to `surfpool-core`, `surfpool-mcp`, and `surfpool-types` from `reev-lib/Cargo.toml`. Remove `async-trait`, `tokio`, etc., as the environment will now be synchronous.
3.  **Revert `GymEnv` to be Synchronous**: Remove `async_trait` and `async fn` from the `GymEnv` trait definition in `src/env.rs`.
4.  **Rewrite `SolanaEnv::reset`**:
    *   The function will be synchronous (`fn`).
    *   It will use `std::process::Command` to spawn `surfpool start` as a child process and store the `Child` handle.
    *   It will poll the validator's RPC endpoint (e.g., `http://127.0.0.1:8899`) until it becomes responsive.
    *   It will generate new, random `Keypair`s for each account in the benchmark's `initial_state` and store them locally in the `keypair_map`.
    *   It will construct and send raw JSON-RPC requests to the `surfnet_setAccount` cheatcode endpoint to create and fund each account on the validator.
5.  **Update `reev-runner`**:
    *   Ensure the `main` function is synchronous (remove `#[tokio::main]`).
    *   Update the evaluation loop to call the synchronous `env.reset()` and `env.step()` methods.
6.  **Verify End-to-End**: Run the `transfer-simple-001.yml` benchmark to confirm the new service-based setup works correctly.