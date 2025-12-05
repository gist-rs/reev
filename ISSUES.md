# Reev Project Issues

## Current Issues

### Issue #314: ZAI SDK Streaming Implementation
**Status**: Low Priority  
**Priority**: Low  
**Description**: Current zai-sdk streaming implementation is simplified (returns non-streaming as single chunk).

**Current Implementation:**
- Located in `crates/zai-sdk/src/client.rs` in `ZaiStreamHandler` implementation
- Simplified approach converts non-streaming response to a single-chunk stream
- Works for current needs but not true streaming

**Impact:**
- Not affecting current functionality
- Could be enhanced in future if real-time streaming is needed
- Low priority as current implementation meets requirements

### Issue #315: Documentation and Test Coverage Enhancement
**Status**: Low Priority  
**Priority**: Low  
**Description**: While basic documentation exists, it could be expanded with more examples.

**Current State:**
- Basic documentation in place
- Tests cover critical functionality
- Could be improved with more integration test coverage

**Impact:**
- Not blocking current development
- Could be enhanced incrementally as needed
- Low priority for now

## Resolved Issues (Last 10)

### Issue #313: ZAI SDK Consolidation - GLM Client Architecture ✅ RESOLVED
**Original Concern**: Whether the three-layer architecture (LlmClient → GLMClient → ZaiClient) was appropriate
**Resolution**: Confirmed this architecture correctly separates concerns:
- `LlmClient`: Defines what the planner needs
- `GLMClient`: Implements LlmClient with reev-specific logic
- `ZaiClient`: Provides generic GLM interactions
**Decision**: Maintain current architecture to keep reev-specific logic contained while keeping zai-sdk project-agnostic

### Issue #312: Structured LLM Response System - Phase 5 Completion ✅ RESOLVED
**Description**: Implementation of structured LLM response system
**Resolution**: 
- Completed comprehensive test coverage with `structured_llm_test.rs`
- Centralized gas reserve calculations in `gas_reserve/mod.rs`
- Updated all components to use standardized gas reserve values
- Fixed planner_test blocking issue

### Issue #311: Gas Reserve Calculation Inconsistencies ✅ RESOLVED
**Description**: Inconsistent gas reserve calculations across components
**Resolution**:
- Created `crates/reev-core/src/gas_reserve/mod.rs` with standardized constants
- Updated all components to use centralized values
- Added comprehensive test suite