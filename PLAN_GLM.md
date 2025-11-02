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
✅ Enhanced Context: Using UnifiedGLMAgent.shared logic
✅ Tool Filtering: allowed_tools conditional logic
✅ Flow Mode: Specialized tool selection working
❌ Direct Completion: model.completion() instead of multi-turn
❌ Manual Routing: Tool calls handled manually vs framework
❌ Hardcoded Model: GLM_4_6 without availability validation
❌ Legacy Response: Manual JSON formatting vs format_comprehensive_response()
```

### 🎯 **Progress Summary**:
- **Phase 1.1 (Agent Builder)**: ❌ 0% - Still using CompletionRequestBuilder
- **Phase 1.2 (No-Fallback)**: ✅ 100% - Provider-specific design implemented
- **Phase 2.1 (Tool Filtering)**: ✅ 100% - Complete via UnifiedGLMAgent
- **Phase 2.2 (Enhanced Context)**: ✅ 100% - Complete via UnifiedGLMAgent
- **Phase 3.1 (Response Formatting)**: ❌ 0% - Manual JSON vs format_comprehensive_response()
- **Model Validation**: 🔴 **NEW ISSUE** - Hardcoded GLM_4_6 (See Issue #8)

**Overall Progress: ~50%**

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

#### 1.2 Model Validation Enhancement 🔴 **HIGH** (Issue #8)
**Target**: Replace hardcoded model with dynamic validation

**Current Problem**:
```rust
// ISSUE #8 - Hardcoded without validation
let model = client.completion_model(zai::GLM_4_6);  // GLM_4_6 is a constant
```

**Required Solution**:
```rust
// NEEDED - Model availability validation
let model_name = "glm-4.6";  // Should be parameter, not hardcoded
let model = client.completion_model(model_name);

// Verify the model is actually available
client.verify().await
    .map_err(|e| anyhow!("ZAI model '{}' validation failed: {}", model_name, e))?;

info!("ZAI model '{}' validated and ready", model_name);
```

**Tasks**:
- [x] ⚠️ **ISSUE IDENTIFIED** - Added to ISSUES.md as #8
- [ ] Replace hardcoded `GLM_4_6` with dynamic model parameter
- [ ] Add `client.verify()` call for model availability
- [ ] Implement proper error handling for model validation failures

#### 1.3 No-Fallback Provider Design ✅ **COMPLETED**
**Status**: ✅ **DONE** - Provider-specific design implemented correctly

**Implementation**: Each agent handles single provider with clear error messages
- ZAIAgent: Only handles ZAI endpoints
- OpenAIAgent: Handles OpenAI with GLM routing
- Local models: Handled explicitly when requested

**Benefits**:
- ✅ Clear error messages instead of silent fallbacks
- ✅ Predictable behavior for users
- ✅ Simplified debugging and testing
- ✅ Provider-specific optimizations

### Phase 2: Enhanced Features Integration (Priority: High)

#### 2.1 Conditional Tool Filtering ✅ **COMPLETED**
**Status**: ✅ **DONE** - Implemented via UnifiedGLMAgent

**Current Implementation**:
```rust
// WORKING - Flow mode vs normal mode
let is_tool_allowed = |tool_name: &str| -> bool {
    match &flow_mode_indicator {
        Some(tools) => tools.contains(&tool_name.to_string()),
        None => {
            // SECURITY: Restrict jupiter_earn tool in normal mode
            tool_name != "jupiter_earn"
        }
    }
};
```

**Features**:
- ✅ Flow mode: Only allowed tools
- ✅ Normal mode: All discovery tools
- ✅ Security restrictions for sensitive tools
- ✅ Proper logging for tool filtering

#### 2.2 Enhanced Context Integration ✅ **COMPLETED**
**Status**: ✅ **DONE** - Implemented via UnifiedGLMAgent

**Current Implementation**:
```rust
// WORKING - Using shared logic
let unified_data = UnifiedGLMAgent::run(model_name, payload, key_map).await?;

