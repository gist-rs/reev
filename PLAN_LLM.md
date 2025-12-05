# PLAN_LLM.md - ZAI SDK Consolidation

## Objective
Create a unified `zai-sdk` crate to consolidate all GLM-4.6 client implementations currently scattered across the codebase.

## Status ✅ COMPLETED
The zai-sdk crate has been successfully implemented with:
- Builder pattern for flexible client configuration
- Support for both Standard and Coding GLM-4.6 variants
- Generic request/response handling with proper error types
- Streaming response support
- Comprehensive examples and documentation

## Current State Analysis
GLM client implementations are currently scattered in multiple locations:
- `crates/reev-agent/src/providers/zai/` (agent-specific implementation)
- `crates/reev-core/src/llm/glm_client.rs` (core implementation)
- Various examples and documentation referencing GLM-4.6

## Solution Architecture
Create `crates/zai-sdk` as a standalone, reusable SDK with:

### Core Structure
```
zai-sdk/
├── src/
│   ├── lib.rs          (main entry point)
│   ├── client.rs       (ZAI client with builder pattern)
│   ├── models.rs       (GLM variants and response types)
│   ├── traits.rs       (generic response/request handling)
│   ├── error.rs        (custom error types)
│   └── lib.rs         (main entry point and streaming type)
└── Cargo.toml
```

### Key Features
1. **Generic Builder Pattern**: `<T>` for response type, `<T,U>` for request/response
2. **Model Abstraction**: Support for all GLM-4.6 variants
3. **Reusability**: Project-agnostic design for use across different projects
4. **Type Safety**: Strong typing with proper error handling
5. **Streaming Support**: Built-in streaming response capabilities
6. **Token Limits**: Both Standard and Coding variants with 200K context/32K output tokens

## Implementation Status

### Phase 1: Core SDK Creation ✅ COMPLETED
1. ✅ Created `zai-sdk` crate structure
2. ✅ Implemented ZAI client with builder pattern
3. ✅ Added model variants and response type abstractions
4. ✅ Implemented generic traits for request/response handling
5. ✅ Added custom error types
6. ✅ Implemented streaming response support
7. ✅ Fixed all compilation issues and warnings
8. ✅ Added examples and documentation

### Phase 2: Integration ⏳ IN PROGRESS
1. ⏳ Update `reev-core` to use `zai-sdk`
2. ⏳ Update `reev-agent` to use `zai-sdk`
3. ⏳ Remove duplicate GLM client implementations
4. ⏳ Update examples to use consolidated SDK

### Phase 3: Documentation & Testing ⏳ PENDING
1. ⏳ Add comprehensive documentation
2. ⏳ Create integration tests
3. ⏳ Update project documentation

## Design Principles
- Follow modular architecture with files under 320-512 lines
- Use builder pattern with generic types for flexibility
- Keep SDK independent of reev-specific logic
- Maintain backward compatibility during transition
- Implement proper error handling and type safety
- Support both standard and coding variants with proper token limits