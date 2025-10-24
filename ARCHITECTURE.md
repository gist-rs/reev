# ARCHITECTURE.md

## Core Flow
```
tui/api → runner → agent → tools → protocols → jupiter → surfpool → score
```

## Services & Ports
- **reev-tui**: Interactive terminal UI (port: none)
- **reev-api**: REST API server (port: 3001)
- **reev-runner**: CLI orchestrator (port: none)
- **reev-agent**: LLM service (port: 9090)
- **surfpool**: Forked mainnet (port: 8899)

## Component Layers

### Entry Points
- `reev-tui`: Terminal UI for benchmark execution
- `reev-api`: REST API for web interface
- `reev-runner`: CLI tool for direct execution

### Core Runner (`reev-runner`)
- Orchestrates benchmark execution
- Manages dependencies (agent + surfpool)
- Handles flow orchestration (multi-step)
- Session logging to database

### Agent Service (`reev-agent`)
- Routes to LLM models (OpenAI/GLM/Local/ZAI)
- Provides tools to AI agents
- OpenTelemetry integration for flow tracking
- HTTP API for runner communication

### Protocol Stack
```
reev-tools → reev-protocols → jupiter-sdk → surfpool
```
- `reev-tools`: Tool wrappers for AI agents
- `reev-protocols`: Protocol implementations
- `jupiter-sdk`: Jupiter DeFi operations
- `surfpool`: Mainnet fork simulation

### Execution & Scoring
```
surfpool → SolanaEnv → scoring → database
```
- `surfpool`: Mock RPC with mainnet state
- `SolanaEnv`: Transaction execution environment
- `scoring`: Two-tier system (75% instruction + 25% execution)
- `database`: Session and result persistence

## Key Data Structures

### Agent Request Flow
1. Runner loads benchmark YAML
2. Runner starts agent service (port 9090)
3. Runner sends prompt + account context to agent
4. Agent routes to appropriate LLM model
5. LLM generates response with tool calls
6. Tools execute via Jupiter SDK
7. Transactions sent to surfpool (port 8899)
8. Results scored and stored

## Ground Truth Data Separation Rules

### 🚨 Critical Architecture Principle
**Ground truth data MUST be separated from real-time context resolution** to prevent information leakage and ensure valid multi-step flow evaluation.

### ✅ Valid Ground Truth Usage
1. **Test Mode**: Use `ground_truth` for fast validation without surfpool
2. **Scoring System**: Use `ground_truth` for final result validation
3. **Deterministic Mode**: Use `ground_truth` for reproducible test behavior

### ❌ Invalid Ground Truth Usage  
1. **LLM Mode**: Ground truth injected into context leaks future information
2. **Context Resolution**: Real blockchain state gets corrupted by expected outcomes
3. **Multi-Step Logic**: Steps become predetermined instead of reactive

### 🛡️ Implementation Rules

#### In Test Files (benchmarks/*.yml)
- `ground_truth`: Final expected state for validation and scoring
- `initial_state`: Starting blockchain state for test setup

#### In FlowAgent (production mode)
```rust
// ❌ WRONG - Leaks future information
context_resolver.resolve_initial_context(
    &initial_state,
    &serde_json::to_value(&benchmark.ground_truth).unwrap_or_default(), // GROUND TRUTH LEAK!
    None,
).await

// ✅ CORRECT - Real-time state only
let ground_truth_for_context = if is_deterministic_mode() {
    Some(&benchmark.ground_truth)
} else {
    None // LLM gets real blockchain state
};

context_resolver.resolve_initial_context(
    &initial_state,
    ground_truth_for_context, // No leakage in LLM mode
    None,
).await
```

#### Mode Detection
```rust
fn is_deterministic_mode() -> bool {
    // Check agent type, environment variable, or benchmark tag
    matches!(agent_name, "deterministic") || 
    std::env::var("REEV_DETERMINISTIC").is_ok() ||
    benchmark.tags.contains(&"deterministic".to_string())
}
```

#### Validation Rules
```rust
// Prevent ground truth usage in LLM mode
if !is_deterministic_mode() && !benchmark.ground_truth.is_null() {
    return Err(anyhow!("Ground truth not allowed in LLM mode"));
}
```

### Tool Categories
- **Native**: SOL transfers (program_id: 111111111...)
- **SPL**: Token operations (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
- **Jupiter**: Swap/lend/earn (JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4)

### Response Formats
- **Jupiter**: `{"transactions": [{"instructions": [...], "completed": true}]}`
- **Simple**: `{"transactions": [{"program_id": "...", "accounts": [...], "data": "..."}]}`

## Critical Integration Points

### Agent Selection Logic
```rust
match agent_name {
    "deterministic" => hardcoded_responses,
    "local" => localhost:1234/v1 (LM Studio),
    "glm-4.6" | "glm-4.6-coding" => ZAI API,
    _ => OpenAI API,
}
```

### Dependency Management
- Runner auto-starts/stops agent (9090) and surfpool (8899)
- Health checks before benchmark execution
- Graceful shutdown on completion

### Scoring System
- Instruction Score (75%): Compare generated vs expected instructions
- On-Chain Score (25%): Transaction execution success/failure
- API benchmarks: `skip_instruction_validation: true` = full score if tools succeed

## File Locations
- Benchmarks: `benchmarks/*.yml`
- Tools: `crates/reev-tools/src/tools/`
- Protocols: `crates/reev-protocols/src/`
- Jupiter SDK: `protocols/jupiter/jup-sdk/`
- Agent: `crates/reev-agent/src/`
- Runner: `crates/reev-runner/src/`

## Environment Variables
- `ZAI_API_KEY`: GLM API access
- `LOCAL_MODEL_NAME`: Custom local model name
- `REEV_SESSION_LOG_PATH`: Session log directory
- `DATABASE_PATH`: SQLite database location

## Common Failure Patterns
1. **Port conflicts**: Services not cleaned up properly
2. **Agent routing**: Wrong model selection for environment
3. **Response parsing**: Jupiter vs simple format mismatch
4. **Tool execution**: Missing API keys or network issues
5. **Scoring**: Instruction validation failures