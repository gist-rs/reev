# TASKS.md

## Critical Context Handling Fixes

### Issue Analysis
- FlowAgent creates tools with placeholder names (USER_WALLET_PUBKEY, RECIPIENT_WALLET_PUBKEY)
- Tools try to parse placeholder names as base58 addresses → FAILS
- Multi-step flows lack proper context consolidation between steps
- SPL transfer uses wrong error enum (NativeTransferError instead of SplTransferError)

### Phase 1: Create Context Resolver Module
**File**: `crates/reev-context/src/lib.rs`
- [ ] Create context resolver that consolidates YAML + surfpool data
- [ ] Define YAML schema for context with validation
- [ ] Implement placeholder resolution to real addresses
- [ ] Add tests for context resolution without LLM calls

### Phase 2: Fix FlowAgent Context Building
**File**: `crates/reev-agent/src/flow/agent.rs`
- [ ] Remove duplicate tool creation from create_conditional_toolset()
- [ ] Create context resolver before calling run_agent()
- [ ] Resolve ALL placeholders to real addresses
- [ ] Update build_context_prompt_with_keymap() to use resolved addresses
- [ ] Add tests for resolved context before LLM calls

### Phase 3: Add Multi-Step Context Management
**File**: `crates/reev-agent/src/flow/agent.rs`
- [ ] Track context changes between flow steps
- [ ] Consolidate account states after each transaction
- [ ] Handle step dependencies (depends_on field)
- [ ] Update context for each step based on previous results
- [ ] Add tests for multi-step context consolidation

### Phase 4: Fix Tool Creation and Error Types
**Files**: 
- `crates/reev-tools/src/tools/native.rs`
- `crates/reev-agent/src/tools/native.rs`
- [ ] Create SplTransferError enum separate from NativeTransferError
- [ ] Update SplTransferTool to use SplTransferError
- [ ] Fix base58 parsing to use resolved addresses
- [ ] Add tests for error handling with real addresses

### Phase 5: Add Context Validation Tests
**File**: `tests/context_validation_test.rs`
- [ ] Test all benchmark YAML files context resolution
- [ ] Validate context schema compliance
- [ ] Test placeholders are fully resolved
- [ ] Test multi-step flow context consolidation
- [ ] Run tests without LLM calls to ensure correctness

### Acceptance Criteria
1. All placeholders resolved to real addresses before tool execution
2. Context validation passes for all benchmarks without LLM calls
3. Multi-step flows properly consolidate context between steps
4. No more "Invalid Base58 string" errors
5. Each phase has passing tests and commits

### Files to Modify
- `crates/reev-context/src/lib.rs` (new)
- `crates/reev-agent/src/flow/agent.rs`
- `crates/reev-tools/src/tools/native.rs`
- `crates/reev-agent/src/tools/native.rs`
- `tests/context_validation_test.rs` (new)
- `Cargo.toml` (add reev-context dependency)