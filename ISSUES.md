# Reev Project Issues

## Current Issues

### Issue #312: Structured LLM Response System - Implementation Gaps and Issues
**Status**: In Progress  
**Priority**: High  
**Description**: Analysis of the current implementation against PLAN_ALL.md and TASKS.md reveals significant gaps between planned and implemented structured LLM response system.

**What's Implemented (Phases 1-4 mostly complete):**

1. **Phase 1: Structured Data Types** - FULLY IMPLEMENTED
   - Location: `crates/reev-core/src/prompt_processor/types.rs`
   - `StructuredRefinedPrompt` struct with all required fields
   - `PromptAction` enum with all required variants
   - `PromptParameters` struct with required fields

2. **Phase 2: Prompt Processing** - MOSTLY IMPLEMENTED
   - Location: `crates/reev-core/src/prompt_processor/mod.rs`
   - Structured system prompt implemented
   - `process_prompt_structured` method implemented
   - "All" keyword handling implemented

3. **Phase 3: Execution Flow** - PARTIALLY IMPLEMENTED
   - Location: `crates/reev-core/src/execution/rig_agent/mod.rs`
   - `execute_step_with_rig_and_history` uses structured fields when available
   - `create_tool_calls_from_structured_data` method implemented

4. **Phase 4: YML Generation** - MOSTLY IMPLEMENTED
   - Location: `crates/reev-core/src/yml_generator/mod.rs`
   - `generate_flow_from_structured_prompt` implemented
   - Handles all action types (Transfer, Swap, Lend, Earn, Borrow)
   - `YmlStep` includes `structured_prompt` field

**Critical Issues (Why they matter):**

1. **Missing Phase 5 Implementation**:
   - What's missing: Comprehensive test coverage for structured responses
   - Why it matters: Without tests, we can't ensure reliability of the system
   - Where: Missing `structured_llm_test.rs` file mentioned in TASKS.md

2. **Fallback Mechanism Violates "No Fallback" Principle**:
   - What's wrong: System falls back to rule-based parsing when structured response fails
   - Why it's cheating: PLAN_ALL.md specifically states "no fallback, aim for LLM as a plan"
   - Where: `process_prompt_structured` in `prompt_processor/mod.rs` lines 320-350

3. **Incomplete Validation Logic**:
   - What's wrong: Validation relies on simple keyword matching rather than parameter consistency
   - Why it's risky: Invalid responses could pass validation by containing right keywords
   - Where: `validate_structured_response` in `prompt_processor/validation.rs`

4. **Double Gas Deduction Not Fully Fixed**:
   - What's wrong: Gas reserve is calculated in multiple places
   - Why it's problematic: PLAN_ALL.md highlights this as a key issue to solve
   - Where: `process_prompt_structured` and various tool implementations

5. **Incomplete Direct Parameter Extraction**:
   - What's wrong: Execution sometimes still falls back to parsing text
   - Why it's not pure: Violates the goal of using structured data directly
   - Where: Mixed approach in `execute_step_with_rig_and_history`

**Immediate Action Items:**

1. Remove fallback mechanisms to fully implement "LLM as a plan"
2. Enhance validation logic to check parameter consistency, not just keywords
3. Create comprehensive test suite for structured responses
4. Fix double gas deduction by centralizing gas reserve calculation
5. Ensure all tool execution uses structured parameters directly

**Success Criteria Yet to Meet:**

1. Eliminate ALL fallbacks to rule-based parsing
2. Implement robust validation that ensures parameter consistency
3. Achieve >90% test coverage for structured response system
4. Fix double gas deduction completely

