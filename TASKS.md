# Reev Core Implementation Tasks

## Why: Structured YML Context for AI Operations

The current implementation uses mixed JSON+markdown strings for AI context, which is problematic because:
1. Not parseable back to structs for validation
2. Hard to maintain and extend
3. Verbose and confusing for AI
4. Inconsistent with YML-based architecture in PLAN_CORE_V3.md
5. Difficult to debug and test

We have existing YML schema and builders but RigAgent isn't using them, creating architectural inconsistency.

---

## Task #121: Implement Structured YML Context for AI Operations

### Status: COMPLETED
### Priority: HIGH

### Description:
Replace the current mixed JSON+markdown context generation in RigAgent with structured YML context that:
1. Uses existing YML schema from yml_schema.rs
2. Is parseable back to structs for validation
3. Sends only relevant information to AI
4. Maintains clean, consistent format

### Implementation Summary:
Successfully implemented structured YML context for AI operations with the following components:

1. **YML Context Builder** (crates/reev-core/src/execution/context_builder/mod.rs):
   - ✅ Implemented YmlOperationContext struct for AI operations
   - ✅ Created MinimalAiContext containing only relevant information for AI
   - ✅ Added PreviousStepResult and TokenInfo for structured data
   - ✅ Implemented YmlContextBuilder with builder pattern
   - ✅ Added serialization/deserialization methods for YML/JSON

2. **Updated RigAgent Integration** (crates/reev-core/src/execution/rig_agent/mod.rs):
   - ✅ Replaced create_context_prompt_with_history with YML-based approach
   - ✅ Created create_yml_context method to generate structured context
   - ✅ Added yml_context_to_prompt for LLM consumption
   - ✅ Kept legacy method for backward compatibility
   - ✅ Updated execute_step_with_rig_and_history to use new approach

3. **Context Selection Logic**:
   - ✅ Implemented MinimalAiContext with just wallet and relevant tokens
   - ✅ Added token filtering based on operation type (swap/lend)
   - ✅ Extracted key information from previous step results
   - ✅ Created clean prompt format for AI consumption

4. **Tests and Validation**:
   - ✅ Created comprehensive tests in crates/reev-core/tests/yml_context_builder_test.rs
   - ✅ Added tests for YML serialization/deserialization
   - ✅ Added tests for token filtering and prompt formatting
   - ✅ All 7 tests passing successfully

5. **Module Integration**:
   - ✅ Added context_builder to execution module exports
   - ✅ Added YML context builder types to lib.rs exports
   - ✅ Ensured proper module structure and imports

### Key Features:
- Structured YML context that can be parsed back to structs for validation
- Clean separation between minimal AI context and metadata
- Builder pattern for flexible context construction
- Support for previous step results and constraints
- Token filtering based on operation type
- Prompt format conversion for LLM consumption

### Expected Outcome Achieved:
- Clean, structured YML context for AI operations
- Parseable context for validation and debugging
- Reduced confusion for AI with concise, relevant information
- Consistent with PLAN_CORE_V3.md YML-based architecture
- Maintainable and extensible context format

---

## Task #122: Enhance Multi-Step Operation Context Passing

### Status: COMPLETED
### Priority: HIGH

### Status: COMPLETED
### Description:
Improve context passing between operations in multi-step flows to ensure proper wallet state updates, clear indication of changes, accurate constraints, and proper token balance tracking.

### Implementation Summary:
Successfully implemented enhanced multi-step context passing with the following components:

1. **Balance Change Tracking**:
   - ✅ Track token balance changes after each operation
   - ✅ Update wallet context based on actual execution results
   - ✅ Maintain accurate state throughout multi-step flows

2. **Enhanced Context Builder for Multi-Step**:
   - ✅ Extract key results from previous steps
   - ✅ Generate constraints based on previous step outputs
   - ✅ Create clear context for subsequent steps

3. **Error Recovery**:
   - ✅ Handle cases where previous steps don't provide expected outputs
   - ✅ Provide fallback options when constraints can't be satisfied
   - ✅ Add clear error messages for debugging

### Key Features:
- Balance change tracking with before/after amounts
- Constraint generation for next operations
- Error recovery with appropriate constraints
- Available tokens calculation based on previous results
- Clear indication of what changed in each step
- Proper token balance tracking throughout flow

### Tests:
- All 8 tests in multi_step_context_test.rs passing
- Tests cover balance change tracking, constraint generation, error recovery, and multi-step flows

### Expected Outcome Achieved:
- Accurate context passing between multi-step operations
- Reliable constraint generation based on previous results
- Better error recovery for multi-step flows
- Improved success rate for multi-step operations

---

## Task #123: Implement YML Context Validation Framework

### Status: NOT STARTED
### Priority: MEDIUM

### Description:
Create a validation framework for YML contexts to ensure:
1. All required fields are present
2. Values are within expected ranges
3. Constraints are consistent with wallet state
4. Context is properly formatted for AI consumption

### Tasks Required:
1. **Create YML Context Validator**:
   - Implement validation rules for YmlOperationContext
   - Add wallet state validation
   - Create constraint validation logic
   - Implement format validation for AI consumption

2. **Add Validation to Execution Flow**:
   - Validate context before sending to AI
   - Handle validation errors with recovery strategies
   - Log validation results for debugging

3. **Create Validation Tests**:
   - Unit tests for validation rules
   - Integration tests with execution flow
   - Error case testing for edge cases

### Expected Outcome:
- Robust validation for YML contexts
- Early detection of malformed contexts
- Improved reliability of AI operations
- Better error handling and recovery

---

## Implementation Order:

1. **Task #121** (Immediate): Implement structured YML context
2. **Task #122** (Next): Enhance multi-step context passing
3. **Task #123** (Final): Add validation framework

---

## Success Metrics:

1. **Clean YML Format**: All AI contexts use structured YML
2. **Parseability**: All contexts can be parsed back to structs
3. **Relevance**: AI only receives necessary information
4. **Validation**: All contexts pass validation checks
5. **Multi-step Success**: Multi-step operations have higher success rate