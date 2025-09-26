# NOW.md: Current Development Phase - Building a Comparative LLM Agent

This document outlines the immediate, actionable plan for the current development phase of the `reev` project.

---

## Phase 11: Building a Tool-Calling LLM Agent for Comparative Evaluation

**Goal:** Implement a new `reev-llm-agent` that uses a tool-calling architecture. This new agent will be evaluated *against* our existing `reev-agent`, which serves as the "ground truth" or "oracle" for perfect performance.

### Key Concept: The Dual-Agent Strategy

We are **not** replacing the mock agent. Instead, we are creating a system that can run benchmarks using two different agents to allow for direct comparison:

1.  **`reev-agent` (The Oracle):** The existing code-based agent. It serves as our ground truth, deterministically generating the *correct* instruction for a given prompt. Its performance represents a perfect score of 1.0.

2.  **`reev-llm-agent` (The Subject):** This is the new agent we will build in a new crate. It will use the `rig` crate to interact with a real LLM, asking it to choose the correct "tool" (on-chain action) and provide the right parameters based on the prompt. Its performance will be measured against the `reev-agent`.

### 1. Define On-Chain Actions as `rig` "Tools"

The core on-chain actions will be defined as structs that implement the `rig::Tool` trait. This allows `rig` to present them to the LLM as callable functions.

-   **`SolTransferTool`**: Will describe the native SOL transfer action to the LLM and will call our centralized `reev_lib::actions::sol_transfer::create_instruction` function.
-   **`SplTransferTool`**: Will describe the SPL Token transfer action to the LLM and will call `reev_lib::actions::spl_transfer::create_instruction`.

### 2. Implement the `reev-llm-agent` using `rig`

A new agent implementation will be created that orchestrates the LLM interaction.

-   It will initialize a `rig` agent, providing it with a system preamble and registering the available tools (`SolTransferTool`, `SplTransferTool`).
-   When a prompt is received, the `reev-llm-agent` will use `rig` to query the LLM.
-   `rig` will send the prompt and the tool definitions to the LLM.
-   The LLM is expected to respond with a request to call one of the tools with specific arguments.
-   The `reev-llm-agent` will execute the tool call (which generates the raw instruction) and return the result.

### 3. LLM Interaction Example

The `reev-llm-agent` will make a `POST` request to an LLM service, structured like this:

```bash
curl http://localhost:1234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-local-model",
    "messages": [
      { "role": "system", "content": "You are a helpful Solana assistant. Your goal is to generate the correct transaction to fulfill the user''s request by using the provided tools." },
      { "role": "user", "content": "Please send 15 USDC from USER_USDC_ATA to RECIPIENT_USDC_ATA." }
    ],
    "tools": [ /* Rig will inject the tool definitions here */ ],
    "tool_choice": "auto"
}'
```

### 4. Implement Agent Selection in the Runner

The `reev-runner` will be modified to accept a parameter that selects which agent to use for the evaluation run, enabling our comparative analysis.

**Example Usage:**

```bash
# Run the benchmark against the perfect "oracle" agent
cargo run -p reev-runner -- --agent agent benchmarks/002-spl-transfer.yml

# Run the same benchmark against the new LLM-based agent
cargo run -p reev-runner -- --agent llm-agent benchmarks/002-spl-transfer.yml
```
