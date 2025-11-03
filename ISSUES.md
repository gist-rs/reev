# Issues

## 🎯 **Current Active Issues**

---

## Issue #2: Dynamic Flow Implementation - reev-orchestrator Crate

**Priority**: 🔴 **CRITICAL**
**Status**: 🔴 **OPEN**
**Assigned**: reev-orchestrator

**Problem**: Current system "cheats" by reading static YML files with hardcoded prompts, limiting flexibility and real-world usability.

**Phase 1 Tasks**:
- [ ] Create `reev-orchestrator` crate with basic structure
- [ ] Extract mock data from `protocols/jupiter/jup-sdk/tests/token_test.rs`
- [ ] Implement context resolver for wallet balance and prices
- [ ] Create YML generator for context-aware prompts
- [ ] Add CLI integration with `--dynamic` flag
- [ ] Implement temporary file generation in `/tmp/dynamic-{timestamp}.yml`

**Acceptance Criteria**:
- [ ] Dynamic flows work for basic patterns (swap, lend, swap+lend)
- [ ] Context resolution < 1s for typical wallets
- [ ] 99.9% backward compatibility maintained
- [ ] Generated prompts achieve same success rates as static

**Dependencies**: reev-types, reev-tools, reev-protocols
**Timeline**: Phase 1 (Week 1-2)
**Risk**: Medium - New architecture, minimal integration risk

---

## Issue #3: Dynamic Flow Runner Integration

**Priority**: 🟡 **HIGH**
**Status**: 🔴 **OPEN**
**Assigned**: reev-runner

**Problem**: Runner needs modification to support dynamic flow sources while maintaining static file compatibility.

**Phase 1 Tasks**:
- [ ] Modify `RunBenchmark` to accept `BenchmarkSource` enum
- [ ] Add support for temporary generated YML files
- [ ] Implement feature flag for `dynamic_flows = "bridge"`
- [ ] Add dynamic flow execution metrics
- [ ] Ensure backward compatibility with existing CLI

**Target Implementation**:
```rust
pub enum BenchmarkSource {
    StaticFile { path: String },
    DynamicFlow { prompt: String, wallet: String },
}

pub struct RunBenchmark {
    pub source: BenchmarkSource,
    // ... existing fields
}
```

**Acceptance Criteria**:
- [ ] Existing static YML execution unchanged
- [ ] Dynamic YML generation works seamlessly
- [ ] Performance impact < 100ms overhead
- [ ] All existing tests pass

**Dependencies**: Issue #2 (reev-orchestrator)
**Timeline**: Phase 1 (Week 1-2)
**Risk**: Low - Enhances existing functionality

---

## Issue #4: Agent Context Enhancement

**Priority**: 🟡 **HIGH**
**Status**: 🔴 **OPEN**
**Assigned**: reev-agent

**Problem**: Agents need to receive and utilize dynamic context (wallet balance, prices) for context-aware prompt generation.

**Phase 1 Tasks**:
- [ ] Enhance agent interface to accept `PromptContext`
- [ ] Modify `UnifiedGLMAgent` to process dynamic context
- [ ] Add context injection into prompt generation
- [ ] Implement context-aware tool selection
- [ ] Add OpenTelemetry spans for context resolution

**Target Implementation**:
```rust
fn execute_agent(
    benchmark_content: String,
    dynamic_context: PromptContext,
    generated_prompt: Option<String>
) -> Result
```

**Acceptance Criteria**:
- [ ] Agents can process wallet context
- [ ] Dynamic prompts generate same success rates as static
- [ ] Context resolution properly traced
- [ ] No regression in existing agent functionality

**Dependencies**: Issue #2 (reev-orchestrator), Issue #3 (reev-runner)
**Timeline**: Phase 1 (Week 1-2)
**Risk**: Low - Enhancement, not breaking change

---

## Issue #5: Mock Data System for Testing

**Priority**: 🟡 **HIGH**
**Status**: 🔴 **OPEN**
**Assigned**: reev-orchestrator

**Problem**: Need comprehensive mock data system for testing dynamic flows without external dependencies.

**Phase 1 Tasks**:
- [ ] Extract token/price data from Jupiter SDK tests
- [ ] Create `tests/mock_data.rs` with static mock responses
- [ ] Implement mock wallet context generator
- [ ] Add mock transaction responses
- [ ] Create integration test suite with 100% coverage

