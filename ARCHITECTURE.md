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

## YAML Ground Truth Testing Strategy

### 🧪 Two-Tier Validation Approach

#### 1. Context Validation Tests (`benchmark_context_validation.rs`)
**Purpose**: Test LLM input format without external dependencies
- **Scope**: Context preparation and YAML format validation
- **Dependencies**: None (completely self-contained)
- **Ground Truth**: Extracted but intentionally ignored (`_ground_truth`)
- **Use Case**: Ensure LLM receives correct context format
- **Command**: `cargo test -p reev-context --test benchmark_context_validation -- --nocapture`

#### 2. YAML Ground Truth Tests (`benchmark_yaml_validation.rs`)
**Purpose**: End-to-end YAML validation without surfpool
- **Scope**: Complete benchmark structure validation
- **Dependencies**: None (ground truth only, no external services)
- **Ground Truth**: Actively applied and validated
- **Use Case**: Verify benchmark YAMLs are well-formed and self-contained
- **Command**: `cargo test -p reev-context --test benchmark_yaml_validation -- --nocapture`

#### 3. Ground Truth Separation Tests (`ground_truth_separation_test.rs`)
**Purpose**: Validate architectural principle of no information leakage
- **Scope**: Mode detection and ground truth access control
- **Dependencies**: Agent service only
- **Coverage**: 6 comprehensive test cases
- **Use Case**: Ensure deterministic vs LLM mode separation works correctly
- **Command**: `cargo test -p reev-agent --test ground_truth_separation_test -- --nocapture`

#### 4. Integration Tests (`benchmarks_test.rs`) 
**Purpose**: End-to-end validation of ALL benchmarks against surfpool
- **Scope**: Complete system integration with real Solana programs
- **Dependencies**: surfpool (mainnet fork) + agent service
- **Coverage**: All `benchmarks/*.yml` files with configurable agents
- **Use Case**: Production readiness validation with real blockchain state
- **Command**: `cargo test -p reev-runner benchmarks_test -- --nocapture`

### 🎯 Test Execution Strategy

#### Before Production Changes:
```bash
# 1. Validate YAML structure and ground truth
cargo test -p reev-context --test benchmark_yaml_validation -- --nocapture

# 2. Validate context preparation for LLM
cargo test -p reev-context --test benchmark_context_validation -- --nocapture

# 3. Validate ground truth separation architecture
cargo test -p reev-agent --test ground_truth_separation_test -- --nocapture

# 4. Validate complete integration with surfpool
cargo test -p reev-runner benchmarks_test -- --nocapture
```

#### Continuous Integration:
- **Fast feedback**: YAML validation tests (no external deps)
- **Full validation**: All three test suites
- **Security check**: Ground truth separation tests must pass

### 📋 Testing Checklist for YAML Changes

When modifying benchmark YAML files:
1. ✅ **Structure Validation**: `benchmark_yaml_validation.rs` passes
2. ✅ **Context Generation**: `benchmark_context_validation.rs` passes  
3. ✅ **No Information Leakage**: `ground_truth_separation_test.rs` passes
4. ✅ **Integration Testing**: `benchmarks_test.rs` passes with surfpool
5. ✅ **Clippy Compliance**: `cargo clippy --fix --allow-dirty` clean
6. ✅ **Compilation**: `cargo check` succeeds

## File Locations
- Benchmarks: `benchmarks/*.yml`
- Tools: `crates/reev-tools/src/tools/`
- Protocols: `crates/reev-protocols/src/`
- Jupiter SDK: `protocols/jupiter/jup-sdk/`
- Agent: `crates/reev-agent/src/`
- Runner: `crates/reev-runner/src/`
- Context Tests: `crates/reev-context/tests/`
- Ground Truth Tests: `crates/reev-agent/tests/`
- Integration Tests: `crates/reev-runner/tests/`

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
6. **Ground Truth Leakage**: Incorrect mode detection causing information leak
7. **YAML Structure**: Invalid benchmark format failing validation
8. **Context Resolution**: Placeholder resolution failures in test environment
9. **Integration Failures**: surfpool unavailability or agent startup issues
10. **Benchmark Execution**: Transaction failures in real mainnet fork environment

## Ground Truth Testing Troubleshooting

### Environment Variable Conflicts
```bash
# Clean environment before running tests
unset REEV_DETERMINISTIC
cargo test -p reev-agent --test ground_truth_separation_test -- --nocapture
```

### Test Isolation Issues
- Tests use `serial_test` crate to prevent interference
- Environment variables set/cleaned per test
- Run individually for debugging: `cargo test test_name`

### YAML Validation Failures
- Check `initial_state` structure (required fields: pubkey, owner, lamports)
- Verify `ground_truth` contains proper `final_state_assertions`
- Ensure JSON/YML syntax is valid

### Context Resolution Errors
- Placeholder names must be consistent across YAML sections
- Mock address generation handles unknown placeholders
- Key mapping verified between initial state and ground truth assertions

### Integration Test Failures
```bash
# Check surfpool availability
curl -X POST http://127.0.0.1:8899 -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# Check agent availability  
curl http://127.0.0.1:9090/health

# Run integration tests with specific agent
cargo test -p reev-runner benchmarks_test -- --nocapture --agent gpt-4
```

### Agent Configuration Issues
```bash
# Available agents for integration tests:
cargo run -p reev-runner -- --agent deterministic benchmarks/    # Perfect responses
cargo run -p reev-runner -- --agent gpt-4 benchmarks/          # OpenAI
cargo run -p reev-runner -- --agent glm-4.6 benchmarks/        # GLM via ZAI
cargo run -p reev-runner -- --agent local benchmarks/            # Local LM Studio
```