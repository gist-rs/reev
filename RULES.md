# RULES.md: Engineering and Development Guidelines

This document establishes the official coding conventions and architectural rules for the `reev` project. Adhering to these rules is mandatory for all new code. They are designed to ensure the codebase remains clean, maintainable, and scalable for the production-ready framework.

---

## 1. Architectural Principles

### Rule 1.1: Strict Separation of Concerns
-   **Core Library (`reev-lib`)**: Contains all core evaluation logic, environment implementations, agent traits, and protocol handlers. Must remain UI-agnostic.
-   **Binary Crates (`reev-runner`, `reev-tui`, `reev-agent`)**: User-facing entrypoints handling UI layer (CLI, TUI, API). Must not contain core evaluation logic.
-   **Protocol Modules**: Jupiter, native, and other protocol handlers in dedicated modules with clear interfaces.
-   **Mainnet Fork Validation**: All operations must work on real mainnet-forked environments, not mock data.

### Rule 1.2: Service-Oriented Environment
-   **Smart Service Management**: Surfpool is treated as an intelligent managed service with automatic detection, binary caching, and shared instance support.
-   **RPC-Only Communication**: All surfpool interaction occurs through JSON-RPC API. Direct library linking is forbidden.
-   **Lifecycle Management**: Automatic startup, health monitoring, and graceful shutdown are required.

### Rule 1.3: Protocol Abstraction
-   **Protocol Traits**: All protocols must implement standardized traits for consistency.
-   **SDK Integration**: Use official SDKs (Jupiter SDK) instead of direct API calls when available.
-   **Mainnet Fork Validation**: All operations must work on real mainnet-forked environments, not mock data.

### Rule 1.4: Production-Ready Architecture
-   **Comprehensive Testing**: All features must have corresponding benchmarks with 100% success rates.
-   **Scoring Validation**: Must include test cases covering 0%, ~50%, ~75%, and 100% score scenarios.
-   **Anti-False-Positive Testing**: Must differentiate between failure modes (no attempt vs attempted but failed).
-   **Flow Execution Support**: Must support multi-step flow benchmarks with proper transaction isolation.
-   **Error Handling**: Robust error handling with clear logging and graceful degradation.
-   **Performance Optimization**: Binary caching, shared instances, and efficient resource management.

### Rule 1.5: Flow Framework Architecture
-   **Step-by-Step Execution**: Flow benchmarks must execute each step as a separate transaction.
-   **Transaction Isolation**: Step failures must not cascade to other steps in the flow.
-   **State Propagation**: Account states must flow correctly between steps automatically.
-   **Agent Consistency**: Both deterministic and AI agents must handle flows identically.
-   **Dependency Resolution**: Step dependencies must be properly resolved before execution.

---

## 2. Code and Module Structure

### Rule 2.1: Thin Binaries (`main.rs`)
-   The `main.rs` file of any binary crate MUST be a "thin entrypoint."
-   Its responsibilities are limited to: parsing arguments, setting up configuration, calling a single top-level `run()` function, and handling the final `Result`.

### Rule 2.2: Return Early Pattern
-   Functions MUST use the "return early" pattern (guard clauses) to handle errors or trivial cases at the beginning of the function. This reduces nesting and improves readability.

### Rule 2.3: Use `match` for Control Flow
-   For conditional logic with more than one `else if` case, a `match` statement MUST be used.
-   The `SolanaEnv::step` function MUST use a `match` statement on the `tool_name` to dispatch to the correct action handler.

### Rule 2.4: Module Index Files (`mod.rs`)
-   Module index files (`mod.rs`) MUST only contain `mod` and `pub use` statements. They serve as a public API for the module and MUST NOT contain any logic, structs, or enums.

---

## 3. Development Process

### Rule 3.1: Plan-Driven Development
-   Significant feature development or refactoring MUST be preceded by an update to `PLAN.md` and `UI.md`.
-   The work must be broken down into specific, verifiable steps in `TASKS.md`.

### Rule 3.2: Debugging and Logging
-   **Don't Guess, Prove**: When debugging, do not guess the cause. Insert logging statements (`info!`, `debug!`, `trace!`) to trace execution and inspect state step-by-step. Start from a last known-working state and reintroduce changes incrementally.
-   **Use `tracing`**: The `tracing` crate is the project standard. Use it for all logging. Control log verbosity with the `RUST_LOG` environment variable (e.g., `RUST_LOG=reev_lib=trace`).
-   **Scoring Debug**: Use `RUST_LOG=debug` to get detailed scoring breakdown including component comparisons and weight calculations.
-   **Score Validation**: Always validate that new benchmarks produce expected scores within ±5% tolerance.
-   **Flow Debug**: Use `RUST_LOG=info` to trace flow step execution and identify step failures.