**Mock Data Structure**:
```rust
pub struct MockWalletContext {
    pub owner: String,
    pub sol_balance: u64,
    pub token_balances: HashMap<String, TokenBalance>,
    pub token_prices: HashMap<String, f64>,
    pub total_value_usd: f64,
}
```

**Acceptance Criteria**:
- [ ] Mock data covers all common DeFi scenarios
- [ ] Tests run without external dependencies
- [ ] Mock data stays in sync with Jupiter SDK
- [ ] 100% test coverage for dynamic flows

**Dependencies**: None (can start immediately)
**Timeline**: Phase 1 (Week 1)
**Risk**: Low - Testing infrastructure

---

## Issue #6: Template System Implementation

**Priority**: 🟢 **MEDIUM**
**Status**: 🔴 **OPEN**
**Assigned**: reev-orchestrator

**Problem**: Need template system for generating context-aware prompts for common DeFi patterns.

**Phase 1 Tasks**:
- [ ] Design template hierarchy (base/protocols/scenarios)
- [ ] Implement Handlebars-based template engine
- [ ] Create templates for swap, lend, swap+lend patterns
- [ ] Add template validation and inheritance
- [ ] Implement template caching for performance

**Template Structure**:
```
templates/
├── base/
│   ├── swap.hbs
│   └── lend.hbs
├── protocols/
│   └── jupiter/
└── scenarios/
    └── swap_then_lend.hbs
```

**Acceptance Criteria**:
- [ ] Templates support 90% of common patterns
- [ ] Template compilation < 10ms
- [ ] Template inheritance works correctly
- [ ] Templates generate context-aware prompts

**Dependencies**: Issue #2 (reev-orchestrator)
**Timeline**: Phase 1 (Week 2)
**Risk**: Low - Template system, isolated component

---

## Issue #1: ZAI Agent Agent Builder Pattern Migration

**Priority**: 🟡 HIGH
**Status**: 🔴 **OPEN**
**Assigned**: reev-agent

**Problem**: ZAI Agent still uses legacy `CompletionRequestBuilder` instead of modern agent builder pattern

**Current Implementation**:
```rust
// LEGACY - Single completion without multi-turn
let mut request_builder = CompletionRequestBuilder::new(model.clone(), &unified_data.enhanced_user_request);
request_builder = request_builder.tool(unified_data.tools.sol_tool.definition(String::new()).await;
let request = request_builder.build();
let result = model.completion(request).await?;
```

**Target Implementation**:
```rust
// MODERN - Agent builder with multi-turn support (from OpenAI agent)
let agent = client
    .completion_model(&model_name)
    .into_agent_builder()
    .preamble(&enhanced_prompt)
    .tool(tools.sol_tool)
    .tool(tools.spl_tool)
    .build();

let response = agent
    .prompt(&enhanced_user_request)
    .multi_turn(conversation_depth)
    .await?;
```

**Current Status**: 🟢 **Production Ready** - Current implementation works correctly
**Reason for Upgrade**: Feature parity with OpenAI agent (multi-turn conversations)

**Tasks**:
- [ ] Replace `CompletionRequestBuilder` with `agent_builder()` pattern
- [ ] Implement multi-turn conversation support
- [ ] Test GLM-4.6 compatibility with agent_builder
- [ ] Enable step-by-step reasoning for complex DeFi operations

**Acceptance Criteria**:
- [ ] ZAI Agent uses agent_builder pattern
- [ ] Multi-turn conversations enabled
- [ ] GLM-4.6 compatibility verified
- [ ] Performance parity with OpenAI agent

**Risk**: Low - Current working implementation serves as fallback

---

## Issue #2: ✅ RESOLVED - ZAI Agent Response Formatting

**Priority**: ✅ COMPLETED
**Status**: 🟢 **DONE**
**Assigned**: reev-agent

**Problem**: ❌ RESOLVED - ZAI Agent now uses standardized response formatting

**Current Implementation**:
```rust
// ✅ MODERN - Using unified response formatting
let tool_calls = AgentHelper::extract_tool_calls_from_otel();
UnifiedGLMAgent::format_response(&response_str, "ZAIAgent", Some(tool_calls)).await
```

**Resolution Details**:
- ✅ Replaced manual JSON formatting with `UnifiedGLMAgent::format_response()`
- ✅ Added execution result extraction via shared function
- ✅ Integrated OpenTelemetry tool call extraction
- ✅ Standardized error handling across agents
- ✅ Ensured consistency with OpenAI agent responses

**Acceptance Criteria**:
- [✅] Response formatting standardized across all agents
- [✅] OpenTelemetry integration for tool call extraction
- [✅] Consistent error handling
- [✅] Cross-agent response compatibility

