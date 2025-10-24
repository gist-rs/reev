# TASKS.md

## Session ID Unification - Critical Architecture Fix (IN PROGRESS)

### 🚨 Current Problem Analysis
**Session ID Chaos**: Multiple different IDs created across components:
- Runner session_id: `f0133fcd-37bc-48b7-b24b-02cabed2e6e9` (database tracking)
- Flow logger session_id: `791450d6-eab3-4f63-a922-fec89a554ba8` (created independently)
- Agent otel session_ids: `7229967a-8bb6-4003-ac1e-134f4c71876a.json`, `23105291-893d-4a58-9622-e02d41a6649f.json` (multiple created)

**Root Cause**:
- `LlmRequest.id` contains benchmark_id (`"001-sol-transfer"`), not session_id
- Each component creates its own UUID instead of using runner's session_id
- No single source of truth for session ID propagation

### Phase 1: Fix LlmRequest Structure ✅ COMPLETED
**File**: `crates/reev-agent/src/lib.rs`
- [x] Add `session_id` field to `LlmRequest` struct
- [x] Keep existing `id` field for benchmark_id
- [x] Update all LlmRequest creations in tests and examples

### Phase 2: Fix Runner Payload Population ✅ COMPLETED
**File**: `crates/reev-lib/src/llm_agent.rs` and `crates/reev-runner/src/lib.rs`
- [x] Add `session_id` field to `LlmAgent` struct with setter method
- [x] Populate `session_id` with runner's generated UUID before agent call
- [x] Update GLM payload to include session_id when available
- [x] Keep `id` as benchmark_id ("001-sol-transfer")

### Phase 3: Fix Component Initialization ✅ COMPLETED
**Files**: 
- `crates/reev-flow/src/enhanced_otel.rs` (enhanced otel logger)
- `crates/reev-flow/src/otel.rs` (flow tracing)
- `crates/reev-flow/src/logger.rs` (flow logger)
- `crates/reev-agent/src/run.rs` (agent otel)
- `crates/reev-lib/src/llm_agent.rs` (llm agent)
- [x] Enhanced Otel: Add `with_session_id()` method and `init_enhanced_otel_logging_with_session()`
- [x] Flow Tracing: Add `init_flow_tracing_with_session()` function
- [x] Flow Logger: Add `new_with_session()` method
- [x] Agent otel: Initialize with `payload.session_id`
- [x] LlmAgent: Use session_id for flow tracing and payload
- [x] Remove independent UUID generation in all components

### Phase 4: Update File Naming and Extraction ✅ COMPLETED
**File**: `crates/reev-runner/src/lib.rs`
- [x] Update `extract_tool_calls_from_agent_logs()` to use specific session_id: `otel_{session_id}.json`
- [x] Remove fallback "scan all files" logic
- [x] Add session_id verification when parsing tool calls
- [x] Update logging to show specific file being processed
- [x] Ensure single otel file per session

### Expected Results
- Single session_id (`f0133fcd...`) from start to finish
- Single otel file: `otel_f0133fcd-37bc-48b7-b24b-02cabed2e6e9.json`
- Runner finds and processes its specific otel file
- Clear separation: benchmark_id for identification, session_id for tracing

### Implementation Status: 🔄 TESTING IN PROGRESS - ISSUES FOUND
- ✅ Session ID unified across all components
- ✅ No more multiple UUID generation in logic
- ❌ **ISSUE**: Multiple otel files still being created (session_id unification not working end-to-end)
- ❌ **ISSUE**: reev-agent still generates random session_id before runner's session_id arrives

### Implementation Priority
1. ✅ LlmRequest struct update (foundation)
2. ✅ Runner payload population (data flow)
3. ✅ Component initialization fixes (ID consistency)
4. ✅ File naming/extraction updates (final integration)

### Current Issues Discovered
- reev-agent starts and immediately calls `init_enhanced_otel_logging()` with new UUID
- Later calls to `init_enhanced_otel_logging_with_session()` fail because global logger already set
- Result: Multiple otel files created, session_id unification broken

### Next Steps: Fix Global Logger Initialization
- Remove early otel initialization from reev-agent startup
- Ensure `init_enhanced_otel_logging_with_session()` is called first with runner's session_id
- Test end-to-end single session_id flow
- Verify only one otel file per benchmark run
