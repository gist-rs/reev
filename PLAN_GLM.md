# PLAN_GLM.md - ZAI Agent Modernization & GLM-4.6 Enhancement

## 🎯 Refined Requirements

### Enhanced Context Integration
- **Multi-turn Conversation Support**: Enable step-by-step reasoning for complex DeFi operations
- **API Priority System**: Local → GLM → OpenAI → Fallback with smart detection
- **Conditional Tool Filtering**: Flow mode vs normal mode with dynamic tool selection
- **Enhanced Context Building**: Account information, state awareness, and optimization
- **Comprehensive Response Formatting**: OpenTelemetry integration and execution result extraction

### GLM-4.6 Specific Requirements
- **Model Compatibility**: Ensure GLM-4.6 works with enhanced framework patterns
- **Tool Definition Formatting**: Provider-specific tool schema adaptation
- **Conversation Depth Optimization**: Context-aware turn management
- **Error Handling**: GLM-specific error patterns and recovery

## 📊 Current State Analysis

### OpenAI Agent (Working Reference)
```rust
✅ Multi-turn: agent.multi_turn(conversation_depth)
✅ API Priority: local → GLM → OpenAI → fallback
✅ Tool Filtering: allowed_tools conditional logic
✅ Enhanced Context: AgentHelper.build_enhanced_context()
✅ Response Formatting: format_comprehensive_response()
```

### ZAI Agent (Legacy Implementation)
```rust
❌ Direct Completion: model.completion() instead of multi-turn
❌ Manual Routing: Tool calls handled manually vs framework
❌ Missing API Priority: No fallback system
❌ No Tool Filtering: Missing allowed_tools logic
❌ Legacy Context: No enhanced integration
```

## 🚀 Implementation Plan

### Phase 1: Core Architecture Migration (Priority: Critical)

#### 1.1 Replace Direct Completion with Agent Builder
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
    .completion_model(&actual_model_name)
    .completions_api()
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

#### 1.2 API Priority System Implementation
**Target**: Add comprehensive API key and endpoint priority logic

**Implementation**:
```rust
let (client, actual_model_name) = match model_name {
    "local" => {
        // Local model configuration
        let local_url = std::env::var("LOCAL_MODEL_URL")
            .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
        let dummy_key = "dummy-key-for-local-model";
        let actual_model = std::env::var("LOCAL_MODEL_NAME")
            .unwrap_or_else(|_| "qwen3-coder-30b-a3b-instruct-mlx".to_string());
        
        (zai::Client::builder(dummy_key).base_url(&local_url).build(), actual_model)
    }
    model if model.starts_with("glm-") => {
        // GLM models with ZAI endpoint
        let zai_api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY required for GLM models"))?;
        let zai_api_url = std::env::var("ZAI_API_URL")
            .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());
        
        (zai::Client::builder(&zai_api_key).base_url(&zai_api_url).build(), model_name.to_string())
    }
    _ => {
        // Fallback logic
        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            // Use OpenAI client for non-GLM models
            (create_openai_client(&openai_key)?, model_name.to_string())
        } else {
            // Final fallback to local
            create_local_client()?
        }
    }
};
```

**Tasks**:
- [ ] Implement local → GLM → OpenAI → fallback detection
- [ ] Add environment variable validation
- [ ] Support dynamic endpoint configuration
- [ ] Add comprehensive error handling

### Phase 2: Enhanced Features Integration (Priority: High)

#### 2.1 Conditional Tool Filtering
**Target**: Implement flow mode vs normal mode tool selection

**OpenAI Reference Pattern**:
```rust
let agent = if let Some(allowed_tools) = allowed_tools {
    // Flow mode: only allowed tools
    info!("[OpenAIAgent] Flow mode: Only allowing {} tools: {:?}", allowed_tools.len(), allowed_tools);
    let mut builder = client.agent_builder()
        .preamble(&enhanced_prompt)
        .tool(tools.sol_tool)
        .tool(tools.spl_tool);
    
    if allowed_tools.contains(&"get_lend_earn_tokens".to_string()) {
        builder = builder.tool(tools.lend_earn_tokens_tool);
    }
    if allowed_tools.contains(&"jupiter_earn".to_string()) {
        builder = builder.tool(tools.jupiter_earn_tool);
    }
    
    builder.build()
} else {
    // Normal mode: all discovery tools
    client.agent_builder()
        .preamble(&enhanced_prompt)
        .tool(tools.sol_tool)
        .tool(tools.spl_tool)
        .tool(tools.jupiter_earn_tool)
        .tool(tools.lend_earn_tokens_tool)
        .build()
};
```

**Tasks**:
- [ ] Add allowed_tools checking logic
- [ ] Implement conditional tool builder pattern
- [ ] Add proper logging for tool filtering
- [ ] Match OpenAI's tool selection logic

