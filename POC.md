# Proof of Concept Plan: Code-Based Transaction Generation

This document outlines the plan to evolve the `reev-agent` from a simple mock server to a more robust, code-based transaction generator. This approach eliminates the unreliability of having an LLM generate precise, byte-perfect instruction data and serves as the foundation for more complex, programmatically-assisted transaction construction.

The immediate goal is to make the `001-sol-transfer.yml` benchmark pass with a perfect score.

## Plan

1.  **Update `reev-agent` Dependencies**:
    -   Add the necessary Solana crates to the `reev-agent/Cargo.toml`.
    -   Crucially, these dependencies will be pinned to the *exact* versions specified in the provided `solana-web3-wasm/Cargo.toml` example. We will explicitly avoid using workspace versions for this stage to ensure a controlled, reproducible environment.

2.  **Implement Code-Based Transaction Generation**:
    -   The logic within the `reev-agent`'s `/gen/tx` handler will be updated.
    -   It will specifically parse the incoming prompt to identify the request for the `001-sol-transfer.yml` benchmark.
    -   Upon detection, the handler will bypass any mock logic and use the `solana-sdk` crate functions (e.g., `system_instruction::transfer`) to construct a valid `Instruction` object.
    -   Detailed logging will be added to trace each step of the process: receiving the request, identifying the prompt, constructing the instruction, and serializing the response.

3.  **Serialize and Return the Correct Instruction**:
    -   The programmatically generated `Instruction` will be converted into the `RawInstruction` struct that the `reev-runner` expects.
    -   This includes correctly serializing the instruction's `data` field into a base58 string.
    -   This `RawInstruction` will be sent back as the JSON response.

4.  **Achieve E2E Test Success**:
    -   Launch the updated `reev-agent` server (which will be managed automatically by the `reev-runner`).
    -   Run the `001-sol-transfer.yml` benchmark.
    -   The expected outcome is a successful run with a final score of `1.0`. This will validate that the code-based transaction generation is correct and that the entire pipeline functions perfectly for a "happy path" scenario.