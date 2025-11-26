# Reev Core Implementation Issues

## Issue #100: Remove Duplicated Flow Creation Functions (COMPLETED)
### Status: COMPLETED
### Description:
There is duplication in the codebase with flow creation functions like `create_swap_then_lend_flow`, `create_swap_flow`, etc. appearing in both planner.rs and yml_schema.rs. The planner.rs versions are not used by any current code or tests (marked as "never used" by compiler), but they create confusion and maintenance burden.

### Success Criteria:
- Remove unused flow creation functions in planner.rs:
  - generate_flow_rule_based()
  - create_swap_flow()
  - create_transfer_flow()
  - create_lend_flow()
  - create_swap_then_lend_flow()
  - Related helper functions like parse_intent(), extract_swap_params(), etc.
- Keep builder functions in yml_schema.rs for testing
- Existing tests still pass after removal
- Documentation updated to reflect current implementation
- No compiler warnings about dead code

### Tasks Completed:
1. ✅ Identified all unused functions in planner.rs related to flow creation
2. ✅ Removed these functions and their related code
3. ✅ Ensured no code references the removed functions
4. ✅ Created a cleaner planner.rs with only essential functions (425 lines, within 320-512 line limit)
5. ✅ Split yml_generator into smaller modules (mod.rs: 189 lines, operation_types.rs: 179 lines, flow_builders.rs: 242 lines)
6. ✅ Fixed all compilation errors and warnings

---

## Issue #101: Integrate Validation Into Execution Flow (NOT STARTED)
### Status: NOT STARTED
### Description:
FlowValidator exists in validation.rs but is not fully integrated into the execution flow. V3 plan requires runtime validation against ground truth during execution to ensure constraints are met.

### Success Criteria:
- FlowValidator integrated into Executor before execution
- Parameters validated against ground truth before tool execution
- Results validated against expected outcomes after execution
- Validation failures trigger appropriate error recovery
- Clear validation error messages for debugging
- Tests validate parameter and result validation

### Tasks Required:
1. Integrate FlowValidator into Executor.execute_flow():
   - Add parameter validation before tool execution
   - Add result validation after tool execution
   - Add validation error handling
2. Enhance FlowValidator with more comprehensive checks:
   - Parameter constraint validation
   - Wallet context validation
   - Result state validation
3. Add validation error handling to Executor
4. Create comprehensive validation error messages
5. Add validation feedback to RigAgent
6. Create tests for parameter and result validation

---

## Issue #102: Implement Error Recovery Engine (NOT STARTED)
### Status: NOT STARTED
### Description:
Current implementation has basic error handling but lacks sophisticated recovery strategies. V3 plan requires intelligent error recovery based on error types.

### Success Criteria:
- ErrorRecoveryEngine implemented with strategy patterns
- Parameter validation failures trigger parameter adjustments
- Tool execution failures trigger retries with backoff or alternative tools
- Result validation failures trigger specific suggestions
- Error recovery integrates with RigAgent for tool selection changes
- Tests validate all error recovery scenarios

### Tasks Required:
1. Create ErrorRecoveryEngine in a new module:
   - Define error types and recovery strategies
   - Implement parameter adjustment strategies
   - Implement retry logic with backoff
   - Add alternative tool selection via RigAgent
2. Integrate ErrorRecoveryEngine into Executor:
   - Add error recovery calls after validation failures
   - Add error recovery calls after tool execution failures
   - Add error recovery calls after result validation failures
3. Create comprehensive error reporting
4. Add tests for all error recovery scenarios

---

## Issue #103: Fix OperationParser Architecture Mismatch (IN PROGRESS)
### Status: IN PROGRESS
## Issue #108: Fix LanguageRefiner Hardcoded Fallback Response (COMPLETED)
### Status: COMPLETED
### Description:
LanguageRefiner had hardcoded fallback responses in the `extract_refined_prompt_from_reasoning` function that were changing operation types, violating the V3 plan. When LLM response parsing failed, it was returning "Send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq" which changed "swap 0.1 SOL for USDC" to a transfer operation with a specific recipient address. This created a fundamental architectural violation where operation types were being changed during refinement.