#### 2.2 Enhanced Context Integration
**Target**: Replace manual context with `AgentHelper.build_enhanced_context()`

**Current ZAI Pattern**:
```rust
// Missing enhanced context integration
let enhanced_prompt = manual_prompt_build(&payload);
```

**Target Pattern**:
```rust
let (context_integration, enhanced_prompt_data, enhanced_prompt) =
    AgentHelper::build_enhanced_context(&payload, &key_map)?;

let conversation_depth = AgentHelper::determine_conversation_depth(
    &context_integration,
    &enhanced_prompt_data,
    payload.initial_state.as_deref().unwrap_or(&[]),
    &key_map,
    &payload.id,
);
```

**Tasks**:
- [ ] Replace manual prompt building with AgentHelper
- [ ] Add conversation depth optimization
- [ ] Implement context integration logic
- [ ] Add proper logging and debugging

### Phase 3: Response & Error Handling (Priority: Medium)

#### 3.1 Standardized Response Formatting
**Target**: Use `AgentHelper.format_comprehensive_response()`

**Current ZAI Pattern**:
```rust
// Manual JSON response formatting
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
- [ ] Replace manual response formatting
- [ ] Add OpenTelemetry integration
- [ ] Implement execution result extraction
- [ ] Standardize error handling

#### 3.2 Enhanced Logging & Debugging
**Target**: Align logging patterns with OpenAI agent

**Tasks**:
- [ ] Add comprehensive execution logging
- [ ] Implement debug information tracking
- [ ] Add performance metrics
- [ ] Standardize error messages

### Phase 4: Testing & Validation (Priority: High)

#### 4.1 GLM-4.6 Compatibility Testing
**Test Cases**:
- [ ] Basic agent_builder functionality
- [ ] Multi-turn conversation handling
- [ ] Tool execution and responses
- [ ] Error handling and recovery
- [ ] Performance benchmarking

#### 4.2 Integration Testing
**Test Scenarios**:
- [ ] API priority fallback system
- [ ] Conditional tool filtering
- [ ] Enhanced context building
- [ ] Response formatting consistency
- [ ] Cross-agent compatibility

## 🎯 Success Criteria

### Functional Requirements
- ✅ ZAI agent matches OpenAI agent capabilities
- ✅ GLM-4.6 full compatibility with enhanced framework
- ✅ Multi-turn conversation support
- ✅ API priority and fallback system
- ✅ Conditional tool filtering
- ✅ Enhanced context integration

### Technical Requirements
- ✅ Zero regression in existing functionality
- ✅ Consistent response formatting across agents
- ✅ Proper error handling and logging
- ✅ Performance parity with OpenAI agent
- ✅ Comprehensive test coverage

### Integration Requirements
- ✅ Seamless FlowAgent integration
- ✅ OpenTelemetry compatibility
- ✅ Database logging consistency
- ✅ Environment variable handling
- ✅ Configuration management

## 📋 Implementation Checklist

### Pre-Implementation
- [ ] Review OpenAI agent enhancements since ZAI was last updated
- [ ] Test current ZAI agent functionality baseline
- [ ] Identify GLM-4.6 specific requirements
- [ ] Prepare test environment and datasets

### Phase 1 Implementation
- [ ] Replace direct completion with agent_builder
- [ ] Implement API priority system
- [ ] Add multi-turn conversation support
- [ ] Test basic functionality

### Phase 2 Implementation
- [ ] Add conditional tool filtering
- [ ] Implement enhanced context integration
- [ ] Add conversation depth optimization
- [ ] Test with complex scenarios

### Phase 3 Implementation
- [ ] Standardize response formatting
- [ ] Add OpenTelemetry integration
- [ ] Implement enhanced logging
- [ ] Test error handling

### Phase 4 Validation
- [ ] Run comprehensive test suite
- [ ] Performance benchmarking
- [ ] Integration testing
- [ ] Documentation updates

## 🚨 Risk Assessment

### High Risk Items
- **GLM-4.6 Compatibility**: Unknown if agent_builder pattern works with GLM
- **API Changes**: ZAI provider might have different requirements
- **Performance Impact**: Multi-turn conversations may be slower

### Mitigation Strategies
- **Gradual Migration**: Implement changes incrementally with rollback capability
- **Comprehensive Testing**: Extensive test coverage before production
- **Performance Monitoring**: Benchmark against current implementation

### Dependencies
- **ZAI Provider Updates**: May need provider-side changes for full compatibility
- **Rig Framework**: Ensure framework supports required features
- **Environment Configuration**: Proper setup for all API endpoints

## 📅 Timeline Estimate

- **Phase 1**: 2-3 days (Core architecture)
- **Phase 2**: 2-3 days (Enhanced features)
- **Phase 3**: 1-2 days (Response handling)
- **Phase 4**: 2-3 days (Testing & validation)

**Total Estimated Time**: 7-11 days

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