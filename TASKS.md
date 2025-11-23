# Reev Core Implementation Tasks

## 🎯 **Why: Third Implementation with Code Reuse**

This is our third implementation attempt of the verifiable AI-generated DeFi flows architecture. We have working code in previous implementations that must be reused - not migrated or rewritten. The goal is to consolidate working functionality into the new architecture outlined in PLAN_CORE_V2.md.

## 🔄 **Architecture Overview**

```
User Prompt → [reev-core/planner] → YML Flow → [reev-core/executor] → Tool Calls → [reev-orchestrator] → Execution
```

### Crate Structure:
- **reev-core**: Core architecture with planner/executor modules
- **reev-orchestrator**: Simplified execution orchestrator (reuse existing)
- **reev-planner**: Module within reev-core (not separate crate)

## 📋 **Implementation Tasks**

### Task 1: Create reev-core Crate (Day 1)

**Objective**: Establish core architecture with YML schemas and validation

**Implementation Steps**:
1. Create `reev/crates/reev-core/Cargo.toml` with dependencies:
   - `serde` for YML serialization
   - `uuid` v7 for time-sortable IDs
   - `anyhow` for error handling
   - `reev-types` for shared types

2. Implement basic YML schemas in `reev-core/src/yml_schema.rs`:
   - `WalletContext`: Subject wallet info with tokens
   - `FlowStep`: Prompt and context for each step
   - `GroundTruth`: Assertions and validation criteria

3. Create `reev-core/src/lib.rs` with module exports:
   ```rust
   pub mod yml_schema;
   pub mod planner;
   pub mod executor;
   pub mod context;
   pub mod validation;
   ```

**Code Reuse Guidance**:
- Reuse YML structures from `reev-orchestrator/src/gateway.rs`
- Adapt from existing `DynamicFlowPlan` in `reev-types`
- Use existing wallet context patterns from `reev-orchestrator/src/context_resolver.rs`

### Task 2: Implement Planner Module (Day 2)

**Objective**: Phase 1 LLM integration for structured YML generation

**Implementation Steps**:
1. Create `reev-core/src/planner.rs` with:
   - `Planner` struct with LLM client
   - `refine_and_plan()` method for Phase 1
   - Language refinement and intent analysis

2. Implement wallet context handling:
   - Production mode: Use provided wallet context
   - Benchmark mode: Handle `USER_WALLET_PUBKEY` placeholder

3. Add LLM prompt generation:
   - Minimal structured YML for LLM input
   - Tool context integration
   - Response parsing into structured YML

**Code Reuse Guidance**:
- Reuse LLM client from `reev-agent`
- Adapt from `refine_prompt()` in `reev-orchestrator/src/gateway.rs`
- Use existing context patterns from `reev-orchestrator/src/context_resolver.rs`
- Benchmark mode logic from existing orchestrator

### Task 3: Implement Executor Module (Day 3)

**Objective**: Phase 2 tool execution with parameter generation

**Implementation Steps**:
1. Create `reev-core/src/executor.rs` with:
   - `Executor` struct for tool execution
   - Tool-specific parameter generation
   - Step-by-step execution with validation

2. Implement error recovery:
   - Atomic flow with `flow[step]` like `tx[ix]`
   - Network error retry (once)
   - Slippage error retry with refined parameters

3. Add ground truth validation:
   - Pre-execution guardrails
   - Post-execution state verification
   - Error tolerance for slippage (1%)

**Code Reuse Guidance**:
- Reuse tool calling logic from `reev-tools`
- Adapt from `execute_flow_with_recovery()` in `reev-orchestrator/src/gateway.rs`
- Use existing error handling patterns from `reev-orchestrator/src/recovery.rs`

### Task 4: Refactor reev-orchestrator (Day 4)

**Objective**: Simplify orchestrator to use reev-core components

