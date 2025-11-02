# Issues

## 🎯 **Current Active Issues**

---

## Issue #1: ZAI Agent Agent Builder Pattern Migration

**Priority**: 🔴 CRITICAL
**Status**: 🔴 **OPEN**
**Assigned**: reev-agent

**Problem**: ZAI Agent still uses legacy `CompletionRequestBuilder` instead of modern agent builder pattern

**Current Implementation**:
```rust
// LEGACY - Single completion without multi-turn
let request = CompletionRequestBuilder::new(model.clone(), &enhanced_user_request)
    .tool(sol_tool_def)
    .build();
let result = model.completion(request).await?;
```

**Target Implementation**:
```rust
// MODERN - Agent builder with multi-turn support
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
- [ ] Enable step-by-step reasoning for complex DeFi operations

**Acceptance Criteria**:
- [ ] ZAI Agent uses agent_builder pattern
- [ ] Multi-turn conversations enabled
- [ ] Manual tool routing removed
- [ ] GLM-4.6 compatibility verified
- [ ] Performance parity with OpenAI agent

---

## Issue #2: ZAI Agent Standardized Response Formatting

**Priority**: 🟡 HIGH
**Status**: 🔴 **OPEN**
**Assigned**: reev-agent

**Problem**: ZAI Agent uses manual JSON formatting instead of standardized response formatting

**Current Implementation**:
```rust
// LEGACY - Manual JSON formatting
let response_json = json!({
    "transactions": [tool_result],
    "summary": summary,
    "signatures": ["estimated_signature"]
}).to_string();
```

**Target Implementation**:
```rust
// STANDARDIZED - Using AgentHelper
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
- [ ] Integrate OpenTelemetry tool call extraction
- [ ] Standardize error handling across agents
- [ ] Ensure consistency with OpenAI agent responses

**Acceptance Criteria**:
- [ ] Response formatting standardized across all agents
- [ ] OpenTelemetry integration for tool call extraction
- [ ] Consistent error handling
- [ ] Cross-agent response compatibility

---

## 📊 **Implementation Progress**

### ✅ **Completed Work**:
- **Enhanced Context Integration**: ✅ Complete via UnifiedGLMAgent
- **Conditional Tool Filtering**: ✅ Complete via UnifiedGLMAgent  
- **Model Validation**: ✅ Complete (Issue #8 from previous version)
- **No-Fallback Provider Design**: ✅ Complete
- **Comprehensive OTEL Implementation**: ✅ Complete (100% coverage)
- **Agent Tool Coverage**: ✅ Complete (13/13 tools enhanced)

### 🔴 **Remaining Critical Path**:
1. **Issue #1**: Agent Builder Pattern Migration (Phase 1.1)
2. **Issue #2**: Standardized Response Formatting (Phase 3.1)

**Total Remaining Work**: 2 critical issues
**Estimated Timeline**: 4-6 days

---

## 🎯 **Success Criteria**

### GLM-4.6 Full Compatibility
- [ ] Multi-turn conversation support enabled
- [ ] Step-by-step reasoning for complex operations
- [ ] Agent builder pattern working with ZAI provider
- [ ] Consistent response formatting across agents

### Technical Requirements  
- [ ] Zero regression in existing functionality
- [ ] Performance parity with OpenAI agent
- [ ] Comprehensive test coverage for new features
- [ ] Cross-agent compatibility maintained

### Integration Requirements
- [ ] Seamless FlowAgent integration
- [ ] OpenTelemetry compatibility maintained
- [ ] Clear error messages for users
- [ ] Documentation updates completed

---