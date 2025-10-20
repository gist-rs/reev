# TOFIX.md - Current Issues to Fix

## Priority: MEDIUM - Tool Call Extraction Pipeline Works, Need LLM Response Analysis

## ✅ MAJOR BREAKTHROUGH - Infrastructure Working

### Issue RESOLVED: Infrastructure 
- ✅ Session logging infrastructure confirmed working
- ✅ Tool calls successfully appear in session logs  
- ✅ Flow visualization now shows tool_count correctly
- ✅ Flow diagram generates with proper tool node styling

### Confirmed Working Pipeline
```
Benchmark → LlmAgent.get_action() → Tool Extraction → Session Logger → Flow API ✅
```

### Current Status
- ✅ Unit tests all passing (4/4)  
- ✅ Tool call extraction methods implemented in LlmAgent
- ✅ Session logging and flow visualization working perfectly
- ❌ LLM response text not containing expected keywords for extraction

### Remaining Work Needed
1. **Analyze GLM Response Format**: Examine actual response text from GLM-4.6 agent
2. **Update Detection Logic**: Adapt keyword detection to match real response patterns
3. **Remove Fallback**: Clean up temporary testing code in runner

### Test Results
- ✅ Flow API now shows `"tool_count": 1` with test fallback
- ✅ Flow diagram displays: `Agent --> testtoola3fc75c4 : benchmark_id = "001-sol-transfer"...`
- ✅ Tool nodes styled with green color in diagram

### Current Issue
The `LlmAgent.extract_tool_calls_from_response()` method is being called but the GLM response text doesn't contain the expected keywords (swap, transfer, balance, etc.). Need to analyze actual GLM response format to update detection logic.

### Files Working Correctly
- `crates/reev-lib/src/llm_agent.rs` - Extraction infrastructure ✅
- `crates/reev-runner/src/lib.rs` - Tool call integration ✅  
- `crates/reev-api/src/handlers/flows.rs` - Flow visualization ✅
- `logs/sessions/*.json` - Session logging ✅

### Files Involved
- `crates/reev-lib/src/llm_agent.rs` - Extraction implementation
- `crates/reev-runner/src/lib.rs` - Benchmark execution pipeline
- `logs/sessions/*.json` - Session logs for verification