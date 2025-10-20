# API Usage with cURL

This document provides examples of how to interact with the Reev API using cURL commands.

## 🔍 Tool Call Logging with OpenTelemetry

The reev-agent uses OpenTelemetry (OTEL) tracing to log all tool calls to `logs/tool_calls.log`. This log file contains detailed information about agent execution flows and is used to generate flow diagrams.

### Generate Tool Call Logs

To generate `logs/tool_calls.log` for flow visualization:

```bash
# Run the OpenTelemetry logging demo
cargo run --example otel_tool_logging_demo

# Or run any benchmark example
cargo run --example 001-sol-transfer --agent local

# The tool calls will be logged to logs/tool_calls.log
```

### Generate Flow Diagram from Tool Logs

```bash
# Generate Mermaid diagram from tool_calls.log
cargo run --bin flow_visualizer -- --input logs/tool_calls.log

# Generate with custom output file
cargo run --bin flow_visualizer -- --input logs/tool_calls.log --output my_diagram.mmd

# Generate HTML preview
cargo run --bin flow_visualizer -- --input logs/tool_calls.log --html
```

### Tool Log Format

The `logs/tool_calls.log` contains structured entries like:
```
2024-01-15T10:30:00.123Z INFO [OpenAIAgent] Starting agent execution with OpenTelemetry tracing
2024-01-15T10:30:01.456Z INFO [AccountBalanceTool] Starting tool execution in accountbalance_tool_call with args: {"pubkey": "USER_1"}
2024-01-15T10:30:02.789Z INFO [AccountBalanceTool] Tool completed successfully in accountbalance_tool_call with result: {"balance": "100 USDC"}
2024-01-15T10:30:03.012Z INFO [JupiterSwapTool] Starting tool execution in jupiterswap_tool_call with args: {"amount": "0.1", "input_mint": "So11111111111111111111111111111111111111112", "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}
2024-01-15T10:30:04.345Z INFO [JupiterSwapTool] Tool completed successfully in jupiterswap_tool_call with result: {"output_amount": "5.23"}
2024-01-15T10:30:05.678Z INFO [OpenAIAgent] Agent execution completed
```

### Environment Variables

```bash
# Disable tool logging (default: enabled)
export REEV_ENABLE_TOOL_LOGGING=false

# Tool logging is enabled by default
# Logs are written to logs/tool_calls.log
```

## 🚀 Running Benchmarks

### Basic Benchmark Execution

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/{benchmark-id}/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'
```

**Example:**
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'
```

### With Different Agent Types

```bash
# Local agent
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local"
  }'

# Gemini model
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4.6"
  }'

# GLM model
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "glm-4-6"
  }'
```

### With Agent Configuration

```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local",
    "config": {
      "model": "gpt-4",
      "api_base": "http://localhost:8000",
      "api_key": "your-api-key-here",
      "max_tokens": 2000,
      "temperature": 0.1
    }
  }'
```

## 📋 Available Endpoints

### List Benchmarks
```bash
curl http://localhost:3001/api/v1/benchmarks
```

### Check Execution Status
```bash
curl http://localhost:3001/api/v1/benchmarks/{benchmark-id}/status/{execution-id}
```

### Stop Execution
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/{benchmark-id}/stop/{execution-id}
```

### Get Flow Diagram (Session-based)
```bash
# Get flow for specific session
curl http://localhost:3001/api/v1/flows/{session-id}

# Get HTML version
curl http://localhost:3001/api/v1/flows/{session-id}?format=html
```

### Agent Configuration
```bash
# Save agent config
curl -X POST http://localhost:3001/api/v1/agents/config \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "local",
    "model": "gpt-4",
    "api_base": "http://localhost:8000",
    "api_key": "your-api-key"
  }'

# Get agent config
curl http://localhost:3001/api/v1/agents/config/{agent-type}

# Test agent connection
curl -X POST http://localhost:3001/api/v1/agents/test \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "local",
    "model": "gpt-4",
    "api_base": "http://localhost:8000",
    "api_key": "your-api-key"
  }'
```

## 📊 Available Benchmarks

- `001-sol-transfer` - SOL transfer
- `002-spl-transfer` - SPL token transfer
- `003-spl-transfer-fail` - Failed SPL transfer
- `004-partial-score-spl-transfer` - Partial score SPL transfer
- `100-jup-swap-sol-usdc` - Jupiter SOL/USDC swap
- `110-jup-lend-deposit-sol` - Jupiter lending SOL deposit
- `111-jup-lend-deposit-usdc` - Jupiter lending USDC deposit
- `112-jup-lend-withdraw-sol` - Jupiter lending SOL withdraw
- `113-jup-lend-withdraw-usdc` - Jupiter lending USDC withdraw
- `114-jup-positions-and-earnings` - Jupiter positions
- `115-jup-lend-mint-usdc` - Jupiter lending USDC mint
- `116-jup-lend-redeem-usdc` - Jupiter lending USDC redeem
- `200-jup-swap-then-lend-deposit` - Swap then lend deposit

## 🔄 Complete Workflow Example

```bash
# 1. Check API health
curl http://localhost:3001/api/v1/health

# 2. List available benchmarks
curl http://localhost:3001/api/v1/benchmarks

# 3. Run a benchmark
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic"
  }'

# Response: {"execution_id":"uuid-here","status":"started"}

# 4. Check execution status
curl http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/uuid-here

# 5. Once complete, get the flow diagram using session_id
curl http://localhost:3001/api/v1/flows/{session-id}

# 6. Get HTML version for visualization
curl http://localhost:3001/api/v1/flows/{session-id}?format=html > flow.html
```

## 🛠️ Tips

1. **Pretty Print JSON**: Add `| jq` to any curl command for formatted output
   ```bash
   curl http://localhost:3001/api/v1/benchmarks | jq
   ```

2. **Save Responses**: Redirect output to files
   ```bash
   curl http://localhost:3001/api/v1/flows/{session-id}?format=html > flow_diagram.html
   ```

3. **Include Headers**: Add `-v` for verbose output to debug issues
   ```bash
   curl -v -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
     -H "Content-Type: application/json" \
     -d '{"agent": "deterministic"}'
   ```

4. **Check Response Times**: Use `-w` to measure API performance
   ```bash
   curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3001/api/v1/health
   ```

## 📝 Session ID vs Execution ID

- **Execution ID**: Temporary ID for tracking running benchmark execution
- **Session ID**: Permanent ID for completed benchmark sessions (used for flow diagrams)
- After execution completes, use the session_id from the results to access flow diagrams

## 🔄 Complete Workflow with Flow Visualization

```bash
# 1. Check API health
curl http://localhost:3001/api/v1/health

# 2. List available benchmarks
curl http://localhost:3001/api/v1/benchmarks

# 3. Run a benchmark (generates session with tool calls)
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "local"
  }'

# Response: {"execution_id":"uuid-here","status":"started"}

# 4. Check execution status
curl http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/uuid-here

# 5. Once complete, get the flow diagram using session_id
curl http://localhost:3001/api/v1/flows/{session-id}

# 6. Alternative: Generate flow diagram from tool_calls.log directly
cargo run --bin flow_visualizer -- --input logs/tool_calls.log --html
```

## 🎯 Flow Visualization Sources

There are two ways to generate flow diagrams:

1. **Session-based API**: `/flows/{session-id}` - Uses session logs from database
2. **Tool Log CLI**: `cargo run --bin flow_visualizer` - Uses `logs/tool_calls.log` directly

Both methods generate the same Mermaid stateDiagram format showing agent decision flows and tool execution patterns.
