# TASKS.md

## Critical Context Handling Fixes

### Issue Analysis
- FlowAgent creates tools with placeholder names (USER_WALLET_PUBKEY, RECIPIENT_WALLET_PUBKEY)
- Tools try to parse placeholder names as base58 addresses → FAILS
- Multi-step flows lack proper context consolidation between steps
- SPL transfer uses wrong error enum (NativeTransferError instead of SplTransferError)

### Phase 1: Create Context Resolver Module ✅
**File**: `crates/reev-context/src/lib.rs`
- [x] Create context resolver that consolidates YAML + surfpool data
- [x] Define YAML schema for context with validation
- [x] Implement placeholder resolution to real addresses
- [x] Add tests for context resolution without LLM calls

### Phase 2: Fix FlowAgent Context Building ✅
**File**: `crates/reev-agent/src/flow/agent.rs`
- [x] Remove duplicate tool creation from create_conditional_toolset()
- [x] Create context resolver before calling run_agent()
- [x] Resolve ALL placeholders to real addresses
- [x] Update context building to use resolved addresses instead of placeholders
- [x] Add tests for resolved context before LLM calls

### Phase 3: Add Multi-Step Context Management ✅
**File**: `crates/reev-agent/src/flow/agent.rs`
- [x] Track context changes between flow steps
- [x] Consolidate account states after each transaction
- [x] Handle step dependencies (depends_on field)
- [x] Update context for each step based on previous results
- [x] Add tests for multi-step context consolidation

### Phase 4: Fix Tool Creation and Error Types ✅
**Files**: 
- `crates/reev-tools/src/tools/native.rs`
- `crates/reev-agent/src/tools/native.rs`
- [x] Create SplTransferError enum separate from NativeTransferError
- [x] Update SplTransferTool to use SplTransferError
- [x] Fix base58 parsing to use resolved addresses
- [x] Add tests for error handling with real addresses

### Phase 5: Add Context Validation Tests ✅
**File**: `crates/reev-context/tests/context_validation_test.rs`
- [x] Test all benchmark YAML files context resolution
- [x] Validate context schema compliance
- [x] Test placeholders are fully resolved
- [x] Test multi-step flow context consolidation
- [x] Run tests without LLM calls to ensure correctness

### Acceptance Criteria
1. All placeholders resolved to real addresses before tool execution ✅
2. Context validation passes for all benchmarks without LLM calls ✅
3. Multi-step flows properly consolidate context between steps ✅
4. No more "Invalid Base58 string" errors ✅
5. Each phase has passing tests and commits ✅



### Files Modified
- `crates/reev-context/src/lib.rs` (new)
- `crates/reev-agent/src/flow/agent.rs`
- `crates/reev-tools/src/tools/native.rs`
- `crates/reev-agent/src/tools/native.rs`
- `crates/reev-context/tests/context_validation_test.rs` (new)
- `crates/reev-context/tests/benchmark_context_validation.rs` (fixed)
- `Cargo.toml` (add reev-context dependency)

### Phase 6: Fix SPL Token Amount YAML Output ✅
**Files**: 
- `crates/reev-context/tests/benchmark_context_validation.rs`
- **Issue**: Mock context creation failed to parse YAML Number values, only handled strings
- **Root Cause**: `value.as_str()` check failed for `Number(50000000)` YAML values
- **Fix**: Enhanced parsing to handle Numbers, Strings, Booleans, and fallback conversion
- **Result**: SPL token amounts now appear in YAML context for LLM decisions
- **Validation**: Added comprehensive tests for both mock and production context resolver