**Resolution Date**: November 2024

---

## 🎯 **GLM Authentication & Routing - RESOLVED** ✅

**Issue**: GLM-4.6-coding authentication failure
**Status**: 🟢 **RESOLVED** (November 2024)

**Resolution**:
- ✅ Both `glm-4.6-coding` and `glm-4.6` use `ZAI_API_KEY`
- ✅ Both agents route through reev-agent with different endpoints
- ✅ Model name properly stripped to `glm-4.6` for ZAI validation
- ✅ No fallbacks - clear errors when `ZAI_API_KEY` missing
- ✅ Only deterministic fallback when no specific agent configured

**Test Results**:
- ✅ `glm-4.6-coding`: Score 100.0% - Working
- ✅ `glm-4.6`: Score 100.0% - Working  
- ✅ `deterministic`: Score 100.0% - Working
- ✅ `local`: Score 100.0% - Working

---

## 📊 **Implementation Progress** (Updated December 2024)

### 🔴 **Dynamic Flow Implementation (Phase 1)**:
- **Issue #2**: reev-orchestrator crate creation - 🔴 **NOT STARTED**
- **Issue #3**: Runner integration - 🔴 **NOT STARTED** 
- **Issue #4**: Agent context enhancement - 🔴 **NOT STARTED**
- **Issue #5**: Mock data system - 🔴 **NOT STARTED**
- **Issue #6**: Template system - 🔴 **NOT STARTED**

### ✅ **Completed Work**:

### ✅ **Completed Work**:
- **GLM Authentication & Routing**: ✅ Complete - Both GLM agents working
- **Enhanced Context Integration**: ✅ Complete via UnifiedGLMAgent
- **Conditional Tool Filtering**: ✅ Complete via UnifiedGLMAgent  
- **Model Validation**: ✅ Complete (Issue #8 from previous version)
- **Standardized Response Formatting**: ✅ Complete via UnifiedGLMAgent::format_response()
- **No-Fallback Provider Design**: ✅ Complete
- **Comprehensive OTEL Implementation**: ✅ Complete (100% coverage)
- **Agent Tool Coverage**: ✅ Complete (13/13 tools enhanced)

### 🔴 **Remaining Work**:
1. **Issue #1**: Agent Builder Pattern Migration (Optional - for feature parity)
2. **Issue #2**: Dynamic Flow Implementation - Phase 1 (Critical)
3. **Issue #3**: Runner Integration (High)
4. **Issue #4**: Agent Context Enhancement (High)
5. **Issue #5**: Mock Data System (High)
6. **Issue #6**: Template System Implementation (Medium)

**Total Remaining Work**: 6 issues (1 enhancement + 5 dynamic flow)
**Current Status**: 🟡 **In Progress** - Dynamic flow implementation started

---

## 🎯 **Dynamic Flow Success Criteria (Phase 1)**

### Technical Requirements
- [ ] Dynamic flows work for swap, lend, swap+lend patterns
- [ ] Context resolution < 500ms with caching
- [ ] 99.9% backward compatibility maintained
- [ ] < 100ms flow execution overhead vs static
- [ ] Template system supports 90% of common patterns

### User Experience
- [ ] Natural language prompts work for basic DeFi operations
- [ ] Context-aware prompts adapt to user wallet state
- [ ] Clear error messages with recovery suggestions
- [ ] CLI `--dynamic` flag works seamlessly

### Developer Experience
- [ ] Comprehensive mock-based testing
- [ ] Clear separation between static and dynamic flows
- [ ] Template inheritance and validation
- [ ] Performance parity with existing system

---

## 🎯 **GLM Success Criteria**

### GLM-4.6 Full Compatibility
- [✅] Consistent response formatting across agents
- [ ] Multi-turn conversation support enabled (enhancement)
- [ ] Step-by-step reasoning for complex operations (enhancement)
- [ ] Agent builder pattern working with ZAI provider (enhancement)

### Technical Requirements  
- [✅] Zero regression in existing functionality
- [✅] Performance parity with OpenAI agent
- [✅] Comprehensive test coverage for current features
- [✅] Cross-agent compatibility maintained
- [ ] Enhanced test coverage for multi-turn features (enhancement)

### Integration Requirements
- [✅] Seamless FlowAgent integration
- [✅] OpenTelemetry compatibility maintained
- [✅] Clear error messages for users
- [✅] Documentation updates completed

---