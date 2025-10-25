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

### ✅ Remove Metadata Fields - COMPLETED
**Problem**: Multiple metadata fields exist throughout codebase that need removal:
- Database schema: `session_tool_calls.metadata` column
- Struct definitions: `LogEvent`, `TestResult`, `FlowBenchmark`, `StepResult`, `EventContent`, `SessionLog`
- No actual database files found - can delete without migration

**Solution Applied**:
1. ✅ Removed metadata column from database schema files
2. ✅ Removed metadata fields from all struct definitions
3. ✅ Removed metadata-related code usage (serialization, assignments, logging)
4. ✅ Updated all function calls and test files
5. ✅ Fixed compilation errors and borrow checker issues

**Files Modified**:
- `crates/reev-db/.schema/current_schema.sql`
- `crates/reev-db/.schema/003_add_tool_calls_table.sql`
- `crates/reev-db/src/types.rs`
- `crates/reev-db/src/shared/benchmark.rs`
- `crates/reev-agent/src/flow/state.rs`
- `crates/reev-agent/src/flow/benchmark.rs`
- `crates/reev-agent/src/flow/agent.rs`
- `crates/reev-agent/src/flow/secure/executor.rs`
- `crates/reev-agent/examples/200-jup-swap-then-lend-deposit.rs`
- `crates/reev-flow/src/types.rs`
- `crates/reev-flow/src/utils.rs`
- `crates/reev-flow/src/logger.rs`
- `crates/reev-flow/src/database.rs`
- `crates/reev-lib/src/session_logger/mod.rs`
- `crates/reev-db/src/writer/sessions.rs`
- `crates/reev-db/src/reader.rs`
- `crates/reev-runner/src/lib.rs`
- Multiple test files in `crates/reev-flow/tests/`

**Status**: All metadata fields removed, project compiles successfully