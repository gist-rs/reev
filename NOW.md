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

## Phase 13: Granular Scoring System

The scoring model has been upgraded from a binary pass/fail (1.0 or 0.0) to a cumulative, weighted system.

*   **Weighted Assertions:** Benchmarks can now assign a `weight` to each on-chain assertion, allowing for more nuanced evaluations.
*   **Cumulative Score:** The final score is now a percentage representing the sum of weights of all passed assertions, giving a more detailed picture of an agent's performance. A transaction failure still results in a score of `0.0`.

This change provides much richer feedback for iterative agent development.

---

## Current Status and Expected Outcomes

*   **TUI and `cargo test` are Consistent:** The behavior seen in the TUI is now perfectly mirrored by `cargo test`.
*   **The Swap Benchmark Fails Correctly (for the Deterministic Agent):** When running `100-jup-swap-sol-usdc.yml` with the **deterministic agent**, it will **fail**. This is the **correct and expected outcome**. It fails because a real transaction from the Jupiter API is built against mainnet's live state (e.g., liquidity pool balances), which is inconsistent with our local user's state (who only exists on the fork).
*   **A Path for AI is Clear:** This setup correctly frames the problem for an AI agent. The swap benchmark is a perfect example of a task that is **unsolvable by a simple agent**, which is a critical capability to measure.

---

## Next Up: Phase 14 - End-to-End AI Agent Integration Test

With the framework now stable and consistent, the final piece of validation is to create an end-to-end integration test that proves a **real AI agent** can solve a complex benchmark that the deterministic agent cannot.

This test is the ultimate proof that the `reev` framework can successfully evaluate a capable on-chain AI agent, validating the entire execution loop from runner to LLM and back.

### Implementation Plan

As detailed in `PLAN.md`, the test will:
1.  Create a new test file: `crates/reev-runner/tests/ai_agent_test.rs`.
2.  Programmatically start the `reev-agent` server in AI mode.
3.  Run the Jupiter Swap benchmark (`100-jup-swap-sol-usdc.yml`).
4.  Assert that the final score is `1.0`, confirming the AI agent successfully generated a valid transaction that passed all on-chain assertions.
