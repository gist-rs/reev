# Plan to Implement Structured LLM Response System

## Executive Summary

This document outlines a comprehensive plan to implement a structured LLM response system that replaces rule-based parsing with direct parameter extraction from LLM responses. This implementation aligns with the V3 architecture and will solve critical issues with operation parsing, particularly for the "all" keyword handling.

## Problem Analysis

### Current State
- The system processes transfer requests through `QueryHandler` → `Planner` → `PromptProcessor` → `YmlGenerator`
- The `PromptProcessor` uses unstructured text responses from LLM, requiring rule-based parsing
- The `YmlGenerator` creates YML flows but lacks proper structure for direct parameter access
- Rule-based parsing is brittle and fails with slight variations in prompts
- Inconsistent parameter extraction across different operation types

### Key Issues Identified
1. **Brittle parsing**: Current system relies on regex matching that breaks with minor variations
2. **Double gas deduction**: Jupiter swap tool deducts gas reserve twice (once in prompt processor, again when checking for "all" keyword)
3. **Poor extensibility**: Adding new operation types requires updating multiple parsing rules
4. **Inconsistent validation**: Different operations have different parameter extraction logic

## Solution Overview

Implement a structured LLM response system where the LLM returns JSON with:
1. Refined prompt text
2. Detected action type
3. Extracted addresses and parameters
4. Confidence metrics

This approach eliminates the need for rule-based parsing and provides a more reliable, extensible foundation.

## Phase 1: Define Structured Data Types

### Purpose
Create the foundation for structured data handling that will carry extracted parameters through the pipeline.

### Tasks
1. Define `StructuredRefinedPrompt` struct with fields:
   - `refined_prompt`: String (clearer version of original prompt)
   - `action`: `PromptAction` enum (Transfer, Swap, Lend, Earn, Borrow, Unknown)
   - `subject_pubkey`: Option<String> (wallet performing action)
   - `target_pubkey`: Option<String> (destination address)
   - `parameters`: `PromptParameters` struct
   - `confidence`: f32 (LLM confidence in extraction)
   - `usable_amount`: Option<f64> (calculated amount for "all" keyword)

2. Define `PromptParameters` struct with:
   - `amount`: Option<String> (amount to transfer/swap/lend)
   - `input_mint`: Option<String> (input token mint address)
   - `output_mint`: Option<String> (output token mint address)
   - Additional flexible parameters via `HashMap<String, serde_json::Value>`

### Implementation Location
- Primary: `crates/reev-core/src/prompt_processor/types.rs`

## Phase 2: Update Prompt Processing

### Purpose
Modify LLM interaction to return structured data directly, eliminating the need for rule-based parsing.

### Tasks
1. Create structured system prompt instructing LLM to respond with JSON
2. Update `process_prompt_structured` to return `StructuredRefinedPrompt`
3. Implement proper "all" keyword handling:
   - Calculate max transferable amount based on operation type
   - Include this amount in both refined prompt and parameters
   - Set `usable_amount` field for validation

### Implementation Location
- Primary: `crates/reev-core/src/prompt_processor/mod.rs`

## Phase 3: Update Execution Flow

### Purpose
Integrate structured data into execution pipeline to directly use extracted parameters without parsing.

### Tasks
1. Modify `execute_step_with_rig_and_history` to use structured fields when available
2. Remove rule-based extraction from `extract_tool_calls` when structured data exists
3. Add `create_tool_calls_from_structured_data` method to create tool calls directly
4. Update `YmlStep` to include `structured_prompt` field for data passing

### Implementation Location
- Primary: `crates/reev-core/src/execution/rig_agent/mod.rs`
- Supporting: `crates/reev-core/src/yml_schema.rs`

## Phase 4: Update YML Generation

### Purpose
Ensure YML generator can create flows based on structured data and handle all operation types.

### Tasks
1. Update `generate_flow_from_structured_prompt` to handle all action types:
   - Transfer: Create appropriate tool calls with recipient and amount
   - Swap: Create tool calls with input/output mints and amount
   - Lend: Create tool calls with mint and amount
   - Earn: Create tool calls with mint only
   - Borrow: Create tool calls with mint and amount

2. Set `structured_prompt` field in `YmlStep` to preserve structured data

### Implementation Location
- Primary: `crates/reev-core/src/yml_generator/mod.rs`

## Phase 5: Validation and Testing

### Purpose
Ensure the structured response system is reliable and doesn't introduce regressions.

### Tasks
1. Update tests to handle cases where wallet balance is insufficient
2. Create tests for each action type with structured data
3. Verify "all" keyword handling works correctly for all operation types
4. Ensure tool calls are created correctly from structured parameters

### Implementation Location
- Primary: `crates/reev-core/tests/prompt_processor_tests.rs`

## Success Criteria

1. ✅ Elimination of double gas reserve deduction for swaps
2. ✅ Improved operation extraction accuracy from ~85% to >95%
3. ✅ Enhanced extensibility for new operation types
4. ✅ Reduced execution time by eliminating regex parsing
5. ✅ Comprehensive test coverage (>90%)

## Timeline

- Phase 1-2: 2-3 days
- Phase 3-4: 3-4 days
- Phase 5: 2-3 days
- Total: 7-10 days

## Dependencies

- Access to LLM API for testing
- Production environment for monitoring setup
- Completion of current rig_agent refactoring

## Next Steps

1. Implement validation logic for structured responses
2. Add comprehensive test coverage
3. Implement monitoring and metrics
4. Performance testing and optimization
5. Documentation updates

## Conclusion

The structured LLM response system provides a more reliable, extensible foundation for operation handling. By having the LLM directly extract parameters in a structured format, we eliminate brittle rule-based parsing and ensure consistent behavior across all operation types. This approach aligns with the V3 architecture and provides a solid foundation for future enhancements.