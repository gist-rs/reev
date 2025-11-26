# Reev Core Implementation Issues

## Issue #100: Refactor Large Files (NOT STARTED)
### Status: NOT STARTED
### Description:
Several files exceed the 320-512 line limit specified in project rules, particularly planner.rs which is over 1000 lines. Large files are difficult to maintain and understand, and violate modular architecture principles.

### Success Criteria:
- All files under 320-512 lines as specified in project rules
- planner.rs broken into focused, single-responsibility modules
- Clear interfaces between modules
- Maintained functionality after refactoring
- Updated documentation for new module structure

### Tasks Required:
1. Break down planner.rs into focused modules:
   - planner/mod.rs (main planner interface)
   - planner/llm_integration.rs (LLM client interactions)
   - planner/flow_generation.rs (flow creation logic)
   - planner/parameter_extraction.rs (parameter extraction from prompts)
2. Check other files for line limit violations and refactor as needed
3. Update imports and references after refactoring
4. Run tests to ensure functionality is maintained
5. Update documentation to reflect new module structure

---

## Issue #101: Implement Runtime Validation Against Ground Truth (IN PROGRESS)
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
3. ⏳ Implement FlowValidator enhancement to validate parameters before execution
4. ⏳ Implement result validation against ground truth
5. ⏳ Add validation feedback to rig agent
6. ⏳ Create comprehensive validation error messages
7. ⏳ Add validation to execution flow
8. ⏳ Create tests for parameter and result validation

---

## Issue #102: Implement Comprehensive Error Recovery Strategies (NOT STARTED)
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

## Issue #103: Consolidate Test Utilities (NOT STARTED)
### Status: NOT STARTED
### Description:
There is significant duplication in test helper functions across multiple test files. This violates the DRY principle and makes maintenance difficult.

### Success Criteria:
- Unified test utilities in tests/common/ module
- No duplicated helper functions across test files
- All tests use common utilities
- Easier to maintain and extend test utilities
- Consistent test patterns across all test files

### Tasks Required:
1. Create unified tests/common/ module with:
   - test_setup.rs (wallet/context setup)
   - mock_helpers.rs (mock implementations)
   - assertion_helpers.rs (test assertions)
2. Remove duplicated helper functions across test files
3. Update all test files to use common utilities
4. Ensure tests still pass after refactoring
5. Add documentation for common test utilities

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

---

## Issue #105: Remove Outdated Code (NOT STARTED)
### Status: NOT STARTED
### Description:
There is outdated code from previous iterations that is no longer used but still present in the codebase. This increases complexity and maintenance burden.

### Success Criteria:
- Remove unused code from previous iterations
- Deprecate components that have been replaced by V3 implementations
- Clean up unused imports and dead code
- Update documentation to reflect current architecture
- Ensure no regressions after cleanup

### Tasks Required:
1. Identify unused code with code analysis tools
2. Remove or deprecate outdated components
3. Clean up unused imports
4. Update documentation to remove references to deprecated components
5. Run tests to ensure no regressions
6. Document removed components for reference

---

## Issue #106: Implement Prompt Refinement Phase (NOT STARTED)
### Status: NOT STARTED
### Description:
Current implementation doesn't have proper LLM-based prompt refinement as specified in V3 plan. The LLM should focus solely on refining user language without determining tools or structure.

### Success Criteria:
- LLM integration for prompt refinement only (not structure generation)
- Typos and language variations normalized
- Refined prompts used for tool selection in Phase 2
- Tests validating prompt refinement quality
- Integration with rule-based YML generation

### Tasks Required:
1. Implement LLM prompt refinement service
2. Create templates for common refinement patterns
3. Add tests for prompt refinement quality
4. Integrate with existing YML generation
5. Add logging for refinement process

---

## Issue #107: Implement Rig Agent for Tool Selection (NOT STARTED)
### Status: NOT STARTED
### Description:
Phase 2 should use rig agent for tool selection and parameter extraction from refined prompts instead of direct tool calls. This requires implementing rig integration.

### Success Criteria:
- Rig agent integrated for tool selection
- Parameter extraction from refined prompts via LLM
- Tool execution through rig framework
- Fallback to direct execution functions
- Tests validating rig-based execution

### Tasks Required:
1. Create rig agent with available tools (SolTransfer, JupiterSwap, etc.)
2. Implement parameter extraction from refined prompts
3. Integrate tool execution through rig framework
4. Add fallback to direct execution functions
5. Create tests for rig-based execution

---

### Implementation Priority

### Week 1:
- Issue #100: Refactor Large Files (start with planner.rs)
- Issue #106: Implement Prompt Refinement Phase

### Week 2:
- Issue #100: Continue refactoring large files
- Issue #103: Consolidate Test Utilities
- Issue #107: Implement Rig Agent for Tool Selection

### Week 3:
- Issue #101: Implement Runtime Validation Against Ground Truth
- Issue #105: Remove Outdated Code

### Week 4:
- Issue #102: Implement Comprehensive Error Recovery Strategies

## Notes:
- All issues should be implemented following V3 plan architecture
- Maintain backward compatibility during transition where possible
- Focus on clear separation of concerns between components
- Ensure comprehensive testing before production deployment
- Keep files under 320-512 lines as specified in project rules