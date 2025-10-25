# TOFIX.md

## Remaining Issues

### ✅ RESOLVED: Session ID Unification - Fixed

**Previous Problem**: 
Session ID was missing from LLM API requests when `agent_name="local"`, causing 422 Unprocessable Entity errors due to missing `session_id` field.

**Root Cause**: 
The "default reev API format" payload in `LlmAgent::get_action()` did not include the `session_id` field, even when it was available on the agent instance.

**Solution Applied**:
- Modified `crates/reev-lib/src/llm_agent.rs` to add `session_id` to default payload format
- Added proper logging to confirm session_id inclusion
- Both GLM and default API routes now include session_id when available

**Verification**:
- ✅ Session ID now included in LLM payloads: `"session_id": "1fdc9e9f-5688-4ab5-8dad-2bfc22de58c3"`
- ✅ LLM API requests succeed without 422 errors
- ✅ Tool calls logged with correct session_id in otel files
- ✅ Single session_id flow achieved: runner → agent → otel → runner extraction

**Architecture Status**: 
- Session ID unification is **COMPLETE**
- Single session_id flows through entire system
- Clean separation: `id` for benchmark_id, `session_id` for tracing
- Ready for production use

## Minor Cleanup Items

### 📝 FlowAgent Session ID Enhancement
- Added `new_with_session()` method to FlowAgent for consistency
- Maintains backward compatibility with existing `new()` method
- Not currently used in runner flow but available for future use

### 📝 Empty Otel Files 
- Runner's early tracing initialization creates empty otel file
- Does not affect functionality - actual tool calls go to session-specific file
- Consider minor improvement in future to avoid empty file creation