### Success Criteria:
- Remove hardcoded fallback responses that change operation types
- Ensure operation type preservation during language refinement
- Fix fallback logic to preserve original operation type (swap, transfer, lend)
- Add proper error handling when LLM response can't be parsed
- Test with "swap 0.1 SOL for USDC" to ensure it remains a swap operation

### Tasks Completed:
1. Fixed LanguageRefiner to preserve operation types:
   - Ensured "swap" operations aren't changed to "transfer" during refinement
   - Removed hardcoded fallback responses that change operation types
   - Preserve tokens and amounts in refined prompts
   - Updated system prompt to emphasize operation type preservation
   - ✅ Fixed hardcoded fallback responses to return original prompt unchanged
   - ✅ Enhanced system prompt to emphasize operation type preservation

2. Updated YmlGenerator to not pre-determine operations:
   - Simplified YmlGenerator to create steps with refined prompts only
   - Removed operation extraction from refined prompts
   - Removed expected_tools pre-determination from YML steps
   - Generated simpler YML steps with just refined prompts
   - ✅ Removed OperationParser usage from UnifiedFlowBuilder
   - ✅ Simplified YmlGenerator to create steps with refined prompts only
   - ✅ Removed pre-determination of expected_tools

3. Refactored OperationParser per V3 plan:
   - Kept OperationParser for flexible YML generation as per V3 Phase 2
   - Don't use it to extract operations from refined prompts
   - Use it for composable step creation in YML generation
   - Implemented operation types defined in V3 plan
   - ✅ Removed OperationParser usage from YML generation
   - ✅ Simplified tests to verify operation type preservation

4. Fixed architectural alignment with V3 plan:
   - Ensured RigAgent receives refined prompts directly
   - Implemented proper LLM-driven tool selection based on refined prompts
   - Removed dependency on pre-determined expected_tools
   - Extract parameters dynamically using LLM, not regex

5. Update execution flow to use RigAgent properly:
   - Pass refined prompts directly to RigAgent
   - Let RigAgent determine tools and parameters
   - Remove dependency on pre-determined expected_tools

---

## Issue #104: Fix E2E Test Failures (COMPLETED)
### Status: COMPLETED
### Description:
The e2e tests (e2e_swap.rs, e2e_transfer.rs, e2e_rig_agent.rs) are not working correctly. These tests are critical for validating the end-to-end functionality of the system.

### Success Criteria:
- All e2e tests pass consistently
- Tests properly validate actual blockchain transactions
- Clear error messages for test failures
- Tests work with the current V3 implementation
- Tests provide good coverage of common use cases

### Tasks Completed:
1. ✅ Fixed e2e_swap.rs to work with current implementation
2. ✅ Refactored test files to use common utilities in tests/common/
3. ✅ Added serial_test annotation to ensure tests run sequentially
4. ✅ Updated test setup and teardown procedures
5. ✅ Enhanced error reporting for test failures
6. ✅ Documented test setup requirements
7. ✅ Verified tests use current implementation path (refine_and_plan → LanguageRefiner → YmlGenerator → Executor)

---

## Issue #108: Fix LanguageRefiner Hardcoded Fallback Response (COMPLETED)
### Status: COMPLETED
### Description:
LanguageRefiner has hardcoded fallback responses in the `extract_refined_prompt_from_reasoning` function that change operation types, violating V3 plan. When LLM response parsing fails, it returns "Send 1 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq" which changes "swap 0.1 SOL for USDC" to a transfer operation with a specific recipient address. This creates a fundamental architectural violation where operation types are being changed during refinement.

### Success Criteria:
- Remove hardcoded fallback responses that change operation types
- Ensure operation type preservation during language refinement
- Fix the fallback logic to preserve original operation type (swap, transfer, lend)
- Add proper error handling when LLM response can't be parsed
- Test with "swap 0.1 SOL for USDC" to ensure it remains a swap operation

### Tasks Required:
1. Fix the hardcoded fallback responses in `extract_refined_prompt_from_reasoning`:
   - The current implementation always returns "Send 1 SOL to [address]" as fallback
   - This changes swap operations to transfer operations
   - It also adds a recipient address that wasn't in the original prompt

