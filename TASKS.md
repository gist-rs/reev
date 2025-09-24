# TASKS.md: Development Roadmap (Service-Oriented)

This document outlines the development plan based on the service-oriented architecture defined in `PLAN.md`. The `SolanaEnv` manages `surfpool` as an external process, communicating exclusively via its JSON-RPC API.

---

## Phase 1: Foundational Scaffolding (Complete)

-   [x] **Task 1.1**: Initialize Cargo Workspace (`reev-lib`, `reev-runner`).
-   [x] **Task 1.2**: Define Core Traits (`GymEnv`, `Agent`).
-   [x] **Task 1.3**: Define Core Structs (`AgentAction`, `AgentObservation`, `TestCase`, `Step`).
-   [x] **Task 1.4**: Implement YAML parsing for `TestCase` benchmarks.

---

## Phase 2: `SolanaEnv` as a Service (Complete)

**Goal:** Implement the `SolanaEnv` to manage the `surfpool` validator as a black-box service, configured entirely via RPC calls.

-   [x] **Task 2.1: Implement `SolanaEnv` Structure**
    -   [x] The `SolanaEnv` struct holds `Option<Child>` for the process, an `RpcClient`, and a `HashMap` for `Keypair`s.
-   [x] **Task 2.2: Implement `SolanaEnv::reset`**
    -   [x] The function terminates any previous `surfpool` process.
    -   [x] It spawns `surfpool start` using `std::process::Command`.
    -   [x] It polls the RPC endpoint until the validator is responsive.
    -   [x] It generates `Keypair`s for all accounts in the `initial_state` and populates the `keypair_map`.
    -   [x] It uses the `surfnet_setAccount` JSON-RPC cheatcode to create and fund each account on the validator according to the benchmark specification.
-   [x] **Task 2.3: Implement `SolanaEnv::step`**
    -   [x] The function dispatches `AgentAction`s to the appropriate action-building logic (e.g., `actions::sol_transfer`).
    -   [x] It signs the transaction with the correct `Keypair` from the `keypair_map`.
    -   [x] It sends the transaction using the `RpcClient` and waits for confirmation.
    -   [x] It checks ground truth assertions to determine if the episode should terminate.
-   [x] **Task 2.4: Implement `SolanaEnv::close`**
    -   [x] The function ensures the `surfpool` child process is terminated cleanly.
-   [x] **Task 2.5: Full End-to-End Test**
    -   [x] Run and verify the `transfer-simple-001.yml` benchmark to confirm the entire flow works correctly.

---

## Phase 3: Expanding Action Space & Agent Capabilities

**Goal:** Implement handlers for more complex Solana interactions (SPL Tokens, NFTs) and make the agent more capable.

-   [ ] **Task 3.1: Implement SPL-Token Transfer Action**
    -   [ ] Create a new module `reev-lib/src/actions/spl_transfer.rs`.
    -   [ ] Implement a `build_transaction` function that creates an `spl_token::instruction::transfer` transaction.
    -   [ ] Update `SolanaEnv::step` to dispatch to this new action handler when `tool_name` is `spl_transfer`.
    -   [ ] Update `check_assertion` in `metrics.rs` to handle `TokenAccountBalance` assertions by fetching and parsing SPL token account data.

-   [ ] **Task 3.2: Verify with `spl-transfer-001.yml`**
    -   [ ] Update `DummyAgent` to be able to execute the `spl_transfer` action from the benchmark file.
    -   [ ] Run the `spl-transfer-001.yml` benchmark and ensure it passes.

-   [ ] **Task 3.3: Abstract Agent Action Logic**
    -   [ ] Modify `DummyAgent` to parse the `expected_tool_calls` from the `GroundTruth` of a `TestCase` instead of having hardcoded actions. This makes the agent generic enough to run any benchmark without code changes.

-   [ ] **Task 3.4: Implement NFT Transfer**
    -   [ ] An NFT transfer is just an SPL token transfer where the amount is 1 and the mint has 0 decimals. Verify the existing `spl_transfer` action works for the `nft-transfer-001.yml` benchmark.

---

## Phase 4: Metrics, Tracing, and Reporting

**Goal:** Build out the framework's ability to measure and report on agent performance in detail.

-   [ ] **Task 4.1**: Implement Full Trace Capture.
    -   [ ] Ensure every `thought`, `action`, and `observation` is recorded in the `ExecutionTrace`.
-   [ ] **Task 4.2**: Implement Advanced Quantitative Metrics.
    -   [ ] **Tool Selection Accuracy (TSA)**: Compare the agent's tool calls against the `expected_tool_calls` in the ground truth.
    -   [ ] **Parameterization Accuracy (PA)**: For correctly selected tools, check if the parameters match the ground truth.
-   [ ] **Task 4.3**: Implement ASCII Trace Visualization.
    -   [ ] Write a renderer that outputs a human-readable summary of the `ExecutionTrace`.
-   [ ] **Task 4.4**: Generate a Summary Report.
    -   [ ] Add logic to the runner to aggregate metrics from multiple benchmark runs and output a final report in Markdown or JSON.

---

## Phase 5: LLM Integration

**Goal:** Replace the `DummyAgent` with a real LLM-powered agent and evaluate its performance.

-   [ ] **Task 5.1**: Implement `LlmAgent`.
    -   [ ] Create a new `LlmAgent` struct that implements the `Agent` trait.
    -   [ ] Use `reqwest` to call an LLM API (e.g., OpenAI).
    -   [ ] Implement prompt engineering logic to serialize the observation and conversation history into a prompt.
    -   [ ] Implement response parsing logic to convert the LLM's output into an `AgentAction`.
-   [ ] **Task 5.2**: Expand the `SolanaBench` Suite.
    -   [ ] Create a wider variety of benchmarks covering more complex scenarios and edge cases.