# RULES.md: Engineering and Development Guidelines

This document establishes the official coding conventions and architectural rules for the `reev` project. Adhering to these rules is mandatory for all new code. They are designed to ensure the codebase remains clean, maintainable, and scalable for the production-ready framework.

---

## 1. Architectural Principles

### Rule 1.1: Strict Separation of Concerns
-   **Core Library (`reev-lib`)**: Contains all core evaluation logic, environment implementations, agent traits, and protocol handlers. Must remain UI-agnostic.
-   **Binary Crates (`reev-runner`, `reev-tui`, `reev-agent`)**: User-facing entrypoints handling UI layer (CLI, TUI, API). Must not contain core evaluation logic.
-   **Protocol Modules**: Jupiter, native, and other protocol handlers in dedicated modules with clear interfaces.

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
-   **Error Handling**: Robust error handling with clear logging and graceful degradation.
-   **Performance Optimization**: Binary caching, shared instances, and efficient resource management.

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
