# `reev-lib`: The Core Evaluation Library

`reev-lib` is the core library crate for the `reev` framework. It contains all the foundational logic for the evaluation environment, agent interfaces, and benchmark specifications. It is designed to be a modular, reusable library consumed by frontend crates like `reev-runner` and `reev-tui`.

## Role in the Workspace

This crate is the "brain" of the `reev` ecosystem. It encapsulates the entire logic of the evaluation process, ensuring a strict separation of concerns from the user interface. Its primary responsibilities are:

-   Defining the standard interfaces for agent-environment interaction.
-   Implementing the hermetic Solana environment.
-   Providing the data structures for defining and parsing benchmarks.
-   Containing the logic for building and processing on-chain actions.
-   Calculating performance metrics.

For overall project architecture and goals, please see the [main project `README.md`](../../../README.md).

## Key Components

-   **`GymEnv` Trait**: Located in `src/env.rs`, this is the Rust-native, Gymnasium-inspired trait that defines the standard contract for any evaluation environment (`reset`, `step`, `render`, `close`).

-   **`SolanaEnv` Struct**: The primary implementation of the `GymEnv` trait. It manages the entire lifecycle of an external `surfpool` validator process, ensuring a hermetic and reproducible testing environment by interacting with it exclusively via its JSON-RPC API.

-   **`Agent` Trait**: Defines a standard interface for an agent, centered around the `get_action` method.

-   **Benchmark Definitions (`benchmark.rs`)**: Contains all the Rust structs (e.g., `TestCase`, `InitialAccountState`) that map directly to the `reev-benchmarks` YAML format, enabling strongly-typed parsing via `serde`.

-   **`Instruction Processing`**: The `SolanaEnv` is designed to receive a complete, raw instruction from an agent. It is responsible for safely constructing, signing, and executing a transaction from this instruction, removing the need for a predefined, tool-based action system.

## 🧪 Testing Strategy

### Test Files (5 tests)
- `constants_amounts_test.rs` - Validates Solana amount constants and conversions
- `constants_addresses_test.rs` - Tests Solana address constants and validation
- `constants_env_test.rs` - Environment variable configuration testing
- `otel_extraction_test.rs` - OpenTelemetry trace extraction validation
- `transfers.rs` - Transfer instruction processing tests

### Running Tests
```bash
# Run all lib tests
cargo test -p reev-lib

# Run specific test with output
cargo test -p reev-lib --test constants_amounts_test -- --nocapture

# Test transfer logic
cargo test -p reev-lib --test transfers -- --nocapture
```

### Test Coverage
- **Constants**: Amounts, addresses, and environment variables
- **Instruction Processing**: Transfer operations and transaction building
- **Environment Configuration**: API key and endpoint validation
- **Observability**: OpenTelemetry trace extraction and analysis