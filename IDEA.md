# reev 🪸 (REproducible EVaaluation)

## A Framework for the Reproducible Evaluation of Solana-Native LLM Agents

### Introduction

The evaluation of Large Language Models (LLMs) has matured for core competencies like language understanding, but these methodologies are insufficient for LLM-based autonomous agents. An agent perceives a dynamic environment, formulates plans, and executes actions to achieve goals. When the environment is a high-stakes, stateful system like the Solana blockchain, the evaluation paradigm must shift from assessing static outputs to analyzing dynamic, interactive behavior in a reproducible manner.

`reev` is a Rust-native framework designed for this purpose. It provides the architecture, tools, and methodology to rigorously evaluate Solana agents, ensuring that results are verifiable, trustworthy, and directly comparable.

---

## Part I: Core Principles

### 1.1. The Agentic Paradigm

Evaluating an LLM agent is fundamentally different from evaluating a simple text generator. The focus shifts from the quality of a single output to the effectiveness of a sequence of actions. For a Solana agent, we must evaluate:

-   **Task Completion Rate:** Did the agent achieve the goal specified by the user's prompt?
-   **Instruction Generation Quality:** Did the agent generate a correct and secure raw transaction instruction to achieve the goal? This represents an evolution from the initial "tool-choosing" model to a more advanced "instruction-generating" model, testing the LLM's deep, low-level knowledge.
-   **Efficiency:** How many transactions and how much in fees did the agent consume?
-   **Robustness:** How does the agent handle errors like failed transactions?

### 1.2. Reproducibility Through a Hermetic Environment

The central challenge is evaluating a non-deterministic LLM against a deterministic blockchain. `reev` solves this by enforcing a **hermetic** evaluation environment.

This is achieved by treating the `surfpool` test validator as an **external, ephemeral service**. For each test run, the `SolanaEnv` programmatically spawns a fresh `surfpool` process, configures the on-chain state to exact specifications via RPC "cheatcodes," and terminates it afterward. This guarantees that every evaluation starts from the identical state, making the results fully reproducible.

### 1.3. Gymnasium-Inspired API

To ensure a clean separation of concerns, `reev` adopts the principles of the industry-standard Gymnasium API in a native Rust `trait`. The `GymEnv` trait defines the core interaction loop (`reset`, `step`, `render`, `close`), allowing agent logic to be developed independently of the environment's internal workings.

---

## Part II: The `reev` Architecture

The framework is organized into a Cargo workspace with a clear division of responsibilities.

### 2.1. `reev-lib`: The Core Framework

This library contains all the core logic, including:

-   **`SolanaEnv`**: The concrete implementation of the `GymEnv` trait. It manages the `surfpool` lifecycle and all RPC communication.
-   **`Agent` Trait**: A simple trait defining the agent interface.
-   **Benchmark Structs**: Rust types that define the structure of a `reev-benchmarks` YAML file, enabling strongly-typed parsing with `serde`.
-   **Instruction Processing**: The framework is designed to receive a complete, raw instruction from the agent. The `SolanaEnv` is then responsible for safely constructing, signing, and executing a transaction from this instruction.

### 2.2. `reev-runner`: The CLI Orchestrator

A thin binary that orchestrates the evaluation process. It is responsible for:
-   Parsing command-line arguments (e.g., the path to a benchmark file).
-   Instantiating the `SolanaEnv` and `Agent`.
-   Running the main evaluation loop.
-   Calling the metrics calculation modules and printing the final report.

---

## Part III: The `reev-benchmarks` Suite

`reev-benchmarks` is the suite of evaluation tasks, defined in YAML files, that serves as the executable specification for agent capabilities.

### 3.1. Anatomy of a Benchmark File

Each `.yml` file is a self-contained test case with a standardized structure:

-   **`id`**: A unique identifier (e.g., `001-SPL-TRANSFER-SIMPLE`).
-   **`description`**: A human-readable summary.
-   **`initial_state`**: A declarative list of accounts to create on the test validator, supporting SOL accounts, SPL token accounts, and on-the-fly SPL token mint initialization.
-   **`prompt`**: The natural language instruction for the agent.
-   **`ground_truth`**: The objective criteria for success.
    -   `final_state_assertions`: A list of on-chain conditions (e.g., `SolBalance`) that must be true.
    -   `expected_instruction`: The ideal raw Solana instruction the agent should generate, used for validation and scoring.

