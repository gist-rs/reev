# Reev Core Implementation Issues

## Issue #89: Complete RigAgent Implementation (COMPLETED)
### Status: COMPLETED
### Description:
The RigAgent implementation was incomplete - the `prompt_agent` method returned "Prompt agent not implemented yet". This has been fixed with a full implementation that supports both real LLM API calls and mock responses for testing.

### Success Criteria:
- ✅ Implement the `prompt_agent` method to call an LLM API
- ✅ Complete the `execute_step_with_rig` method to properly process responses
- ✅ Ensure the RigAgent can select and execute tools based on expected_tools hints
- ✅ Add comprehensive error handling and validation

### Tasks Required:
1. ✅ Implement actual LLM API integration in `prompt_agent` method
2. ✅ Update `execute_step_with_rig` to handle real LLM responses
3. ✅ Add proper parameter extraction and validation
4. ✅ Complete tool execution with mock blockchain integration for testing
5. ✅ Add comprehensive error handling and recovery strategies

## Issue #83: Implement LLM Language Refinement in Phase 1
### Status: COMPLETED
### Description:
Current implementation uses LLM for both language refinement and structure generation. According to V3 plan, LLM should only refine language in Phase 1, while rule-based templates handle YML structure generation.

### Success Criteria:
- LLM only refines language, fixes typos, and normalizes terminology
- Rule-based templates generate YML structure based on refined prompts
- Clear separation between language refinement and structure generation
- Tests validate language refinement quality and structure generation

### Tasks Required:
1. ✅ Create LanguageRefiner component that uses LLM only for language refinement
2. ✅ Create YmlGenerator component with rule-based templates for structure
3. ✅ Implement prompt refinement tests
4. ✅ Create rule-based templates for common operations (swap, lend, transfer)
5. ✅ Integrate components into existing planner

---

## Issue #90: Missing Test Function in rig_agent_e2e_test
### Status: COMPLETED
### Description:
The rig_agent_e2e_test.rs file contains helper functions but lacks an actual test function with #[tokio::test] attribute. The infrastructure exists but the test cannot be executed because there's no entry point to run the code.

### Problems Identified:
1. Missing test function - only helper functions exist (ensure_surfpool_running, setup_wallet, execute_transfer_with_rig_agent, run_rig_agent_transfer_test)
2. None of the helper functions are called, causing "never used" compiler warnings
3. No validation that RigAgent actually performs tool selection and parameter extraction
4. No assertions to verify the end-to-end functionality works correctly

### Success Criteria:
- Add a proper test function that uses the existing helper functions
- Verify RigAgent is actually being used for tool selection, not falling back to direct execution
- Validate tool selection based on expected_tools hints
- Validate parameter extraction from refined prompts
- Ensure the test can be run with `cargo test -p reev-core --test rig_agent_e2e_test test_rig_agent_transfer`

### Tasks Required:
1. ✅ Add a test function with #[tokio::test] attribute that calls the existing helper functions
2. ✅ Add proper signature extraction to verify RigAgent functionality
3. ✅ Fix the unused client field in RigAgent struct
4. ✅ Ensure proper integration between Executor with RigAgent and the test
5. ✅ Verify that ZAI_API_KEY is properly loaded and used

### Fixes Applied:
1. Added test_rig_agent_transfer() function with #[tokio::test] attribute
2. Fixed signature extraction to handle multiple possible output structures from RigAgent
3. Removed unused client field from RigAgent struct
4. Fixed proper integration between Executor with RigAgent and test
5. Verified ZAI_API_KEY is loaded from .env file

---

## Issue #84: Implement Rig Framework for Tool Selection in Phase 2
### Status: COMPLETED
### Description:
Current implementation uses direct tool calls rather than rig framework for tool selection and calling. V3 plan requires rig framework for LLM-driven tool selection and parameter extraction.

### Success Criteria:
- Rig agent selects tools based on refined prompts
- LLM extracts parameters from refined prompts
- Tool calling handled through rig framework
- Existing direct execution functions maintained as fallbacks
- Tests validate rig-based tool selection and execution

### Progress:
- ✅ Added expected_tools hints to YML steps to guide rig agent
- ✅ Updated YmlFlow and YmlStep schemas with refined_prompt field
- ✅ Implemented rule-based fallback for LLM requests when API fails
- ✅ Created RigAgent component for tool selection and calling
- ✅ Integrated rig framework with existing tool definitions
- ✅ Implemented parameter extraction via LLM
- ✅ Added expected_tools hints in YML steps for rig guidance
- ✅ Created tests for rig-driven execution
- ✅ Maintained direct execution functions as fallbacks

### Tasks Required:
1. ✅ Create RigAgent component for tool selection and calling
2. ✅ Integrate rig framework with existing tool definitions
3. ✅ Implement parameter extraction via LLM
4. ✅ Add expected_tools hints in YML steps for rig guidance
5. ✅ Create tests for rig-driven execution
6. ✅ Maintain direct execution functions as fallbacks

---

## Issue #85: Implement Runtime Validation Against Ground Truth
### Status: NOT STARTED
### Description:
Ground truth validation exists but is not used during execution. V3 plan requires runtime validation against ground truth during execution to ensure constraints are met.

### Success Criteria:
- Parameters validated against ground truth before execution
- Results validated against expected outcomes after execution
- Validation failures trigger appropriate error recovery
- Clear validation error messages for debugging
- Tests validate parameter and result validation

