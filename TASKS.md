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

**PHASES 1-5 COMPLETE**: All context handling improvements implemented and tested

## 🎉 CONTEXT IMPROVEMENT PLAN COMPLETE

### Summary of Changes
We have successfully implemented a comprehensive context resolution system that fixes the core issues:

#### Phase 1: ✅ Context Resolver Module
- Created `crates/reev-context` with centralized context management
- Implements placeholder resolution to real addresses
- Supports multi-step flow context consolidation
- Provides YAML schema validation
- Includes comprehensive test suite

#### Phase 2: ✅ FlowAgent Context Building  
- Integrated ContextResolver into FlowAgent
- Removed duplicate tool creation from create_conditional_toolset()
- Tools are now created only once by model agents with resolved addresses
- Updated context building to use resolved addresses instead of placeholders

#### Phase 3: ✅ Multi-Step Context Management
- Implemented in ContextResolver via `update_context_after_step()`
- FlowAgent tracks context changes between steps
- Step results properly stored for subsequent step dependencies

#### Phase 4: ✅ Tool Creation and Error Types
- Created separate `SplTransferError` enum
- SPL transfer tool now has its own error type
- Base58 parsing errors properly attributed to correct tool
- Fixed shared error enum confusion

#### Phase 5: ✅ Context Validation Tests
- Created comprehensive test suite in `crates/reev-context/tests/context_validation_test.rs`
- Tests cover SOL transfers, SPL transfers, and multi-step flows
- All tests designed to pass without surfpool running
- Validates context schema compliance
- All tests now passing (6/6) ✅

### Root Cause Fixed
The original issue was that FlowAgent was creating tools with placeholder `key_map.clone()` containing names like `"RECIPIENT_WALLET_PUBKEY"` instead of resolved addresses. When SPL transfer tool tried to parse these placeholder names as base58 addresses, it failed with "Invalid Base58 string" error.

**This is now fixed** because:
1. ContextResolver resolves ALL placeholders to real addresses before tool creation
2. Tools receive properly resolved addresses, not placeholder names  
3. Context validation ensures addresses are valid before reaching tools
4. Error types are properly separated for clear attribution

### Files Modified
- `crates/reev-context/` - New context resolver module
- `crates/reev-agent/src/flow/agent.rs` - Updated to use ContextResolver
- `crates/reev-tools/src/tools/native.rs` - Added SplTransferError
- `crates/reev-agent/src/tools/native.rs` - Updated with SplTransferError
- `crates/reev-context/tests/context_validation_test.rs` - Comprehensive validation tests
- `TASKS.md` - Updated with completion status

The system now has robust context handling that will eliminate the "Invalid Base58 string" errors and provide proper multi-step flow support.

### Files Modified
- `crates/reev-context/src/lib.rs` (new)
- `crates/reev-agent/src/flow/agent.rs`
- `crates/reev-tools/src/tools/native.rs`
- `crates/reev-agent/src/tools/native.rs`
- `crates/reev-context/tests/context_validation_test.rs` (new)
- `Cargo.toml` (add reev-context dependency)