### 3.2. Example Benchmark (`spl-transfer-001.yml`)

```yaml
id: 002-SPL-TRANSFER
description: A simple SPL-Token transfer using a mock USDC mint.
tags: ["spl-token", "transfer", "usdc"]

initial_state:
  - pubkey: "USER_WALLET_PUBKEY"
    lamports: 1000000000 # 1 SOL

  - pubkey: "MOCK_USDC_MINT"
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    mint_data:
      decimals: 6

  - pubkey: "USER_USDC_ATA"
    data: '{"mint": "MOCK_USDC_MINT", "owner": "USER_WALLET_PUBKEY", "amount": 50000000}' # 50 USDC

prompt: "Please send 15 USDC from my token account (USER_USDC_ATA) to the recipient's token account (RECIPIENT_USDC_ATA)."

ground_truth:
  final_state_assertions:
    - type: TokenAccountBalance
      pubkey: "RECIPIENT_USDC_ATA"
      expected: 15000000 # 15 USDC
  expected_instruction:
    program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    accounts:
      - { pubkey: "USER_USDC_ATA", is_signer: false, is_writable: true }
      - { pubkey: "RECIPIENT_USDC_ATA", is_signer: false, is_writable: true }
      - { pubkey: "USER_WALLET_PUBKEY", is_signer: true, is_writable: false }
    data: "3kVA21YASy2b"
```

---

## Part IV: Reporting and Visualization

A key goal of `reev` is to provide rich, multi-layered feedback. This is detailed in `UI.md` and will be implemented in phases.

### 4.1. Metrics

-   **Task Success Rate (TSR)** (Implemented): The primary metric, confirming both transaction success and the correctness of final on-chain state.
-   **Tool Selection & Parameterization Accuracy** (Planned): Nuanced metrics measuring the agent's reasoning.

### 4.2. Phased UI Development Plan

1.  **Structured YAML Output**: The runner will generate a canonical YAML file for each test result, containing the full `ExecutionTrace`. This serves as the data foundation for all UI.
2.  **ASCII Tree Rendering**: The runner will also render a human-readable ASCII tree of the trace to the console for immediate feedback.
3.  **Interactive TUI Cockpit**: A `Ratatui`-based TUI will provide a "mission control" for selecting models, running benchmarks, and interactively exploring the results.
4.  **OpenTelemetry Integration**: The framework will be instrumented to emit OTel traces, allowing for advanced performance analysis in observability platforms like Jaeger.

### Conclusion

The `reev` framework provides a robust, reproducible, and extensible foundation for evaluating Solana-native LLM agents. By combining a standardized `GymEnv` interface with a hermetic, service-oriented testing environment, it ensures that evaluation results are both rigorous and verifiable, paving the way for the development of capable and trustworthy on-chain autonomous agents.

---

## Part V: The Comparative Agent Framework

The `reev` framework is built around a comparative evaluation model. It uses a dual-agent architecture within a single `reev-agent` service to test AI performance against a perfect baseline.

### 5.1. The Dual-Agent Architecture

The `reev-agent` crate exposes a single HTTP endpoint (`/gen/tx`) that internally routes requests to one of two agents based on a query parameter:

1.  **The Deterministic Agent (The Oracle):** Activated by the `?mock=true` query parameter. This agent uses hardcoded Rust logic to deterministically generate the *correct* raw Solana instruction for a given benchmark prompt. It serves as the ground truth, representing a perfect performance score.

2.  **The AI Agent (The Subject):** This is the default agent. It uses the `rig` crate to interface with a configured LLM (e.g., Gemini, a local model via OpenAI-compatible API). It presents the on-chain context and a set of available "tools" (`sol_transfer`, `spl_transfer`) to the model, which must then choose the correct tool and parameters to fulfill the user's prompt.

This dual-mode system allows the `reev-runner` and `reev-tui` to seamlessly switch between running a benchmark against the perfect "oracle" and running it against a real AI model, enabling direct, reproducible performance comparisons.

### 5.2. Interactive TUI Cockpit

The `reev-tui` provides a fully interactive `ratatui`-based cockpit for running and analyzing benchmarks. It automatically discovers all benchmarks, runs them asynchronously to keep the UI responsive, and displays live status updates (`RUNNING`, `SUCCEEDED`, `FAILED`). Upon completion, it renders a detailed execution trace for immediate analysis, making it a powerful tool for iterative agent development.