### Rule 3.3: Flow Development Process
-   **Flow Definition**: All flow benchmarks MUST define clear steps with descriptions, prompts, and dependencies.
-   **Step Testing**: Each flow step MUST be testable independently before integration.
-   **Agent Consistency**: Flow implementations MUST work identically across deterministic and AI agents.
-   **Error Isolation**: Step failures MUST be contained and not affect other steps.

---

## 4. Workspace and Dependencies

### Rule 4.1: Production Workspace Structure
-   Maintain flat directory structure under `crates/` with `reev-` prefix.
-   Separate concerns: core library, runners, agents, protocols, and examples.
-   Include comprehensive integration tests for all major components.

### Rule 4.2: Production Toolchain Standards
-   **Error Handling**: `anyhow` for context-rich error management with proper error chaining.
-   **Serialization**: `serde`, `serde_json`, `serde_yaml` for all data handling.
-   **CLI**: `clap` for command-line interfaces with comprehensive help.
-   **TUI**: `ratatui` for interactive terminal interfaces with real-time updates.
-   **Solana**: `solana-client`, `solana-sdk`, `spl-token` for blockchain integration.
-   **LLM Integration**: `rig` for agent-LLM communication with multiple model support.
-   **Observability**: `tracing` for structured logging and performance analysis.

### Rule 4.3: Configuration Management
-   Use `.env` files for local development configuration and secrets.
-   Implement `dotenvy` for environment variable loading.
-   Support multiple configuration sources: CLI args, env files, and config files.
-   Validate all configuration at startup with clear error messages.

### Rule 4.4: Benchmark Development Standards
-   **Score Coverage**: Every benchmark category must include test cases covering the full score spectrum.
-   **Weight Validation**: All benchmark weights must be documented and validated against expected score ranges.
-   **Ground Truth Accuracy**: Expected instructions must be precise and match actual successful executions.
-   **Failure Mode Testing**: Must include intentional failure tests to validate scoring system accuracy.
-   **Flow Definition**: Flow benchmarks must define steps with proper sequencing, dependencies, and timeouts.
-   **Documentation**: Each benchmark must have clear documentation of purpose, expected score, and validation criteria.

### Rule 4.5: Flow Development Requirements
-   **Step Isolation**: Each flow step must execute as an independent transaction with proper error handling.
-   **State Management**: Account state changes must flow correctly between steps without manual intervention.
-   **Dependency Resolution**: Step dependencies must be properly validated before execution.
-   **Agent Parity**: Both deterministic and AI agents must handle flows using the same interface and produce consistent results.
-   **Timeout Handling**: Each step must respect individual timeout constraints.

### Rule 4.6: API-Only Instruction Generation for Jupiter Operations
-   **Jupiter API Mandate**: All Jupiter instructions MUST come from official Jupiter API calls (get_swap_instructions, get_deposit_instructions, get_withdraw_instructions, etc.).
-   **No LLM Instruction Generation**: The LLM is strictly FORBIDDEN from generating Jupiter transaction instructions, instruction data, or base58-encoded data.
-   **Exact API Extraction**: Tools must extract the exact instructions returned by Jupiter API without modification, formatting, or interpretation.
-   **API Response Integrity**: Preserve the complete structure of API responses including program_id, accounts, and data fields exactly as returned by Jupiter.
-   **SDK Enforcement**: Use only official Jupiter SDK methods that internally call the Jupiter API. Direct API manipulation is prohibited.
-   **Instruction Validation**: All Jupiter instructions must be validated against the actual Jupiter API response format and content.
-   **No Custom Data Encoding**: Never create custom instruction data or perform base58 encoding for Jupiter operations.

### Rule 4.7: Testing and Validation Requirements
-   **Score Validation Suite**: Must maintain comprehensive test suite with validated score scenarios.
-   **Regression Testing**: All score validations must pass on every code change.
-   **Performance Monitoring**: Track scoring system performance and ensure consistent execution times.
-   **Database Validation**: Verify that all test results are correctly persisted and retrievable.
-   **Cross-Agent Testing**: Validate scoring consistency across different agent types (deterministic, AI).
-   **Flow Testing**: All flow benchmarks must validate step-by-step execution and proper aggregation.
-   **Transaction Isolation**: Verify that step failures don't cascade to other steps.
-   **State Consistency**: Validate account state propagation between flow steps.
