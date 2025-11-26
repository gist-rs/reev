# Reev Core Implementation Issues

## Issue #100: Remove Duplicated Flow Creation Functions (NOT STARTED)
### Status: NOT STARTED
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

### Tasks Required:
1. Identify all unused functions in planner.rs related to flow creation
2. Remove these functions and their related code
3. Ensure no code references the removed functions
4. Run tests to verify existing functionality still works
5. Update documentation to reflect current implementation path
6. Add tests to verify removal doesn't break functionality

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

## Issue #103: Improve YML Generator Scalability (NOT STARTED)
### Status: NOT STARTED
### Description:
Current YmlGenerator uses pattern matching for fixed operation types (swap, transfer, lend, swap_then_lend), which doesn't scale well. Adding new operation combinations requires new methods and pattern matches. This approach doesn't support dynamic, arbitrary sequences of operations.

### Success Criteria:
- YmlGenerator refactored for dynamic operation sequences
- OperationParser implemented for flexible operation detection
- Composable step builders for dynamic flow creation
- Support for arbitrary operation sequences
- Tests validate complex operation sequences

### Tasks Required:
1. Create OperationParser module:
   - Parse prompts for sequences of operations
   - Support arbitrary operation sequences
   - Extract operation parameters from refined prompts
2. Implement composable step builders:
   - Create individual step builders for each operation type
   - Allow dynamic composition of steps
   - Support parameter passing between steps
3. Refactor YmlGenerator to use OperationParser:
   - Replace fixed operation type matching with dynamic parsing
   - Generate flows based on parsed operation sequences
   - Maintain expected_tools hints for RigAgent
4. Add tests for complex operation sequences

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

## Issue #107: Refactor Large Files (NOT STARTED)
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
1. Break down large files into focused modules:
   - Check planner.rs after removing duplicate functions
   - Check yml_generator.rs for potential splitting
   - Check executor.rs for potential splitting
2. Create focused modules with single responsibilities
3. Update imports and references after refactoring
4. Run tests to ensure functionality is maintained
5. Update documentation to reflect new module structure

---

### Implementation Priority

### Week 1:
- Issue #100: Remove Duplicated Flow Creation Functions (start with planner.rs)
- Issue #103: Improve YML Generator Scalability (OperationParser for dynamic sequences)

### Week 2:
- Issue #101: Integrate Validation Into Execution Flow
- Issue #107: Refactor Large Files (check remaining files after removing duplicates)

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