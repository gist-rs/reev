# Issues

## Critical Issues (V3 Plan Violations)

### Issue #1: Rule-Based Multi-Step Detection (Critical)
**Status**: NOT FIXED
**Location**: `crates/reev-core/src/yml_generator/unified_flow_builder.rs` (lines 45-52)
**Description**: The `UnifiedFlowBuilder` still uses rule-based parsing to detect multi-step operations, violating the V3 plan which requires LLM to handle all language understanding.
**Impact**: System incorrectly interprets user intent and doesn't properly understand natural language multi-step requests.

### Issue #2: Multi-Step Operations Not Properly Executed (Critical)
**Status**: NOT FIXED
**Location**: `crates/reev-core/src/execution/rig_agent/mod.rs`
**Description**: The `RigAgent` only processes single steps instead of iterating through multiple operations in a refined prompt.
**Impact**: Multi-step workflows are not executed as intended, breaking core functionality.

### Issue #3: E2E Tests Don't Validate Multi-Step Functionality (Critical)
**Status**: NOT FIXED
**Location**: `crates/reev-core/tests/e2e_multi_step.rs`
**Description**: Tests appear to pass but only execute one operation at a time. The `test_swap_then_lend` test uses pre-allocated tokens, creating an illusion of multi-step execution.
**Impact**: Test suite doesn't validate actual multi-step operations, masking fundamental issues.

### Issue #4: Language Understanding Still Rule-Based (Critical)
**Status**: NOT FIXED
**Location**: `crates/reev-core/src/yml_generator/mod.rs`
**Description**: The `determine_expected_tools` function uses string matching instead of LLM for language understanding.
**Impact**: Violates core architectural principle that LLM should handle all language understanding.

## Recent Fixes

### 2024-XX-XX: Fixed test_extract_tool_count and test_perform_consolidation_with_sessions tests
**Description**: Two tests were failing but marked as complete in previous iterations.

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