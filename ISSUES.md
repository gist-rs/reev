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
### Status: IN PROGRESS
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
3. ⏳ Enhance FlowValidator to validate parameters before execution
4. ⏳ Implement result validation against ground truth
5. ⏳ Add validation feedback to rig agent
6. ⏳ Create comprehensive validation error messages
7. ⏳ Add validation to execution flow
8. ⏳ Create tests for parameter and result validation

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

## Issue #87: Add Expected_Tools Hints to YML Steps
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

## Issue #88: Implement Comprehensive Testing for V3 Components
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
3. ✅ Create unit tests for RigAgent component
4. ⏳ Create integration tests for component interactions
5. ✅ Create end-to-end tests for V3 workflow
6. ⏳ Add performance benchmarks
7. ⏳ Generate and review coverage reports

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