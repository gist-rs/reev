# Reev Core Tasks

## Current Priorities

### Priority 1: Enhance zai-sdk Streaming Implementation (Optional)
**Why**: Current implementation is simplified (returns non-streaming as single chunk)
**Where**: `crates/zai-sdk/src/client.rs` in `ZaiStreamHandler` implementation

**Tasks**:
1. Implement true streaming response handling
2. Parse Server-Sent Events (SSE) correctly
3. Handle backpressure and connection errors

**Impact**:
- Not blocking current functionality
- Could be enhanced in future if real-time streaming is needed
- Low priority as current implementation meets requirements

### Priority 2: Documentation and Test Coverage Enhancement (Optional)
**Why**: While basic documentation exists, it could be expanded with more examples
**Where**: Across all modules in zai-sdk and reev-core

**Tasks**:
1. Expand zai-sdk documentation with more examples
2. Add integration tests for edge cases
3. Improve inline documentation for complex flows

**Impact**:
- Not blocking current development
- Could be enhanced incrementally as needed
- Low priority for now

## Completed Work

### Structured LLM Response System
All phases of the structured LLM response system have been completed:

1. **Phase 1: Structured Data Types** - COMPLETED
   - Created `StructuredRefinedPrompt` struct with all required fields
   - Implemented `PromptAction` enum with all required variants
   - Implemented `PromptParameters` struct with required fields
   - Added `usable_amount` field for handling "all" keyword

2. **Phase 2: Prompt Processing** - COMPLETED
   - Implemented structured system prompt
   - Implemented `process_prompt_structured` method
   - Implemented "all" keyword handling with MaxAmountCalculator
   - Implemented validation logic in `validation.rs`

3. **Phase 3: Execution Flow** - COMPLETED
   - Updated `execute_step_with_rig_and_history` to use structured fields
   - Implemented `create_tool_calls_from_structured_data` method
   - Removed fallback to rule-based extraction for structured data

4. **Phase 4: YML Generation** - COMPLETED
   - Implemented `generate_flow_from_structured_prompt`
   - Added support for all action types (Transfer, Swap, Lend, Earn, Borrow)
   - Added `structured_prompt` field to `YmlStep`

5. **Phase 5: Testing and Validation** - COMPLETED
   - Created comprehensive test file `structured_llm_test.rs` with 12 tests
   - Fixed planner_test blocking issue by adding `#[tokio::test(flavor = "multi_thread")]` attribute
   - Added tests for validation, parsing, and builder patterns

### Gas Reserve Standardization
- Created centralized gas reserve module: `crates/reev-core/src/gas_reserve/mod.rs`
- Implemented standardized constants for different action types
- Updated all components to use centralized values:
  - MaxAmountCalculator
  - sol_transfer tools
  - planner
  - flow_builders
  - query_handler
- Added comprehensive test suite with 9 tests covering all scenarios
- Constants implemented:
  - TRANSFER_GAS_RESERVE (0.001 SOL)
  - SWAP_GAS_RESERVE (0.005 SOL)
  - DEFAULT_GAS_RESERVE (0.002 SOL)

### ZAI SDK Consolidation
- Created `crates/zai-sdk` as standalone, reusable SDK
- Implemented builder pattern with generic types
- Added support for both Standard and Coding GLM-4.6 variants
- Implemented proper error handling and type safety
- Added streaming response support (simplified implementation)
- Updated `reev-core` to use `zai-sdk` with three-layer architecture:
  - `LlmClient` trait (defines what the planner needs)
  - `GLMClient` implementation (implements LlmClient with reev-specific logic)
  - `ZaiClient` (provides generic GLM interactions)
- Partially updated `reev-agent` (legacy, not prioritized for deprecation)
- Removed duplicate GLM client implementations