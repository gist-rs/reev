# Reev Project Issues

## Current Issues

### Issue #312: Structured LLM Response System - Implementation Gaps and Issues
**Status**: In Progress  
**Priority**: High  
**Description**: Analysis of the current implementation against PLAN_ALL.md and TASKS.md reveals significant gaps between planned and implemented structured LLM response system.

**Recent Progress (Dec 2024)**:
- ✅ Fixed planner_test blocking issue by adding `#[tokio::test(flavor = "multi_thread")]` attribute
- ✅ Created comprehensive test file `structured_llm_test.rs` with 12 tests covering validation, parsing, and builder patterns
- ✅ Updated documentation to reflect current implementation state

**What's Implemented (Phases 1-4 mostly complete):**

1. **Phase 1: Structured Data Types** - FULLY IMPLEMENTED
   - Location: `crates/reev-core/src/prompt_processor/types.rs`
   - `StructuredRefinedPrompt` struct with all required fields
   - `PromptAction` enum with all required variants
   - `PromptParameters` struct with required fields
   - Added `usable_amount` field for handling "all" keyword

2. **Phase 2: Prompt Processing** - FULLY IMPLEMENTED
   - Location: `crates/reev-core/src/prompt_processor/mod.rs` and `structured_processor.rs`
   - Structured system prompt implemented
   - `process_prompt_structured` method implemented
   - "All" keyword handling implemented with MaxAmountCalculator
   - Validation logic implemented in `validation.rs`

3. **Phase 3: Execution Flow** - PARTIALLY IMPLEMENTED
   - Location: `crates/reev-core/src/execution/rig_agent/mod.rs`
   - `execute_step_with_rig_and_history` uses structured fields when available
   - `create_tool_calls_from_structured_data` method implemented
   - Still has fallback to rule-based extraction for some operations

4. **Phase 4: YML Generation** - FULLY IMPLEMENTED
   - Location: `crates/reev-core/src/yml_generator/mod.rs`
   - `generate_flow_from_structured_prompt` implemented
   - Handles all action types (Transfer, Swap, Lend, Earn, Borrow)
   - `YmlStep` includes `structured_prompt` field

**Critical Issues (Why they matter):**

1. **Phase 5 Implementation** - COMPLETED
   - What's done: Created comprehensive test coverage for structured responses
   - Where: `crates/reev-core/tests/structured_llm_test.rs` with 12 tests

2. **Incomplete Parameter Extraction in Execution**:
   - What's wrong: Execution still has partial fallback to rule-based parsing
   - Why it's not pure: Violates the goal of using structured data directly
   - Where: `execute_step_with_rig_and_history` and `extract_multi_step_tool_calls`

3. **Inconsistent Gas Reserve Calculation**:
   - What's wrong: Gas reserve is calculated in multiple places with different values
   - Why it's problematic: PLAN_ALL.md highlights this as a key issue to solve
   - Where: `MaxAmountCalculator`, various tool implementations (0.001-0.05 SOL)

4. **Validation Logic Relies on Keyword Matching**:
   - What's wrong: Validation relies on simple keyword matching rather than parameter consistency
   - Why it's risky: Invalid responses could pass validation by containing right keywords
   - Where: `validate_structured_response` in `prompt_processor/validation.rs`

5. **Fallback Mechanism with mock data**:
   - What's wrong: Fallback when structured response fails
   - Why it's problematic: Could result in mislead, wrong data
   - Where: `process_prompt_structured` returns error when validation fails? if so it's good nvm just re-check.

**Immediate Action Items:**

1. Eliminate remaining rule-based parsing in execution
   - Focus: `extract_multi_step_tool_calls` in `rig_agent/mod.rs`
   - Target: Remove all regex parsing when structured data is available

2. Standardize gas reserve calculation across components
   - Focus: Consolidate gas calculation in `MaxAmountCalculator`
   - Target: Replace hard-coded values in tool implementations

3. Enhance validation logic to check parameter consistency
   - Focus: Add parameter validation beyond keyword matching
   - Target: Validate actual parameter values against constraints

4. Add appropriate fallback mechanism with proper logging
   - Focus: Graceful degradation when structured response fails
   - Target: Never crash on unexpected input

5. Expand test coverage for edge cases and multi-step operations
   - Focus: Multi-step prompts and edge cases
   - Target: Add tests to existing `structured_llm_test.rs`

**Success Criteria Progress:**

1. ✅ Create comprehensive test suite for structured responses (ACHIEVED)
   - File: `crates/reev-core/tests/structured_llm_test.rs`
   - Coverage: 12 tests for validation, parsing, and builder patterns

2. 🔄 Eliminate ALL fallbacks to rule-based parsing (IN PROGRESS)
   - Location: `crates/reev-core/src/execution/rig_agent/mod.rs`
   - Target: Remove regex parsing when structured data available

3. 🔄 Implement robust validation that ensures parameter consistency (IN PROGRESS)
   - Location: `crates/reev-core/src/prompt_processor/validation.rs`
   - Target: Add parameter value validation

4. 🔄 Standardize gas reserve calculation across all components (IN PROGRESS)
   - Location: `crates/reev-core/src/prompt_processor/max_amount_calculator.rs`
   - Target: Replace hard-coded values in tool implementations

5. 🔄 Fix planner_test blocking issue (COMPLETED)
   - Solution: Added `#[tokio::test(flavor = "multi_thread")]` attribute
   - Result: All tests now pass