// Includes:
// - AgentHelper::build_enhanced_context()
// - AgentHelper::determine_conversation_depth()
// - AgentTools::new_with_flow_mode()
```

**Features**:
- ✅ Enhanced context building with account information
- ✅ Conversation depth optimization
- ✅ Context integration logic
- ✅ Proper logging and debugging
- ✅ Shared logic across all GLM agents

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
- [ ] Add OpenTelemetry integration
- [ ] Implement execution result extraction
- [ ] Standardize error handling across agents

#### 3.2 Enhanced Logging & Debugging ⚠️ **PARTIAL**
**Status**: 🟡 **SOME LOGGING** - Basic logging exists, needs enhancement

**Current Implementation**:
```rust
// BASIC logging present
info!("[ZAIAgent] Tool called: {}", tool_call.function.name);
info!("[ZAIAgent] Arguments: {}", tool_call.function.arguments);
info!("[ZAIAgent] Tool result: {}", tool_result);
```

**Missing Enhancements**:
- [ ] Comprehensive execution logging
- [ ] Debug information tracking
- [ ] Performance metrics
- [ ] Standardized error messages
- [ ] OpenTelemetry span consistency

### Phase 4: Testing & Validation (Priority: High)

#### 4.1 GLM-4.6 Compatibility Testing
**Test Cases**:
- [ ] Basic agent_builder functionality (after Phase 1.1)
- [ ] Multi-turn conversation handling (after Phase 1.1)
- [ ] Model validation with invalid models (after Issue #8)
- [ ] Tool execution and responses (✅ working)
- [ ] Error handling and recovery (no-fallback approach)
- [ ] Performance benchmarking

#### 4.2 Integration Testing
**Test Scenarios**:
- [x] ✅ Conditional tool filtering (working)
- [x] ✅ Enhanced context building (working)
- [ ] Response formatting consistency (after Phase 3.1)
- [ ] Cross-agent compatibility (after Phase 1.1)
- [ ] Model validation failures (after Issue #8)
- [ ] No-fallback error scenarios

## 🎯 Success Criteria (Updated for No-Fallback Approach)

### Functional Requirements
- ❌ **PENDING** ZAI agent matches OpenAI agent capabilities
- ❌ **PENDING** GLM-4.6 full compatibility with enhanced framework
- ❌ **PENDING** Multi-turn conversation support
- ✅ **DONE** No-fallback provider-specific design
- ✅ **DONE** Conditional tool filtering
- ✅ **DONE** Enhanced context integration

### Technical Requirements
- ✅ **DONE** Zero regression in existing functionality
- ❌ **PENDING** Consistent response formatting across agents
- 🟡 **PARTIAL** Proper error handling and logging
- 🟡 **PARTIAL** Performance parity with OpenAI agent
- 🟡 **PARTIAL** Comprehensive test coverage

### Integration Requirements
- ✅ **DONE** Seamless FlowAgent integration
- ✅ **DONE** OpenTelemetry compatibility
- ✅ **DONE** Database logging consistency
- ❌ **PENDING** Model validation (Issue #8)
- ✅ **DONE** Clear environment variable error messages

## 📋 Implementation Checklist (Updated Progress)

### Pre-Implementation ✅ **COMPLETED**
- [x] ✅ Review OpenAI agent enhancements since ZAI was last updated
- [x] ✅ Test current ZAI agent functionality baseline
- [x] ✅ Identify GLM-4.6 specific requirements
- [x] ✅ Prepare test environment and datasets
- [x] ✅ Adopt no-fallback approach

### Phase 1 Implementation 🔴 **IN PROGRESS**
- [ ] ❌ Replace direct completion with agent_builder (Critical)
- [x] ✅ No-fallback provider design (Completed)
- [ ] 🔴 Model validation enhancement (Issue #8)
- [ ] ❌ Add multi-turn conversation support
- [ ] ❌ Test basic functionality

### Phase 2 Implementation ✅ **COMPLETED**
- [x] ✅ Conditional tool filtering (Working)
- [x] ✅ Enhanced context integration (Working via UnifiedGLMAgent)
- [x] ✅ Conversation depth optimization (Working)
- [x] ✅ Test with complex scenarios (Working)

### Phase 3 Implementation 🟡 **PARTIAL**
- [ ] ❌ Standardize response formatting
- [x] ✅ OpenTelemetry integration (Basic working)
- [ ] 🟡 Enhanced logging (Partial)
- [ ] ❌ Test error handling consistency

### Phase 4 Validation 🟡 **READY TO START**
- [ ] 🟡 Run comprehensive test suite
- [ ] 🟡 Performance benchmarking
- [ ] 🟡 Integration testing
- [ ] ❌ Documentation updates

## 🚨 Risk Assessment (Updated)

### High Risk Items
- **GLM-4.6 Compatibility**: ❌ Unknown if agent_builder pattern works with GLM
- **Model Validation**: 🔴 Hardcoded model limits flexibility (Issue #8)
- **Performance Impact**: Multi-turn conversations may be slower

### Medium Risk Items
- **Response Formatting**: Manual vs standardized format inconsistency
- **No-Fallback Errors**: Users need to handle explicit error messages
- **Testing Coverage**: Comprehensive validation needed after changes

### Mitigation Strategies
- **Gradual Migration**: Implement changes incrementally with rollback capability
- **Comprehensive Testing**: Extensive test coverage before production
- **Performance Monitoring**: Benchmark against current implementation
- **Clear Documentation**: Explicit error handling guidance for users

### Dependencies
- **ZAI Provider Updates**: May need provider-side changes for full compatibility
- **Rig Framework**: Ensure framework supports required features for agent_builder
- **Environment Configuration**: Proper API key setup with clear error messages

## 📅 Timeline Estimate (Updated Progress)

- **Phase 1**: 🔴 **3-4 days** (Critical: agent_builder + model validation)
- **Phase 2**: ✅ **COMPLETED** (Enhanced features - already working)
- **Phase 3**: 🟡 **1-2 days** (Response formatting + enhanced logging)
- **Phase 4**: 🟡 **2-3 days** (Testing & validation)

**Total Estimated Time**: 6-9 days (with 50% already completed)
**Critical Path**: Phase 1.1 (agent_builder pattern) + Issue #8 (model validation)

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