2. Improve fallback logic to preserve operation types:
   - Detect the operation type from the original prompt or LLM reasoning
   - Return a generic prompt that preserves the operation type
   - Don't add recipient addresses that weren't in the original prompt

3. Add proper error handling:
   - When LLM response can't be parsed, preserve the original prompt
   - Or return a simple generic prompt that preserves operation type
   - Don't change operation types in fallback scenarios

4. Update tests to verify operation type preservation:
   - Test that "swap 0.1 SOL for USDC" remains a swap operation
   - Test that "transfer 1 SOL to address" remains a transfer operation
   - Test that "lend 100 USDC" remains a lend operation

---

## Issue #109: Fix OperationParser Architectural Mismatch (COMPLETED)
### Status: COMPLETED
### Description:
Current implementation incorrectly used OperationParser to extract operations from refined prompts using regex, which violated the V3 plan architecture. According to V3, RigAgent should use refined prompts directly for LLM-driven tool selection, not rely on rule-based operation parsing. The OperationParser should only be used in YML generation for creating flexible YML structures, not for extracting operations from already refined prompts.

### Success Criteria:
- Remove OperationParser usage for extracting operations from refined prompts
- Ensure RigAgent uses refined prompts directly for tool selection
- Implement proper YML generation that doesn't pre-determine operations
- Follow V3 plan architecture: LanguageRefiner → YmlGenerator → RigAgent
- Preserve operation intent during language refinement

### Tasks Completed:
1. Removed OperationParser usage in UnifiedFlowBuilder:
   - Removed parsing operations from refined prompts
   - Eliminated unnecessary intermediate processing that LLM should handle
   - Aligned with V3 plan where RigAgent determines tools from refined prompts

2. Simplified YmlGenerator to not pre-determine operations:
   - Removed operation extraction from refined prompts
   - Created simple YML steps with just refined prompts
   - Removed pre-determination of expected_tools based on regex parsing
   - Let RigAgent handle tool selection and parameter extraction

3. Updated execution flow to use RigAgent properly:
   - Ensured RigAgent receives refined prompts directly
   - Removed dependency on pre-determined expected_tools
   - Let RigAgent handle tool selection and parameter extraction via LLM

4. Kept OperationParser only for YML generation as per V3:
   - V3 plan mentions OperationParser in Phase 2 for flexible YML generation
   - Used it for composable step builders, not extract operations
   - Made it a tool for YmlGenerator, not a replacement for RigAgent

---

## Issue #105: Enhance RigAgent Integration (NOT STARTED)
### Status: NOT STARTED
### Description:
RigAgent is implemented but could be enhanced for better tool selection and parameter extraction. Current implementation works but could be more robust with confidence scoring and fallback mechanisms.

### Success Criteria:
- Enhanced RigAgent with confidence scoring for tool selection
- Improved parameter extraction from refined prompts
- Tool selection fallbacks for low-confidence selections
- Better error handling in RigAgent
- Comprehensive tests for RigAgent enhancements

### Tasks Required:
1. Enhance RigAgent tool selection:
   - Add confidence scoring for tool selection
   - Implement tool selection fallbacks
   - Add context-aware tool selection
2. Improve parameter extraction:
   - Enhance parameter extraction from refined prompts
   - Add parameter validation before tool execution
   - Add parameter adjustment strategies
3. Improve RigAgent error handling:
   - Add better error messages
   - Implement error recovery for parameter extraction failures
   - Add alternative tool selection on errors
4. Add comprehensive tests for RigAgent enhancements

---

## Issue #106: Improve LanguageRefiner (NOT STARTED)
### Status: NOT STARTED
### Description:
LanguageRefiner is implemented but could be improved for better prompt refinement quality. Current implementation works but could be more robust with better templates and error handling.

### Success Criteria:
- Enhanced LanguageRefiner with better refinement templates
- Improved error handling for LLM API failures
- Better logging for refinement process
- Tests validating prompt refinement quality
- Integration with YML generation maintained

### Tasks Required:
1. Enhance LanguageRefiner refinement process:
   - Add more sophisticated refinement templates
   - Implement better context preservation
   - Add language-specific refinement rules
