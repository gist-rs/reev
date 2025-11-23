## Issue #64: Implement Actual LLM Client Integration for Planner

### Status: NOT STARTED

### Description:
The planner module has the structure in place but lacks actual LLM client implementation. Currently it only has a trait definition and falls back to rule-based pattern matching.

### Current State:
```rust
// planner.rs - only a trait exists
pub trait LlmClient: Send + Sync {
    async fn generate_flow(&self, prompt: &str) -> Result<String>;
}
```

### Existing GLM Implementation Found:
- **Location**: `reev/crates/reev-agent/src/enhanced/zai_agent.rs` and `openai.rs`
- **Models**: GLM-4.6 and GLM-4.6-coding via ZAI API
- **Unified Logic**: `UnifiedGLMAgent` in `reev/crates/reev-agent/src/enhanced/common/mod.rs`
- **Pattern**: GLM models use unified logic with provider-specific request/response handling

### Tasks Required:
1. **Create LlmClient implementation** using existing GLM-4.6-coding model via ZAI
2. **Leverage UnifiedGLMAgent** for context building and wallet handling
3. **Implement flow-specific prompt template** for YML generation
4. **Connect planner to actual LLM client** instead of rule-based fallback
5. **Test LLM-based flow generation** with various language prompts and typos
6. **Remove rule-based fallback** once LLM integration is confirmed working

### Success Criteria:
- Planner generates YML flows using actual GLM-4.6-coding model instead of rules
- Can handle typos and language variations through LLM
- Maintain backward compatibility with existing interfaces
- Integration uses existing authentication (ZAI_API_KEY)

---

## Issue #65: Implement Actual Tool Execution for Executor

### Status: NOT STARTED

### Description:
The executor module has structure in place but tool execution is completely stubbed out. Currently it only returns mock results.

### Current State:
```rust
// executor.rs - execute_step_with_recovery is a stub
async fn execute_step_with_recovery(
    &self,
    step: &DynamicStep,
    _previous_results: &[StepResult],
) -> Result<StepResult> {
    // Creates mock results without actual execution
```

### Existing Tool Implementation Found:
- **Location**: `reev/crates/reev-tools/src/lib.rs`
- **Tools Available**: SolTransferTool, SplTransferTool, JupiterSwapTool, JupiterLendEarnDepositTool, etc.
- **Agent Integration**: Tools are already integrated with reev-agent via AgentTools in `reev/crates/reev-agent/src/enhanced/common/mod.rs`

### Tasks Required:
1. **Integrate reev-tools** in executor module for actual tool execution
2. **Leverage AgentTools** from reev-agent for tool calling
3. **Implement real tool execution** instead of mock results
4. **Test with various tools** (jupiter_swap, jupiter_lend_earn_deposit, etc.)
5. **Implement error recovery scenarios** with actual tool failures
6. **Add proper context passing** between steps

### Success Criteria:
- Executor calls actual tools via reev-tools
- Real tool results are returned instead of mocks
- Error recovery works with actual tool failures
- Step results contain real tool outputs
- Reuse existing tool implementations from reev-tools

---

## Issue #66: Fix Database Locking Issues in Tests

### Status: NOT STARTED

### Description:
Many tests are failing with "database is locked" errors, preventing comprehensive testing.

### Error Pattern:
```
Error: Schema error: Failed to execute schema statement: CREATE TABLE IF NOT EXISTS execution_sessions
Caused by:
    SQL execution failure: `database is locked`
```

### Tasks Required:
1. **Identify root cause** of database locking issues
2. **Implement proper database cleanup** between tests
3. **Fix concurrent test execution** to avoid conflicts
4. **Ensure isolated test environments** for parallel test execution
5. **Add database connection pooling** if needed

### Success Criteria:
- All tests can run without database locking errors
- Tests can be run in parallel without conflicts
- Database state is properly isolated between tests

---

## Issue #67: Remove Deprecated/Unused Code

### Status: NOT STARTED

### Description:
There may be deprecated or unused code that can be removed to simplify the codebase.

### Tasks Required:
1. **Identify deprecated code** that's no longer used after reev-core integration
2. **Remove unused test files** that fail due to database issues
3. **Clean up unused imports** and dead code
4. **Consolidate duplicate functionality** between old and new implementations
5. **Update documentation** to reflect current architecture

### Success Criteria:
- Cleaner codebase with only necessary code
- No duplicate functionality
- Updated documentation reflecting current state

---

## Issue #69: Fix Database Locking in Remaining Tests

### Status: IDENTIFIED

### Description:
Additional test files are still failing with database locking issues after removing tests from integration_tests.rs.

### Test Files with Database Locking Issues:
- `reev/crates/reev-orchestrator/tests/orchestrator_tests.rs`
  - test_prompt_refinement - FAILED
  - test_swap_flow_generation - FAILED
  - test_swap_lend_flow_generation - FAILED

### Error Pattern:
```
Schema error: Failed to execute schema statement: CREATE TABLE IF NOT EXISTS benchmarks
Caused by:
    SQL execution failure: `database is locked`
```

### Tasks Required:
1. **Identify root cause** of database locking in orchestrator_tests.rs
2. **Fix test isolation** in remaining test files
3. **Remove or fix failing tests** in orchestrator_tests.rs
4. **Ensure database cleanup** between test runs
5. **Update test runner configuration** if needed

### Success Criteria:
- All tests in orchestrator_tests.rs pass without database locking
- Database state is properly isolated between tests
- Tests can run in parallel without conflicts

---


## Issue #68: Implement End-to-End Testing with Real Agent and Tools

### Status: NOT STARTED

### Description:
Currently only basic integration tests exist. We need comprehensive end-to-end testing with actual LLM and tool execution.

### Tasks Required:
1. **Create end-to-end tests** that use real LLM and tool execution
2. **Test with real wallet addresses** and tokens
3. **Verify complete flows** from prompt to execution
4. **Test error scenarios** and recovery
5. **Benchmark performance** against success criteria

### Success Criteria:
- End-to-end flows work with real components
- Performance meets specified criteria (Phase 1 < 2s, Phase 2 < 1s per call)
- 90%+ success rate on common flows
- Comprehensive test coverage for real scenarios

---