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

### Status: NOT STARTED
### Priority: HIGH

### Description:
Replace the current mixed JSON+markdown context generation in RigAgent with structured YML context that:
1. Uses existing YML schema from yml_schema.rs
2. Is parseable back to structs for validation
3. Sends only relevant information to AI
4. Maintains clean, consistent format

### Tasks Required:
1. **Create YML Context Builder** (crates/reev-core/src/execution/context_builder.rs):
   - Implement YmlOperationContext struct for AI operations
   - Create builder methods for constructing context from wallet info and previous results
   - Add serialization/deserialization methods

2. **Update RigAgent Integration** (crates/reev-core/src/execution/rig_agent/mod.rs):
   - Replace create_context_prompt_with_history with YML-based approach
   - Use YmlContextBuilder to create structured context
   - Select only relevant parts for AI (not full context)
   - Send clean YML to AI instead of mixed JSON+markdown

3. **Implement Context Selection Logic**:
   - Create MinimalAiContext struct with just what AI needs
   - Implement selection logic to extract relevant parts
   - Ensure YML is clean and concise for AI consumption

4. **Update Tests**:
   - Create tests for YmlContextBuilder
   - Update e2e tests to verify YML context
   - Add validation tests for YML parsing

5. **Add Validation**:
   - Implement validation for generated YML contexts
   - Add error handling for malformed YML
   - Ensure backward compatibility with existing flows

### Expected Outcome:
- Clean, structured YML context for AI operations
- Parseable context for validation and debugging
- Reduced confusion for AI with concise, relevant information
- Consistent with PLAN_CORE_V3.md YML-based architecture
- Maintainable and extensible context format

---

## Task #122: Enhance Multi-Step Operation Context Passing

### Status: NOT STARTED
### Priority: HIGH

### Description:
Improve context passing between operations in multi-step flows to ensure:
1. Proper wallet state updates after each step
2. Clear indication of what changed in each step
3. Accurate constraints based on previous step results
4. Proper token balance tracking throughout flow

### Tasks Required:
1. **Implement Balance Change Tracking**:
   - Track token balance changes after each operation
   - Update wallet context based on actual execution results
   - Maintain accurate state throughout multi-step flows

2. **Enhance Context Builder for Multi-Step**:
   - Extract key results from previous steps
   - Generate constraints based on previous step outputs
   - Create clear context for subsequent steps

3. **Improve Error Recovery**:
   - Handle cases where previous steps don't provide expected outputs
   - Provide fallback options when constraints can't be satisfied
   - Add clear error messages for debugging

### Expected Outcome:
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