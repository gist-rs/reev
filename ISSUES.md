# Issues

## Current Status: LLM-Based Operation Extraction Implementation

### Issue #1: LLM Integration In Progress
**Status**: IN PROGRESS
**Location**: `crates/reev-core/src/yml_generator/mod.rs` (extract_operations_from_prompt and determine_expected_tools functions)
**Description**: Currently implementing LLM-based operation extraction and tool determination to replace rule-based approaches. GLM client is being properly initialized in planner and passed to YmlGenerator.

**Current Implementation**:
1. YmlGenerator now accepts an LLM client in constructor
2. `extract_operations_from_prompt` uses LLM to identify multiple operations in a prompt
3. `determine_expected_tools` uses LLM to determine required tools
4. Enhanced JSON extraction from GLM responses with fallback handling

**Current Issues**:
1. GLM model is returning analysis text instead of structured JSON
2. JSON parsing is not working correctly for operation extraction
3. Test still shows only 1 step instead of multiple steps

### Issue #2: LLM Response Parsing
**Status**: BEING DEBUGGED
**Location**: `crates/reev-core/src/llm/glm_client.rs`
**Description**: GLM model returns reasoning_content with analysis instead of structured JSON response. Working on extracting JSON from mixed text responses.

### Issue #3: Test Validation
**Status**: WAITING FOR FIXES
**Location**: `crates/reev-core/tests/e2e_multi_step.rs`
**Description**: Once LLM integration is fixed, test should show multiple steps being generated and executed.

## Recent Fixes

### 2024-XX-XX: LLM Integration Started
**Description**: Started implementing LLM-based operation extraction and tool determination. YmlGenerator now properly initialized with LLM client from planner.

**Issues Fixed**:
1. `test_extract_tool_count` was failing because the function was incorrectly counting tool names in JSON content. The test expected 0 for JSON format without "tool_name:" prefix, but was returning 2.

2. `test_perform_consolidation_with_sessions` was failing because the test expected a text-based consolidated session format, but the implementation now creates a JSON structure. Also, the `analyze_session_success` function wasn't correctly detecting failed sessions with "success: false".

**Root Causes**:
1. `extract_tool_count` was counting both quoted and unquoted "tool_name" patterns, and the fallback logic for non-JSON content was being applied to JSON content.

2. `analyze_session_success` was using a fallback condition that didn't check for explicit "success: false" values.

3. The test was expecting a different format for the consolidated session.

**Fixes Applied**:
1. Modified `extract_tool_count` to:
   - Only count unquoted "tool_name:" patterns in JSON content
   - Use fallback logic only for non-JSON content without explicit tool_name patterns

2. Modified `analyze_session_success` to:
   - Check for explicit "success: false" and "\"success\":false" before applying other rules

3. Updated `test_perform_consolidation_with_sessions` to:
   - Expect JSON structure instead of text-based format
   - Verify the JSON contains the expected fields and values

**Tests Status**: All tests now pass

## Recommendations for V3 Compliance

1. **Remove Rule-Based Detection**: Eliminate all string matching and pattern detection from the `UnifiedFlowBuilder` and replace with LLM-based analysis.

2. **Implement True Multi-Step Execution**: Modify `RigAgent` to parse and execute multiple operations from a single refined prompt, maintaining state between operations.

3. **Rewrite E2E Tests**: Create tests that validate actual multi-step execution without pre-allocation tricks, ensuring operations are properly chained.

4. **Implement LLM-First Architecture**: Ensure all language understanding is handled by LLM components, eliminating rule-based approaches throughout the system.

5. **Add Proper Integration Tests**: Create comprehensive tests that validate the entire multi-step workflow from natural language input to execution results.