2. Improve error handling:
   - Add retry logic for LLM API failures
   - Implement fallback refinement strategies
   - Add better error messages
3. Add comprehensive logging for refinement process
4. Add tests for prompt refinement quality
5. Document refinement process and templates

---

## Issue #107: Refactor Large Files (COMPLETED)
### Status: COMPLETED
### Description:
Several files exceed the 320-512 line limit specified in project rules, particularly planner.rs which is over 1000 lines. Large files are difficult to maintain and understand, and violate modular architecture principles.

### Success Criteria:
- All files under 320-512 lines as specified in project rules
- planner.rs broken into focused, single-responsibility modules
- Clear interfaces between modules
- Maintained functionality after refactoring
- Updated documentation for new module structure

### Tasks Completed:
1. ✅ Refactored planner.rs from 1035 lines to 425 lines (within 320-512 line limit)
2. ✅ Split yml_generator/mod.rs from 546 lines into:
   - mod.rs: 189 lines
   - operation_types.rs: 179 lines
   - flow_builders.rs: 242 lines
3. ✅ Created focused modules with single responsibilities
4. ✅ Updated imports and references after refactoring
5. ✅ Verified functionality is maintained with cargo check

---

### Implementation Priority

### Week 1:
- ✅ Issue #100: Remove Duplicated Flow Creation Functions (completed)
- ✅ Issue #104: Fix E2E Test Failures (completed)
- ✅ Issue #107: Refactor Large Files (completed)
- ✅ Issue #108: Fix LanguageRefiner Hardcoded Fallback Response (completed)
- ✅ Issue #109: Fix OperationParser Architectural Mismatch (completed)

### Current State Summary (Handover):
Successfully fixed the architectural mismatch that was causing "swap 0.1 SOL for USDC" to be incorrectly interpreted as "Send 1 SOL to address" (transfer operation). The root cause was two-fold:

1. **LanguageRefiner Hardcoded Fallback Responses**: The `extract_refined_prompt_from_reasoning` function had hardcoded fallback responses that changed operation types when LLM response parsing failed.

2. **OperationParser Architectural Mismatch**: The system was using a rule-based OperationParser to extract operations from refined prompts, which violated V3 plan architecture where RigAgent should determine tools from refined prompts directly.

## Key Changes Made:
1. **Fixed LanguageRefiner**:
   - Removed hardcoded fallback responses that changed operation types
   - Updated system prompt to emphasize operation type preservation
   - Now returns original prompt unchanged when parsing fails

2. **Simplified YmlGenerator**:
   - Removed OperationParser usage for extracting operations from refined prompts
   - Simplified YML generation to create steps with just refined prompts
   - Removed pre-determination of expected_tools

3. **Updated RigAgent Integration**:
   - Ensured RigAgent receives refined prompts directly
   - Let RigAgent handle tool selection and parameter extraction via LLM
   - Removed dependency on pre-determined expected_tools

## Testing:
- All tests passing
- Operation type preservation verified for swap, transfer, and lend operations
- YmlGenerator tests verify simple step creation with refined prompts only

This fixes the fundamental issue where operation types were being changed during language refinement, aligning the implementation with V3 plan architecture.

### Week 2:
- Issue #101: Integrate Validation Into Execution Flow (NOT STARTED)
- Issue #103: Improve YML Generator Scalability (NOT STARTED)

### Week 3:
- Issue #102: Implement Error Recovery Engine (strategy patterns)
- Issue #105: Enhance RigAgent Integration (confidence scoring, fallbacks)

### Week 4:
- Issue #106: Improve LanguageRefiner (better templates, error handling)
- Issue #104: Verify all tests still pass after changes

## Notes:
- All issues should be implemented following V3 plan architecture
- Current implementation path: refine_and_plan → LanguageRefiner → YmlGenerator → Executor
- YML builder functions in yml_schema.rs are for testing only
- FlowValidator, RigAgent, and ErrorRecoveryEngine should be integrated into Executor
- Focus on making YmlGenerator more dynamic for arbitrary operation sequences
- Keep files under 320-512 lines as specified in project rules
- Ensure e2e tests (e2e_swap.rs, e2e_transfer.rs, e2e_rig_agent.rs) continue to pass