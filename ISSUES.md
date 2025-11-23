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

### Why Mock Implementation Appears in the Plan:
1. **Transitional Implementation**: As stated in IMPLEMENTATION_STATUS.md, current implementation has structure but lacks core functionality
2. **Avoiding Real Integration**: Multiple attempts to implement have resulted in mock instead of using existing GLM code
3. **Testing Focus**: Mock is used for testing but is leaking into production paths
4. **Code Duplication**: Creating new implementations instead of leveraging existing working code

### Tasks Required:
1. **STOP creating mock implementations** - they prove nothing and are unusable for production
2. **Use existing GLM implementation** from `reev-agent/src/enhanced/zai_agent.rs`
3. **Create real LlmClient implementation** using GLM-4.6-coding model via ZAI
4. **Leverage UnifiedGLMAgent** for context building and wallet handling
5. **Implement flow-specific prompt template** for YML generation
6. **Connect planner to actual LLM client** instead of rule-based fallback
7. **Test LLM-based flow generation** with various language prompts and typos
8. **Remove rule-based fallback** once LLM integration is confirmed working

### Environment Variable Configuration:
1. **SOLANA_PRIVATE_KEY**: Accept path to id.json file instead of direct key string
   - If not set, check `~/.config/solana/id.json` as default
   - This is NOT IMPLEMENTED YET

### Success Criteria:
- Planner uses actual GLM-4.6-coding model via ZAI API
- YML flows are generated from user prompts in any language/with typos
- Rule-based fallback is completely removed
- Mock implementations are moved to tests folder ONLY
- All existing GLM code is reused without duplication

## Issue #65: Implement Real Tool Execution for Executor

### Status: NOT STARTED

### Description:
The executor module returns mock results instead of executing real tools. This makes the entire system unusable for production DeFi operations.

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

### Existing Tool Implementations Available:
- **Location**: `reev-tools/src/lib.rs`
- **Tools**: JupiterSwap, JupiterLendEarnDeposit, etc.
- **Agent Integration**: Already exists via AgentTools in `reev-agent/src/enhanced/common/mod.rs`

### Tasks Required:
1. **STOP creating mock tool results** - they make the system unusable
2. **Use existing tool implementations** from `reev-tools/src/lib.rs`
3. **Connect executor to actual tool execution** via AgentTools
4. **Implement real tool execution** instead of mock results
5. **Test with real DeFi operations** to ensure functionality

### Success Criteria:
- Executor executes real tools via reev-tools
- Mock results are eliminated from production code
- Tool execution results are returned properly
- All existing tool code is reused without duplication

## Issue #66: Fix Environment Variable Configuration

### Status: NOT STARTED

### Description:
Environment configuration doesn't properly support default Solana key location.

### Current Implementation:
```bash
# Current .env.example
SOLANA_PRIVATE_KEY="YOUR_SOLANA_PRIVATE_KEY"
```

### Required Implementation:
1. **Accept path to id.json**: SOLANA_PRIVATE_KEY should accept a file path
2. **Default location check**: If not set, check `~/.config/solana/id.json`
3. **Update documentation**: Clearly document this behavior in .env.example
4. **Implement logic**: Add code to read key from default location if env var not set

### Success Criteria:
- System reads key from SOLANA_PRIVATE_KEY if set (path or direct key)
- If SOLANA_PRIVATE_KEY not set, system checks `~/.config/solana/id.json`
- Documentation clearly explains this behavior
- Both direct key and file path are supported

## Issue #67: Move Mock Implementations to Tests

### Status: NOT STARTED

### Description:
Mock implementations are in production code paths where they can accidentally be used instead of real implementations.

### Current Problem Areas:
1. `crates/reev-core/src/llm/mock/mod.rs` - Mock LLM client
2. Mock tool execution in executor
3. Mock results in various parts of the codebase

### Tasks Required:
1. **Move all mocks to tests folder**: Create `crates/reev-core/tests/common/mock_helpers.rs`
2. **Remove mocks from src**: Ensure no mock code can be used in production
3. **Feature flag for tests**: Only compile mocks in test configuration
4. **Update documentation**: Clearly mark mocks as test-only

### Success Criteria:
- No mock implementations in src/ folders
- All mocks are in tests/ folders
- Production code cannot accidentally use mocks
- Tests still work with moved mocks

## Issue #68: Fix LLM Integration Avoidance Pattern

### Status: NOT STARTED

### Description:
There's a pattern of avoiding real LLM integration by creating new mock implementations instead of using existing working GLM code.

### Evidence of Problem:
1. Existing GLM implementation exists: `crates/reev-agent/src/enhanced/zai_agent.rs`
2. ZAI provider exists: `crates/reev-agent/src/providers/zai/`
3. Yet planner creates new mock implementation instead of using these
4. This has happened across multiple implementation attempts

### Root Cause:
- Underestimating complexity of integrating existing GLM code
- Creating "simpler" mock implementations as temporary solution
- Not prioritizing real LLM integration as primary requirement

### Tasks Required:
1. **STOP creating new GLM implementations** - use existing ones
2. **Integrate with existing ZAI provider** directly
3. **Reuse UnifiedGLMAgent** without modification
4. **Focus on integration rather than new implementation**
5. **Make real LLM integration the highest priority**

### Success Criteria:
- Planner uses existing GLM-4.6-coding via ZAI provider
- No new GLM implementations are created
- Existing code is reused without duplication
- Integration is minimal and focused

## Issue #69: Fix Testing Database Issues

### Status: NOT STARTED

### Description:
Database locking errors prevent comprehensive testing of the system.

### Current State:
```
# Test errors
database is locked
```

### Tasks Required:
1. **Identify root cause**: Determine why database is being locked
2. **Fix test isolation**: Ensure tests don't interfere with each other
3. **Use in-memory database**: For tests that don't need persistence
4. **Update test fixtures**: Ensure clean state between tests

### Success Criteria:
- All tests run without database locking errors
- Tests are properly isolated
- Test suite provides comprehensive coverage