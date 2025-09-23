# RULES.md: Engineering and Development Guidelines

This document establishes the official coding conventions and architectural rules for this project. Adhering to these rules is mandatory for all new code. They are designed to ensure the codebase remains clean, maintainable, scalable, and easy for both humans and AI agents to understand and modify.

---

## 1. Architectural Principles

### Rule 1.1: Strict Separation of Concerns

-   **Binary Crates (e.g., `<project>-server`, `<project>-cli`)**: Their **only** job is to handle the user interface layer (e.g., HTTP, command-line arguments). They receive input, validate it, call the appropriate function in the core library, and format the output. They must **never** contain core business logic.
-   **The Core Library (e.g., `<project>-lib`)**: This is the brain of the application. It contains all business logic, orchestrates workflows, and provides a stable, public API for other crates to consume. It must be completely agnostic of the user interface (web, CLI, etc.).
-   **Plugin/Feature Crates**: Specialized functionalities (e.g., data sources, specific features) should be encapsulated in their own crates to promote modularity and extensibility.

### Rule 1.2: Plugin-First for Extensibility

-   **Prefer Traits for Behavior**: For behaviors that can be extended (like ingestion sources), a generic `trait` (e.g., `trait Ingestor`) MUST be defined in the core library. Each plugin then provides a struct that implements this trait. This is the required pattern for extensibility.
-   **Self-Contained Logic**: A plugin crate should contain everything it needs to operate: its specific logic, its dependencies, and any prompts or configuration templates it requires.

### Rule 1.3: Workspace and Crate Structure

-   **Flat `crates/` Directory**: The workspace SHOULD maintain a flat directory structure under `crates/`. Logical grouping is achieved through crate naming, not nested directories.
-   **Naming Convention**: All crates that are part of the project's ecosystem SHOULD be prefixed with `<project>-` (e.g., `<project>-feature`, `<project>-server`). This is configured in each crate's `Cargo.toml`.

---

## 2. Code and Module Structure

### Rule 2.1: Centralized vs. Local Types

-   **Local Types**: Each crate SHOULD have its own `src/types.rs` for internal data structures that are not part of its public API.
-   **Shared Public Types**: The central `<project>-lib/src/types.rs` module MUST only contain types that are part of the public API of the core library or are shared between two or more crates in the workspace.

### Rule 2.2: Thin Binaries (`main.rs`)

-   The `main.rs` file of any binary crate (e.g., `<project>-server`, `<project>-cli`) MUST be a "thin entrypoint."
-   Its responsibilities are limited to:
    1.  Setting up logging, configuration, and environment variables.
    2.  Calling a single, well-documented `run()` or `start()` function from its corresponding library.
    3.  Handling the final `Result` at the top level.
-   All application logic MUST reside in the library portion of the crate.

### Rule 2.3: Clean Module Declarations (`mod.rs`)

-   A `mod.rs` file should only be used to declare the modules of its parent directory.
-   It should exclusively contain `pub mod <module_name>;` and occasionally `pub use <module_name>::<item>;` to re-export items and define the module's public API.
-   It MUST NOT contain any `struct`, `enum`, `fn`, or `trait` definitions. This logic belongs in the submodule files themselves.

### Rule 2.4: Return Early Pattern

-   Functions MUST use the "return early" pattern (guard clauses) to handle errors or trivial cases at the beginning of the function. This reduces nesting and improves readability.

### Rule 2.5: Use `match` for Complex Conditionals

-   For conditional logic with more than one `else if` case, a `match` statement MUST be used. This improves readability and ensures exhaustive checking.

### Rule 2.6: Avoid Magic Strings

-   String literals that are used in multiple places or represent important constants (e.g., database paths, configuration keys, task names) MUST NOT be hardcoded directly.
-   They SHOULD be defined as `const` variables in a relevant module or loaded from a configuration file (`config.yml`, `.env`).
-   **Example**: Instead of `let storage = StorageManager::new("db/github_ingest")`, prefer `const GITHUB_DB_DIR: &str = "db/github_ingest"; let storage = StorageManager::new(GITHUB_DB_DIR);`. This centralizes the value, making it easy to change and preventing typos.

---

## 3. Development Process

### Rule 3.1: Plan Before Coding

-   Before undertaking any significant feature development or refactoring, a plan must be laid out in a `PLAN.md` file. It should outline the "why," the proposed changes, and the expected outcome.
-   The plan must then be broken down into a series of small, actionable steps in a `TASK.md` file or as a checklist in a GitHub issue.
-   Each task must be specific and verifiable (e.g., "Move struct `SearchResult` to `anyrag-lib/src/types.rs`").

### Rule 3.2: Use Feature Flags for Optional Functionality

-   Any functionality that can be considered optional, especially plugins or features with heavy dependencies, MUST be gated by a Cargo feature flag. This allows for the compilation of smaller, specialized binaries.
-   Features should be defined in the core library and propagated up to the binary crates.

---

## 4. Testing Methodology

### Rule 4.1: End-to-End Testing with `examples`

-   End-to-end (E2E) tests, which verify a full user workflow, MUST be implemented in the `examples/` directory of the relevant crate (e.g., `<project>-server/examples/`).
-   Each file in `examples/` is a small, runnable binary that acts as a client, demonstrating usage and asserting correctness.
-   **Documentation**: The `README.md` of the crate must document how to run these examples (e.g., `cargo run --example <example_filename>`).
-   This provides both a robust E2E test suite and living documentation for consumers of the library.

---

## 5. Standard Toolchain

To ensure consistency and leverage high-quality, community-vetted solutions, this project standardizes on the following foundational crates. All new code should prefer these libraries for their respective tasks.

-   **Asynchronous Runtime**: `tokio`
    -   **Use Case**: The required runtime for all `async` operations. This includes networking, file I/O, and managing green threads (tasks).

-   **Error Handling**: `anyhow` and `thiserror`
    -   **`thiserror`**: MUST be used in library crates (e.g., `<project>-lib`, `<project>-feature`) to create specific, structured, and typed errors (e.g., `FeatureError`, `ApiError`).
    -   **`anyhow`**: SHOULD be used in binary entrypoints (`main.rs`) and examples for simple, flexible error handling where the exact error type is less important than the context message.

-   **HTTP Client**: `reqwest`
    -   **Use Case**: The standard for making all outgoing HTTP requests to external APIs (e.g., AI providers, web scrapers).

-   **Serialization / Deserialization**: `serde`
    -   **Use Case**: The universal framework for all data serialization and deserialization. This applies to JSON, YAML, and any other data format.

-   **Date and Time**: `chrono`
    -   **Use Case**: The standard for all date and time manipulation, parsing, and formatting.

-   **Async Primitives**: `futures`
    -   **Use Case**: For advanced asynchronous operations, such as working with streams or joining multiple futures.

-   **Configuration Loading**: `dotenvy`
    -   **Use Case**: Used exclusively in binary entrypoints (`main.rs`) to load secrets and configuration from `.env` files into the environment.