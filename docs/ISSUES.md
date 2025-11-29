# Reev Core Issues

## Current Status: LLM Integration & Multi-Step Operations

## 🚨 Critical Issue Requiring Immediate Attention

### Issue #1: Pubkey Parsing Failure in e2e_multi_step Test

**Status**: BLOCKING
**Location**: `crates/reev-core/src/execution/rig_agent/prompting.rs`
**Impact**: All e2e tests fail, blocking multi-step operation development

**Problem Summary**:
- e2e_multi_step test fails with "Failed to parse pubkey: String is the wrong size"
- Pubkey appears correct in logs: `3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr` (44 chars)
- Same pubkey parses correctly in isolated test (`test_pubkey_parsing.rs`)
- Issue is in the LLM parameter extraction or string processing pipeline

**Root Cause Analysis**:
1. LLM extracts pubkey correctly (44 chars, starts with number)
2. String appears correct when passed to JupiterSwapArgs
3. Direct parsing test passes with same pubkey
4. Issue likely in how string is processed between LLM extraction and tool execution

**Implementation Plan**:
1. Add hex representation debugging in JupiterSwapTool::call
   - Log hex bytes of pubkey string before parsing
   - Compare with hex bytes in test_pubkey_parsing.rs
   - Identify any invisible characters or encoding differences

2. Create robust string sanitization
   - Strip invisible characters (BOM, zero-width spaces, etc.)
   - Normalize Unicode (NFC)
   - Add validation for pubkey format (44 chars, base58)

3. Implement fallback parsing with multiple validation methods
   - Try direct Pubkey::from_str first
   - If fails, try sanitized version
   - As last resort, extract from wallet_context directly
   - Add explicit error messages for each failure mode

4. Add comprehensive logging for debugging
   - Log pubkey string at each step in pipeline
   - Log byte-by-byte comparison between working/failing cases
   - Create a dedicated debug test for this issue

**Files to Modify**:
- `crates/reev-tools/src/tools/jupiter_swap.rs` - Add hex debugging and sanitization
- `crates/reev-core/src/execution/rig_agent/tool_execution.rs` - Improve parameter handling
- `crates/reev-core/src/execution/rig_agent/prompting.rs` - Fix pubkey extraction

## Priority Issues

### Issue #2: GLM Response Parsing Issues

**Status**: IN PROGRESS
**Location**: `crates/reev-core/src/llm/glm_client.rs`

**Problem**: GLM returns `reasoning_content` with analysis instead of structured JSON

**Current Workarounds**:
- `extract_json_from_text` function parses JSON from mixed responses
- Fallback responses when JSON extraction fails
- Pattern-based operation extraction as backup

**Implementation Plan**:
1. Enhance JSON extraction with better error handling
   - Add detailed logging for extraction attempts
   - Track which extraction method succeeded
   - Improve regex patterns for edge cases
   - Add validation of extracted JSON structure

2. Consider alternative prompt engineering
   - Experiment with different prompt formats
   - Test more explicit instructions for JSON-only response
   - Try adding JSON schema in the prompt
   - Evaluate temperature settings for more deterministic output

3. Evaluate if GLM is optimal for structured output
   - Compare with other LLMs for JSON output reliability
   - Consider implementing a structured output wrapper
   - Evaluate using OpenAI's function calling or similar features
   - Test with different model versions for improvement

### Issue #3: Multi-Step Operations Architecture

**Status**: PARTIALLY IMPLEMENTED
**Location**: `crates/reev-core/src/yml_generator/mod.rs`

**Working Correctly**:
- Multi-step operations split into separate steps
- Pattern-based extraction handles common scenarios
- Tests pass for single-step operations

**V3 Compliance Issues**:
1. Operations split in YmlGenerator instead of LanguageRefiner
2. Action words not preserved in extracted operations
3. Not fully aligned with V3 architecture requirements

**Implementation Plan**:
1. Move multi-step detection to LanguageRefiner
   - Refactor YmlGenerator to only handle single operations
   - Move sequence detection logic to LanguageRefiner
   - Update Planner to use LanguageRefiner for multi-step
   - Add tests to validate the new flow

2. Preserve action words in operations
   - Modify operation extraction to keep action words
   - Update YML schema to include action context
   - Ensure action words are passed through to execution
   - Add validation that action words are preserved

3. Validate complete V3 compliance
   - Compare current implementation with PLAN_CORE_V3.md
   - Identify any remaining gaps in architecture
   - Add tests specifically for V3 compliance
   - Document migration strategy for remaining changes

## Test Status Summary

**Passing**:
- `test_llm_operation_extraction.rs`: 6/6 tests
- `enhanced_context_test.rs`: 8/8 tests
- `yml_context_builder_test.rs`: 7/7 tests
- `multi_step_context_test.rs`: 8/8 tests
- `test_pubkey_parsing.rs`: 1/1 tests (isolated parsing works)

**Failing**:
- `e2e_multi_step.rs`: Pubkey extraction issue in LLM pipeline

## Implementation Priority

1. **URGENT (Blocker)**: Fix pubkey extraction & parsing (Issue #1)
   - Must be resolved before any e2e tests can pass
   - Blocking all multi-step operation development
   - Estimated effort: 2-3 days
   - **Dependencies**: None

2. **High**: Improve GLM response parsing (Issue #2)
   - Enhances reliability of LLM integration
   - Critical for production stability
   - Estimated effort: 1-2 days
   - **Dependencies**: None

3. **Medium**: Align multi-step with V3 architecture (Issue #3)
   - Long-term architecture improvement
   - Can be addressed after blockers resolved
   - Estimated effort: 3-5 days
   - **Dependencies**: Issue #1, #2

## Immediate Next Steps

1. Today: Implement hex debugging in JupiterSwapTool::call to identify root cause
2. Tomorrow: Implement string sanitization and fallback parsing mechanisms
3. Follow-up: Address GLM response parsing improvements
4. Later: Align with V3 architecture once core issues resolved

## Critical Files for Fixes

1. `crates/reev-tools/src/tools/jupiter_swap.rs` - Pubkey parsing improvements
   - Add hex debugging for pubkey bytes
   - Implement string sanitization
   - Add robust fallback parsing

2. `crates/reev-core/src/execution/rig_agent/tool_execution.rs` - Parameter processing
   - Improve parameter type handling
   - Add validation for critical parameters
   - Fix numeric value conversion

3. `crates/reev-core/src/execution/rig_agent/prompting.rs` - System prompt improvements
   - Fix pubkey extraction from wallet context
   - Improve tool parameter parsing
   - Add robust error handling

4. `crates/reev-core/src/llm/glm_client.rs` - JSON extraction enhancements
   - Improve `extract_json_from_text` function
   - Add better fallback mechanisms
   - Enhance error reporting

5. `crates/reev-core/src/yml_generator/mod.rs` - Architecture alignment
   - Move multi-step detection to LanguageRefiner
   - Preserve action words in operations
   - Align with V3 architecture