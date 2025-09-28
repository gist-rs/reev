# NOW.md: Project Status and Next Steps

## High-Level Summary

The framework has undergone a significant and critical refactoring to unify behavior across all components. Previously, the test suite (`cargo test`), the TUI (`reev-tui`), and the CLI runner (`reev-runner`) had divergent logic for setting up benchmark environments, leading to inconsistent results. This has been resolved.

The project is now architecturally sound, consistent, and ready for the next phase of testing: evaluating a true AI agent.

---

## The Unified Architecture

1.  **Centralized Logic (`reev-lib`):** All complex, benchmark-specific setup logic (like deriving ATAs and using RPC cheat codes to fund them) now lives in `reev-lib/src/test_scenarios.rs`. This is the single source of truth.
2.  **Unified Environment (`SolanaEnv`):** The `SolanaEnv::reset` function now calls the centralized setup logic, guaranteeing that every component gets an identically prepared on-chain environment for any given benchmark.
3.  **Simplified Tests:** The test helpers in `reev-runner/tests/common/` have been stripped of all special-case logic, making them simpler and preventing divergence.
4.  **Realistic Agent Behavior:** The deterministic ("oracle") agent for the Jupiter swap benchmark now correctly calls the real Jupiter API to get a valid transaction.

---

## Current Status and Expected Outcomes

*   **TUI and `cargo test` are Consistent:** The behavior seen in the TUI is now perfectly mirrored by `cargo test`.
*   **The Swap Benchmark Fails Correctly (for the Deterministic Agent):** When running `100-jup-swap-sol-usdc.yml` with the **deterministic agent**, it will **fail**. This is the **correct and expected outcome**. It fails because a real transaction from the Jupiter API is built against mainnet's live state (e.g., liquidity pool balances), which is inconsistent with our local user's state (who only exists on the fork).
*   **A Path for AI is Clear:** This setup correctly frames the problem for an AI agent. The swap benchmark is a perfect example of a task that is **unsolvable by a simple agent**, which is a critical capability to measure.

---

## Next Task: End-to-End AI Agent Integration Test

With the framework now stable and consistent, the final piece of the validation puzzle is missing: an end-to-end test that proves a **real AI agent** can solve a complex benchmark.

### Why This is the Next Step

The `benchmarks_test.rs` suite validates the benchmarks themselves using a *mock* perfect instruction. The `reev-agent`'s deterministic handlers are now also consistent. However, we have not yet tested the full loop:

`TUI/Runner -> SolanaEnv -> LlmAgent -> reev-agent (AI mode) -> LLM -> reev-agent -> LlmAgent -> SolanaEnv -> Score`

The Jupiter Swap benchmark is the ideal candidate for this test because we know the deterministic agent cannot solve it. A passing grade for an AI agent would be a major validation of the entire framework.

### How to Implement the Test

1.  **Create a new test file:** `crates/reev-runner/tests/ai_agent_test.rs`.
2.  **Start the `reev-agent` Server:** The test will need to spawn the `reev-agent` process in the background, similar to how `surfpool_rpc_test.rs` does. This agent will run in its default AI mode (not mock/deterministic).
3.  **Instantiate an `LlmAgent`:** The test will create an instance of `LlmAgent` from `reev-lib`. This agent is configured via environment variables (`.env` file) to point to the `reev-agent` server we just started.
4.  **Run the Jupiter Swap Benchmark:** The test should specifically run the `100-jup-swap-sol-usdc.yml` benchmark.
5.  **Assert Success:** The test must assert that the final score is `1.0`. This will prove that the AI agent can successfully query the Jupiter API, generate a valid transaction, and that this transaction can be executed on our mainnet fork to satisfy the benchmark's final state assertions.

This test will be slower than the others because it involves real network calls to an LLM, but it is the ultimate proof that the `reev` framework can successfully evaluate a real, capable on-chain AI agent.