**Implementation Steps**:
1. Update `reev-orchestrator/Cargo.toml` to depend on `reev-core`
2. Remove redundant code from `reev-orchestrator/src/gateway.rs`:
   - Remove planning logic (now in reev-core/planner)
   - Remove context resolution (now in reev-core)
   - Keep only execution, recovery, and OpenTelemetry

3. Simplify `OrchestratorGateway`:
   - Use `reev_core::Planner` for Phase 1
   - Use `reev_core::Executor` for Phase 2
   - Focus on flow execution and recovery

**Code Reuse Guidance**:
- Keep all existing execution logic
- Keep all recovery mechanisms
- Keep all OpenTelemetry integration
- Remove only planning and context resolution (moved to reev-core)

### Task 5: Integration Testing (Day 5)

**Objective**: End-to-end testing with language variations

**Implementation Steps**:
1. Create integration tests in `tests/`:
   - Basic swap in English
   - Swap with typo ("swp")
   - Swap in different language ("แลก")
   - Multi-step flow with context awareness

2. Test error recovery:
   - Network error scenarios
   - Slippage within tolerance
   - Critical step failures

3. Benchmark mode testing:
   - `USER_WALLET_PUBKEY` resolution
   - SURFPOOL integration
   - Deterministic behavior

**Code Reuse Guidance**:
- Adapt from existing tests in `reev-orchestrator/tests/`
- Use existing test patterns and helpers
- Reuse mock patterns from current implementation

## 🔄 **Code Reuse Strategy**

### Critical Reuse (Do Not Rewrite):
1. **LLM Client Integration**: `reev-agent` - fully working
2. **Tool Execution**: `reev-tools` - fully working
3. **Recovery Engine**: `reev-orchestrator/src/recovery.rs` - fully working
4. **OpenTelemetry Integration**: `reev-orchestrator` - fully working
5. **SURFPOOL Integration**: existing patterns - fully working

### Adapt and Refactor (Not Replace):
1. **YML Structures**: From `reev-orchestrator/src/gateway.rs` - adapt to new schema
2. **Context Resolution**: From `reev-orchestrator/src/context_resolver.rs` - simplify and move
3. **Flow Generation**: From `reev-orchestrator/src/gateway.rs` - split between planner/executor

### Remove Only:
1. **Rule-Based Pattern Matching**: From `reev-orchestrator/src/gateway.rs` - replace with LLM
2. **Direct Tool Execution**: From `reev-orchestrator` - replace with two-phase approach

## 🎯 **Success Criteria**

### Functional Requirements:
- ✅ Handle any language or typos in user prompts
- ✅ Generate valid, structured YML flows
- ✅ Execute flows with proper verification
- ✅ Apply ground truth guardrails during execution

### Code Quality Requirements:
- ✅ Maximum reuse of existing working code
- ✅ No unnecessary rewrites of working components
- ✅ Clear separation of concerns
- ✅ Minimal changes to existing working components

## 📝 **Handover Notes**

### For Next Implementation:

1. **Start with reev-core**: Implement YML schemas first
2. **Focus on Two-Phase Approach**: Clear separation between planning and execution
3. **Reuse Aggressively**: Don't rewrite what already works
4. **Test Language Variations**: Ensure non-English prompts work
5. **Benchmark Mode**: Test with `USER_WALLET_PUBKEY` placeholder

### Common Pitfalls to Avoid:
1. **Don't Reuse Rule-Based Pattern Matching**: This is what we're replacing
2. **Don't Mix Phases**: Keep planning separate from execution
3. **Don't Ignore Error Tolerance**: Include 1% slippage tolerance
4. **Don't Forget Dual Purpose YML**: Both runtime guardrails AND evaluation criteria

## 🚀 **Next Steps After Implementation**

1. **Create reev-core crate** with YML schemas
2. **Implement planner module** with LLM integration
3. **Implement executor module** with tool execution
4. **Refactor reev-orchestrator** to use new components
5. **Comprehensive testing** with language variations