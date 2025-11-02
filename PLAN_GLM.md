# PLAN_GLM.md - ZAI Agent Modernization & GLM-4.6 Enhancement
## 🚨 **NO-FALLBACK APPROACH** - Clear errors, no silent fallbacks

## 🎯 Refined Requirements (No-Fallback Approach)

### Enhanced Context Integration
- **Multi-turn Conversation Support**: Enable step-by-step reasoning for complex DeFi operations
- **Provider-Specific Design**: Each agent handles single provider with clear error messages
- **Conditional Tool Filtering**: Flow mode vs normal mode with dynamic tool selection
- **Enhanced Context Building**: Account information, state awareness, and optimization
- **Comprehensive Response Formatting**: OpenTelemetry integration and execution result extraction

### GLM-4.6 Specific Requirements
- **Model Compatibility**: Ensure GLM-4.6 works with enhanced framework patterns
- **Model Validation**: Dynamic model parameter with availability verification
- **Tool Definition Formatting**: Provider-specific tool schema adaptation
- **Conversation Depth Optimization**: Context-aware turn management
- **Error Handling**: GLM-specific error patterns with fail-fast approach

## 📊 Current State Analysis (No-Fallback Approach)

### OpenAI Agent (Working Reference)
```rust
✅ Multi-turn: agent.multi_turn(conversation_depth)
✅ Tool Filtering: allowed_tools conditional logic
✅ Enhanced Context: AgentHelper.build_enhanced_context()
✅ Response Formatting: format_comprehensive_response()
✅ Clear Error Messages: Provider-specific validation
```

### ZAI Agent (Current Implementation)
```rust
❌ Direct Completion: model.completion() instead of multi-turn
❌ Manual Routing: Tool calls handled manually vs framework
❌ Legacy Response: Manual JSON formatting vs format_comprehensive_response()
```

### 🎯 **Progress Summary**:
- **Phase 1.1 (Agent Builder)**: ❌ 0% - Still using CompletionRequestBuilder
- **Phase 3.1 (Response Formatting)**: ❌ 0% - Manual JSON vs format_comprehensive_response()

**Remaining Work: 2 critical phases** (Model validation ✅ completed in Issue #8)

## 🚀 Implementation Plan

### Phase 1: Core Architecture Migration (Priority: Critical)

#### 1.1 Replace Direct Completion with Agent Builder 🔴 **CRITICAL**
**Target**: Convert from `CompletionRequestBuilder` to `client.agent_builder()`

**Current ZAI Pattern**:
```rust
let request = CompletionRequestBuilder::new(model.clone(), &enhanced_user_request)
    .tool(sol_tool_def)
    .build();
let result = model.completion(request).await?;
```

**Target OpenAI Pattern**:
```rust
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

**Tasks**:
- [ ] Replace `CompletionRequestBuilder` with `agent_builder()` pattern
- [ ] Implement multi-turn conversation support
- [ ] Remove manual tool routing logic
- [ ] Test GLM-4.6 compatibility with agent_builder



### Phase 2: Enhanced Features Integration (Priority: High)



### Phase 3: Response & Error Handling (Priority: Medium)

#### 3.1 Standardized Response Formatting ❌ **NOT STARTED**
**Status**: 🔴 **MISSING** - Still using manual JSON formatting

**Current Problem**:
```rust
// LEGACY - Manual JSON response formatting
let response_json = json!({
    "transactions": [tool_result],
    "summary": summary,
    "signatures": ["estimated_signature"]
}).to_string();
```

**Target Pattern**:
```rust
let execution_result = extract_execution_results(&response_str, "ZAIAgent").await?;
let tool_calls = AgentHelper::extract_tool_calls_from_otel();
AgentHelper::format_comprehensive_response(
    execution_result,
    Some(tool_calls),
    "ZAIAgent"
)
```

**Tasks**:
- [ ] Replace manual response formatting with `format_comprehensive_response()`
- [ ] Add execution result extraction
- [ ] Standardize error handling across agents

### Phase 4: Testing & Validation (Priority: High)

#### 4.1 GLM-4.6 Compatibility Testing
**Test Cases**:
- [ ] Basic agent_builder functionality
- [ ] Multi-turn conversation handling
- [ ] Tool execution and responses
- [ ] Performance benchmarking

#### 4.2 Integration Testing
**Test Scenarios**:
- [ ] Response formatting consistency
- [ ] Cross-agent compatibility
- [ ] Error handling scenarios

## 🎯 Success Criteria (Updated for No-Fallback Approach)

### Functional Requirements
- ❌ **PENDING** ZAI agent matches OpenAI agent capabilities
- ❌ **PENDING** GLM-4.6 full compatibility with enhanced framework
- ❌ **PENDING** Multi-turn conversation support

### Technical Requirements
- ❌ **PENDING** Consistent response formatting across agents
- ❌ **PENDING** Performance parity with OpenAI agent
- ❌ **PENDING** Comprehensive test coverage

### Integration Requirements
- ❌ **PENDING** Cross-agent compatibility
- ✅ **DONE** Model validation (Issue #8)

## 📋 Implementation Checklist (Updated Progress)

### Phase 1 Implementation 🔴 **IN PROGRESS**
- [ ] ❌ Replace direct completion with agent_builder (Critical)
- [ ] ❌ Add multi-turn conversation support
- [ ] ❌ Test basic functionality

### Phase 2 Implementation ❌ **NOT STARTED**
- [ ] ❌ Standardize response formatting
- [ ] ❌ Test error handling consistency

### Phase 3 Validation 🟡 **READY TO START**
- [ ] 🟡 Run comprehensive test suite
- [ ] 🟡 Performance benchmarking
- [ ] 🟡 Integration testing

## 🚨 Risk Assessment (Updated)

### High Risk Items
- **GLM-4.6 Compatibility**: ❌ Unknown if agent_builder pattern works with GLM
- **Performance Impact**: Multi-turn conversations may be slower

### Mitigation Strategies
- **Gradual Migration**: Implement changes incrementally with rollback capability
- **Comprehensive Testing**: Extensive test coverage before production
- **Performance Monitoring**: Benchmark against current implementation

### Dependencies
- **ZAI Provider Updates**: May need provider-side changes for full compatibility
- **Rig Framework**: Ensure framework supports required features for agent_builder

## 📅 Timeline Estimate (Updated Progress)

- **Phase 1**: 🔴 **2-3 days** (Critical: agent_builder + multi-turn)
- **Phase 2**: ❌ **1-2 days** (Response formatting)
- **Phase 3**: 🟡 **2-3 days** (Testing & validation)

**Total Estimated Time**: 5-8 days (model validation ✅ completed)
**Critical Path**: Phase 1.1 (agent_builder pattern) + multi-turn conversation

## 🔗 Related Files

### Reference Implementation
- `crates/reev-agent/src/enhanced/openai.rs` - Working reference
- `crates/reev-agent/src/enhanced/common/mod.rs` - Common helpers
- `crates/reev-agent/src/providers/zai.rs` - ZAI provider

### Test Files
- `tests/agent_integration_test.rs` - Integration tests
- `tests/glm_compatibility_test.rs` - GLM-specific tests

### Configuration
- `.env.example` - Environment variables
- `ARCHITECTURE.md` - Architecture guidelines
- `AGENTS.md` - Agent development rules