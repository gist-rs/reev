# Reev Core Implementation Tasks

## 🎯 **Why: Third Implementation with Code Reuse**

This is our third implementation attempt of the verifiable AI-generated DeFi flows architecture. We have working code in previous implementations that must be reused - not migrated or rewritten. The goal is to consolidate working functionality into the new architecture outlined in PLAN_CORE_V2.md.

## 🔄 **Current Implementation Status**

```
User Prompt → [reev-core/planner] → YML Flow → [reev-core/executor] → Tool Calls → [reev-orchestrator] → Execution
```

### Crate Structure:
- **reev-core**: ✅ Core architecture with planner/executor modules implemented
- **reev-orchestrator**: ✅ Refactored to use reev-core components
- **reev-planner**: ⚠️ Module within reev-core exists but uses rule-based fallback

### Critical Gaps:
- **LLM Integration**: ❌ Planner has trait but no implementation
- **Tool Execution**: ❌ Executor returns mock results instead of executing tools
- **Testing**: ⚠️ Database locking issues prevent comprehensive testing

## 📋 **Implementation Status**

### Task 1: Create reev-core Crate (COMPLETED ✅)

**Status**: Fully Implemented

**Implementation**:
- ✅ Created `reev/crates/reev-core/Cargo.toml` with dependencies
- ✅ Implemented comprehensive YML schemas in `reev-core/src/yml_schema.rs`
- ✅ Created module exports in `reev-core/src/lib.rs`
- ✅ Added test coverage (31 tests passing)

**Code Reused**:
- YML structures from `reev-orchestrator/src/gateway.rs`
- Adapted from existing `DynamicFlowPlan` in `reev-types`
- Wallet context patterns from `reev-orchestrator/src/context_resolver.rs`

### Task 2: Implement Planner Module (PARTIALLY COMPLETED ⚠️)

**Status**: Structure in Place, Core Functionality Missing

**Current Implementation**:
- ✅ Created `reev-core/src/planner.rs` with proper structure
- ✅ Implemented `refine_and_plan()` method signature
- ✅ Added wallet context handling for production/benchmark modes
- ✅ Implemented rule-based fallback for simple flows
- ❌ No actual LLM client implementation (only trait exists)
- ❌ LLM-based flow generation not implemented

**Key Finding**:
- Existing GLM implementation found in `reev-agent/src/enhanced/zai_agent.rs`
- Unified GLM logic exists in `reev-agent/src/enhanced/common/mod.rs`
- Can leverage GLM-4.6-coding model via ZAI API

### Task 3: Implement Executor Module (PARTIALLY COMPLETED ⚠️)

**Status**: Structure in Place, Core Functionality Missing

**Current Implementation**:
- ✅ Created `reev-core/src/executor.rs` with proper structure
- ✅ Implemented step-by-step execution framework
- ✅ Added error recovery structure with configurable retry strategies
- ✅ Implemented conversion between YML flows and DynamicFlowPlan
- ❌ No actual tool execution (stub implementation returns mock results)
- ❌ Tool execution engine integration missing

**Key Finding**:
- Existing tool implementations available in `reev-tools/src/lib.rs`
- Agent integration already exists via AgentTools in `reev-agent/src/enhanced/common/mod.rs`
- Can reuse existing tool calling patterns

### Task 4: Refactor reev-orchestrator (COMPLETED ✅)

**Status**: Fully Implemented

**Implementation**:
- ✅ Updated `reev-orchestrator/Cargo.toml` to depend on `reev-core`
- ✅ Refactored `OrchestratorGateway` to use reev-core components
- ✅ Updated `process_user_request` to use reev-core planner
- ✅ Added conversion methods between YML flows and DynamicFlowPlan
- ✅ Updated constructor methods to initialize reev-core components
- ✅ Added integration tests for reev-core integration

**Code Reused**:
- Kept all existing execution logic
- Kept all recovery mechanisms
- Kept all OpenTelemetry integration
- Removed only planning and context resolution (moved to reev-core)

### Task 5: Integration Testing (PARTIALLY COMPLETED ⚠️)

**Status**: Basic Tests Only, Database Issues Remain

**Current Implementation**:
- ✅ Created 2 integration tests in `orchestrator_refactor_test.rs`
- ✅ `test_reev_core_integration` - PASSED
- ✅ `test_reev_core_benchmark_mode` - PASSED
- ❌ Many other tests failing with "database is locked" errors
- ❌ Removed failing tests from `integration_tests.rs`
- ❌ No end-to-end testing with actual agent and tools

**Test Issues**:
- Database locking errors in `orchestrator_tests.rs`
- Tests in `integration_tests.rs` had to be removed
- No testing of actual LLM integration or tool execution

## 🔄 **Code Reuse Strategy**

### Successfully Reused (Not Rewritten):
1. **YML Structures**: ✅ From `reev-orchestrator/src/gateway.rs` - adapted to new schema
2. **Context Resolution**: ✅ From `reev-orchestrator/src/context_resolver.rs` - simplified and moved
3. **Recovery Engine**: ✅ `reev-orchestrator/src/recovery.rs` - kept working
4. **OpenTelemetry Integration**: ✅ `reev-orchestrator` - kept working
5. **SURFPOOL Integration**: ✅ Existing patterns - kept working

### Found Existing Components (Can Leverage):
1. **LLM Client Integration**: ✅ `reev-agent/src/enhanced/zai_agent.rs` - GLM-4.6-coding model
2. **Unified GLM Logic**: ✅ `reev-agent/src/enhanced/common/mod.rs` - unified agent logic
3. **Tool Execution**: ✅ `reev-tools/src/lib.rs` - existing tool implementations
4. **Agent Integration**: ✅ `reev-agent/src/enhanced/common/mod.rs` - AgentTools integration

### Still Needs Implementation:
1. **LLM Integration for Planner**: ❌ Connect planner to GLM-4.6-coding model
2. **Tool Execution for Executor**: ❌ Connect executor to reev-tools
3. **Database Testing Issues**: ❌ Fix database locking in test suite
4. **End-to-End Testing**: ❌ Test with actual agent and tools

## 🎯 **Success Criteria - Current Status**

### Functional Requirements:
- ❌ Handle any language or typos in user prompts (LLM integration missing)
- ❌ Generate valid, structured YML flows (LLM integration missing)
- ❌ Execute flows with proper verification (tool execution missing)
- ⚠️ Apply ground truth guardrails during execution (structure exists, no execution)

### Code Quality Requirements:
- ✅ Maximum reuse of existing working code
- ✅ Clear separation of concerns
- ✅ Minimal changes to existing working components

## 📝 **Next Critical Steps**

1. **Implement LLM Integration for Planner** (Issue #64)
   - Create LlmClient implementation using existing GLM-4.6-coding model
   - Leverage UnifiedGLMAgent for context building and wallet handling
   - Implement flow-specific prompt template for YML generation

2. **Implement Tool Execution for Executor** (Issue #65)
   - Integrate reev-tools in executor module for actual tool execution
   - Leverage AgentTools from reev-agent for tool calling
   - Implement real tool execution instead of mock results

3. **Fix Database Testing Issues** (Issues #66, #69)
   - Identify root cause of database locking
   - Fix test isolation in remaining test files
   - Remove or fix failing tests in orchestrator_tests.rs

4. **Implement End-to-End Testing** (Issue #68)
   - Create tests with real LLM and tool execution
   - Test with real wallet addresses and tokens
   - Verify complete flows from prompt to execution

5. **Remove Deprecated Code** (Issue #67)
   - Remove deprecated or unused code
   - Clean up unused imports and dead code
   - Update documentation to reflect current architecture