### Tasks Required:
1. ✅ Enhanced YML schema with refined_prompt and expected_tools fields
2. ✅ Updated YmlGenerator to include expected_tools hints in generated flows
3. Implement FlowValidator enhancement to validate parameters before execution
4. Implement result validation against ground truth
5. Add validation feedback to rig agent
6. Create comprehensive validation error messages
7. Add validation to execution flow
8. Create tests for parameter and result validation

---

## Issue #86: Implement Comprehensive Error Recovery Strategies
### Status: NOT STARTED
### Description:
Current implementation has basic error handling but lacks sophisticated recovery strategies. V3 plan requires intelligent error recovery based on error types.

### Success Criteria:
- Parameter validation failures trigger parameter adjustments
- Tool execution failures trigger retries with backoff or alternative tools
- Result validation failures trigger specific suggestions
- Error recovery integrates with rig agent for tool selection changes
- Tests validate all error recovery scenarios

### Tasks Required:
1. Create ErrorRecoveryEngine with strategy patterns
2. Implement parameter adjustment strategies
3. Implement retry logic with backoff
4. Add alternative tool selection via rig agent
5. Create comprehensive error reporting
6. Add tests for all error recovery scenarios

---

## Issue #91: RigAgent Implementation Gaps
### Status: COMPLETED
### Description:
The RigAgent implementation has several gaps that prevent it from being fully functional and testable. These gaps need to be addressed to ensure the RigAgent can properly select tools and extract parameters for Phase 2 execution.

### Problems Identified:
1. Unused client field in RigAgent struct generates compiler warnings
2. Limited error handling in LLM API calls
3. No validation that the selected tool matches expected_tools hints
4. Parameter extraction may not work correctly for all tool types
5. No fallback mechanism when LLM fails to provide valid tool calls

### Success Criteria:
- All compiler warnings in RigAgent implementation resolved
- Robust error handling for LLM API failures
- Validation that selected tools match expected_tools hints
- Parameter extraction works correctly for all supported tools
- Fallback mechanism when LLM fails to provide valid responses

### Tasks Required:
1. ✅ Fix unused client field in RigAgent struct
2. ⏳ Add comprehensive error handling for LLM API calls
3. ✅ Add validation to ensure selected tools match expected_tools hints
4. ✅ Improve parameter extraction for all supported tools
5. ⏳ Implement fallback mechanism when LLM fails to provide valid responses
6. ✅ Add end-to-end test for RigAgent with ToolExecutor
7. ✅ Verify RigAgent properly executes real blockchain transactions

---

## Issue #92: Add Expected_Tools Hints to YML Steps
### Status: COMPLETED
### Description:
YML steps need expected_tools hints to guide rig agent tool selection. This helps the LLM select appropriate tools for each step.

### Success Criteria:
- YML step schema includes expected_tools field
- YmlGenerator populates expected_tools based on operation type
- Rig agent uses expected_tools as hints for tool selection
- Fallback behavior when expected_tools is missing or incorrect
- Tests validate expected_tools hinting functionality

### Tasks Required:
1. ✅ Update YmlStep schema to include expected_tools field
2. ✅ Modify YmlGenerator to populate expected_tools
3. ⏳ Update RigAgent to use expected_tools as hints
4. ⏳ Implement fallback behavior for missing/incorrect hints
5. ✅ Add tests for expected_tools hinting

---

## Issue #93: Implement Comprehensive Testing for V3 Components
### Status: IN PROGRESS
### Description:
New V3 components need comprehensive testing to ensure reliability. This includes unit tests, integration tests, and end-to-end tests.

### Success Criteria:
- Unit tests for all new components (LanguageRefiner, YmlGenerator, RigAgent, etc.)
- Integration tests for component interactions
- End-to-end tests for complete V3 workflow
- Performance benchmarks for V3 components
- Coverage reports showing >80% test coverage

### Tasks Required:
1. ✅ Create unit tests for LanguageRefiner component
2. ✅ Create unit tests for YmlGenerator component
3. ⏳ Create unit tests for RigAgent component (implementation gaps need to be addressed first)
4. ⏳ Create integration tests for component interactions
5. ⏳ Create end-to-end tests for V3 workflow (rig_agent_e2e_test needs actual test function)
6. ⏳ Add performance benchmarks
7. ⏳ Generate and review coverage reports

### Current Issues:
- rig_agent_e2e_test.rs has no actual test function, only helper functions
- RigAgent has unused client field that needs to be fixed before testing
- Need to validate that RigAgent is actually being used in tests, not just falling back to direct execution

---

## Implementation Priority

### Week 1-2:
- Issue #83: Implement LLM Language Refinement in Phase 1
- Issue #87: Add Expected_Tools Hints to YML Steps

### Week 2-3:
- Issue #84: Implement Rig Framework for Tool Selection in Phase 2
- Issue #85: Implement Runtime Validation Against Ground Truth

### Week 3-4:
- Issue #86: Implement Comprehensive Error Recovery Strategies
- Issue #88: Implement Comprehensive Testing for V3 Components

### Migration Strategy:
1. Parallel development of Phase 1 and Phase 2 components
2. Gradual integration with existing code
3. Maintaining backward compatibility during transition
4. Phased rollout with fallback mechanisms
5. Performance monitoring and optimization

## Notes:
- All issues should be implemented following V3 plan architecture
- Maintain backward compatibility during transition
- Focus on clear separation of concerns between components
- Ensure comprehensive testing before production deployment