# RULES.md: Engineering and Development Guidelines

This document establishes the official coding conventions and architectural rules for the `reev` project. Adhering to these rules is mandatory for all new code. They are designed to ensure the codebase remains clean, maintainable, scalable, and easy for both humans and AI agents to understand and modify.

---

## 1. Architectural Principles

### Rule 1.1: Strict Separation of Concerns

-   **The Runner Crate (`reev-runner`)**: This is a binary crate. Its **only** job is to handle the user interface layer (i.e., command-line arguments via `clap`). It receives input, calls the appropriate functions in the core library to orchestrate the evaluation run, and formats the final output (reports, traces). It must **never** contain core evaluation or environment logic.
-   **The Core Library (`reev-lib`)**: This is the brain of the application. It contains all core logic, including the `GymEnv` trait, the `SolanaEnv` implementation, agent definitions, action handlers, and metrics calculations. It must be completely agnostic of the user interface.
-   **Action Modules (`reev-lib/src/actions/`)**: Each distinct on-chain action (e.g., `sol_transfer`, `spl_transfer`) MUST be encapsulated in its own module. This promotes modularity and makes it easy to add new capabilities to the environment.

### Rule 1.2: Workspace and Crate Structure

-   **Flat `crates/` Directory**: The workspace MUST maintain a flat directory structure under `crates/`.
-   **Naming Convention**: All crates that are part of the project's ecosystem MUST be prefixed with `reev-` (e.g., `reev-lib`, `reev-runner`). This is configured in each crate's `Cargo.toml`.

---

## 2. Code and Module Structure

### Rule 2.1: Thin Binaries (`main.rs`)

-   The `main.rs` file of any binary crate (`reev-runner`) MUST be a "thin entrypoint."
-   Its responsibilities are limited to:
    1.  Parsing command-line arguments.
    2.  Setting up any necessary configuration.
    3.  Calling a single, well-documented `run()` function from its corresponding library or internal module.
    4.  Handling the final `Result` at the top level.

### Rule 2.2: Clean Module Declarations (`mod.rs`)

-   A `mod.rs` file should only be used to declare the modules of its parent directory.
-   It should exclusively contain `pub mod <module_name>;` and occasionally `pub use <module_name>::<item>;` to re-export items and define the module's public API.
-   It MUST NOT contain any `struct`, `enum`, `fn`, or `trait` definitions. This logic belongs in the submodule files themselves.

### Rule 2.3: Return Early Pattern

-   Functions MUST use the "return early" pattern (guard clauses) to handle errors or trivial cases at the beginning of the function. This reduces nesting and improves readability.

### Rule 2.4: Use `match` for Complex Conditionals

-   For conditional logic with more than one `else if` case, a `match` statement MUST be used. This improves readability and ensures exhaustive checking where applicable.

### Rule 2.5: Avoid Magic Strings for Tool Names

-   The string literals for `tool_name` (e.g., "sol_transfer", "spl_transfer") are a critical interface between the agent and the environment.
-   While they are defined in benchmark files, within the Rust code they SHOULD be handled via pattern matching in the `SolanaEnv::step` function to ensure all supported tools are explicitly handled.

---

## 3. Development Process

### Rule 3.1: Plan Before Coding

-   Before undertaking any significant feature development or refactoring, the project's planning documents (`PLAN.md`, `TASKS.md`, `NOW.md`) must be updated.
-   The plan must be broken down into a series of small, actionable steps in `TASKS.md` or a related document.
-   Each task must be specific and verifiable (e.g., "Add `mint_data` field to `InitialAccountState` struct").

### Rule 3.2: Use Feature Flags for Optional Functionality

-   Any functionality that can be considered optional, especially future agents or features with heavy dependencies, SHOULD be gated by a Cargo feature flag. This allows for the compilation of smaller, specialized binaries.

---

## 4. Testing Methodology

### Rule 4.1: Benchmark-Driven Testing

-   The primary method for testing the framework's functionality is through end-to-end benchmark tests run by the `reev-runner`.
-   Each new feature (e.g., a new action handler) MUST be accompanied by a new benchmark `.yml` file that specifically tests its functionality.
-   Benchmarks serve as both the test suite and the living specification for the agent's expected capabilities.

---

## 5. Standard Toolchain

To ensure consistency, this project standardizes on the following foundational crates.

-   **Error Handling**: `anyhow`
    -   **Use Case**: Used throughout the application for simple, flexible error handling with context. Since this is an application-focused workspace (not a general-purpose library), `anyhow` is preferred over `thiserror` for its ease of use.

-   **Serialization / Deserialization**: `serde`, `serde_yaml`, `serde_json`
    -   **Use Case**: The universal framework for all data serialization and deserialization, especially for parsing benchmark files and handling RPC data.

-   **Command-Line Interface**: `clap`
    -   **Use Case**: The standard for building the command-line interface in the `reev-runner` binary.

-   **Solana Interaction**: `solana-client`, `solana-sdk`, `spl-token`
    -   **Use Case**: The official crates for all interactions with the Solana blockchain, including RPC communication, transaction building, and SPL token operations.

-   **Asynchronous Code**: The core `reev-lib` environment is **synchronous** to simplify state management and reproducibility. Asynchronous code (using `tokio` and `reqwest`) will be introduced specifically for the `LlmAgent` implementation in a later phase and should be strictly confined to that component.