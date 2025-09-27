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

## Configuration

For AI agents to function, you must provide the necessary API keys or configuration in a `.env` file at the root of the `reev` workspace.

**Example `.env` file:**

```env
# For Google Gemini
GEMINI_API_KEY="YOUR_API_KEY_HERE"

# The base URL for a local OpenAI-compatible model (e.g., LM Studio)
# OPENAI_BASE_URL="http://localhost:1234/v1"
```
