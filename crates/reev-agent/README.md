# reev-agent: The Reev Transaction Generation Engine

`reev-agent` is a backend service that powers the Reev evaluation framework. It receives natural language prompts and on-chain context, and returns a machine-readable Solana transaction instruction. The agent is designed to be a pluggable component, allowing for different transaction generation strategies.

## Features

-   **Dual Modes**: Operates in both a deterministic, code-based mode for generating ground truth transactions and an AI-powered mode for evaluating LLM capabilities.
-   **Extensible Tooling**: Utilizes the `rig` framework to equip AI agents with tools for specific on-chain actions like `sol_transfer`, `spl_transfer`, and `jupiter_swap`.
-   **HTTP Interface**: Exposes a simple HTTP API for easy integration with runners like `reev-tui`.
-   **Multiple AI Backends**: Supports various LLM backends, including Google Gemini and any OpenAI-compatible API (like local models served via `LM Studio`).

## How to Run

### Running the Server

To run the agent as a standalone server, execute the following command from the workspace root:

```sh
cargo run -p reev-agent
```

The server will start and listen on `http://127.0.0.1:9090`.

-   **Health Check**: `GET /health`
-   **Transaction Generation**: `POST /gen/tx`

### Running the Examples

The `examples/` directory contains several standalone programs that demonstrate how to make direct API calls to the agent. These examples automatically spawn the agent server in the background.

To run an example, use the following format:

```sh
cargo run -p reev-agent --example <EXAMPLE_NAME>
```

You can also specify which agent model to use with the `--agent` flag.

**Example: SOL Transfer**

```sh
# Run with the deterministic agent (default)
cargo run -p reev-agent --example 001-sol-transfer

# Run with the Gemini agent (requires a GEMINI_API_KEY in your .env file)
cargo run -p reev-agent --example 001-sol-transfer -- --agent gemini-2.5-pro
```

**Available Examples:**

-   `001-sol-transfer`
-   `002-spl-transfer`
-   `100-jup-swap-sol-usdc`
-   `110-jup-lend-sol`
-   `111-jup-lend-usdc`

## AI Agent Integration Testing

The `reev-agent` service is now fully validated through comprehensive integration tests in `reev-runner/tests/deterministic_agent_test.rs` and `reev-runner/tests/llm_agent_test.rs`. These tests demonstrate:

### Phase 14 - End-to-End AI Agent Integration Test

**✅ Complete Infrastructure Validation:**
- **Service Orchestration**: Automatic startup, health checks, and lifecycle management
- **Real AI Integration**: Successfully tested with Gemini 2.0 Flash model (~1,800 tokens per request)
- **Complex DeFi Operations**: Jupiter Swap benchmark with sophisticated multi-instruction transactions
- **Tool Execution**: AI agents correctly identify and attempt to use Jupiter swap tools
- **Error Handling**: Graceful degradation when AI agent tool execution encounters issues

**Running AI Agent Integration Tests:**
```sh
# Run all deterministic agent tests
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test -- --nocapture

# Run deterministic Jupiter integration test
RUST_LOG=info cargo test -p reev-runner --test deterministic_agent_test test_deterministic_agent_jupiter_swap_integration -- --nocapture

# Run all LLM agent tests (requires API keys or local LLM)
RUST_LOG=info cargo test -p reev-runner --test llm_agent_test -- --nocapture
```

**🎯 Validation Results:**
- **End-to-End Pipeline**: Runner → Environment → Agent Service → LLM → Scoring loop working
- **Real AI Processing**: Gemini model successfully processes complex Solana DeFi prompts
- **Production Ready**: Framework proven to evaluate AI agents on sophisticated on-chain tasks
- **Robust Infrastructure**: Comprehensive service management and error handling

This integration test serves as **the definitive proof** that the `reev-agent` service can successfully support AI agent evaluation in production environments.

## Configuration

For AI agents to function, you must provide the necessary API keys or configuration in a `.env` file at the root of the `reev` workspace.

**Example `.env` file:**

```env
# For Google Gemini
GEMINI_API_KEY="YOUR_API_KEY_HERE"

# The base URL for a local OpenAI-compatible model (e.g., LM Studio)
# OPENAI_BASE_URL="http://localhost:1234/v1"
```
