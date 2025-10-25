## Architecture Analysis

### Current Flow:
```
FlowAgent (orchestrates multi-step flows)
    ↓ calls
run_agent (dispatches to model-specific agents)
    ↓ calls
ZAIAgent/OpenAIAgent (creates tools with resolved key_map)
```

### 🚨 Critical Issue: Ground Truth Data Leakage

**Problem**: FlowAgent passes `benchmark.ground_truth` into `resolve_initial_context()`, breaking real-time multi-step decision making.

**Solution Implemented**: Clean ground truth separation with mode detection
```rust
// In FlowAgent - Proper ground truth separation ✅ FIXED
let ground_truth_for_context =
    if is_deterministic_mode(&self.model_name, &benchmark.id, &benchmark.tags) {
        info!("[FlowAgent] Using ground truth for deterministic mode");
        Some(&benchmark.ground_truth)
    } else {
        info!("[FlowAgent] Using real blockchain state for LLM mode");
        None // LLM gets actual chain state, no future info leakage
    };

// Validate no ground truth leakage in LLM mode
if !is_deterministic_mode(&self.model_name, &benchmark.id, &benchmark.tags)
    && !benchmark.ground_truth.final_state_assertions.is_empty() {
    return Err(anyhow!(
        "Ground truth not allowed in LLM mode - would leak future information"
    ));
}
```

### 🎯 Solution: Ground Truth Separation ✅ IMPLEMENTED

**Option A: Clean Separation** ✅ IMPLEMENTED
- Test files: Use ground_truth for fast validation and scoring
- Production agents: Use real blockchain state only
- Clear architectural boundary between test data and execution data

**Option B: Conditional Ground Truth** ✅ IMPLEMENTED
- Deterministic mode: Use ground_truth for reproducible tests
- LLM mode: Use blockchain state for real evaluation

Status**: 🟢 COMPLETED - Ground truth leakage eliminated, all compilation errors fixed

## Sol Transfer Tool Call Consolidation - Database Deduplication

### 🚨 Current Problem Analysis
**Duplicate Tool Calls**: Each `sol_transfer` operation creates 2 database rows instead of 1:

```sql
-- Row 1: Initial call with input params, empty output
1 | 9973fce4-2379-449c-8048-a88942205cc4 | sol_transfer | 1761359959 | 0 | {"amount":100000000,...} | {} | success | | {} | 1761359965

-- Row 2: Completion call with empty input, actual output  
2 | 9973fce4-2379-449c-8048-a88942205cc4 | sol_transfer | 1761359959 | 0 | {} | "[{program_id...}]" | success | | {} | 1761359965
```

**Root Cause**:
- `log_tool_call!` macro creates initial entry with 0 execution_time and placeholder data
- `log_tool_completion!` macro creates second entry with actual results
- No consolidation logic exists in database writer
- Both entries have same (session_id, tool_name, start_time) but different input/output data

### Solution Strategy: Smart Consolidation

**Phase 1: Tool Call Consolidation Logic** 
- Add `store_tool_call_consolidated()` method to `DatabaseWriter`
- Detect duplicates by (session_id, tool_name, start_time) within 1-second window
- Merge input_params from first call + output_result from second call
- Use execution_time from completion call, discard initial 0ms placeholder

**Phase 2: Enhanced Tool Call Tracking**
- Modify `log_tool_call!` to mark as "in_progress" status
- Modify `log_tool_completion!` to update existing entry instead of creating new
- Add unique constraint on (session_id, tool_name, start_time) to prevent duplicates

**Phase 3: Database Schema Updates**
- Add status enum: 'in_progress', 'success', 'error', 'timeout'
- Add `updated_at` timestamp for tracking modifications
- Create unique index to enforce single entry per tool execution

**Implementation Location**:
- `crates/reev-db/src/writer/sessions.rs`: Add consolidation logic ✅
- `crates/reev-agent/src/enhanced/common/mod.rs`: Fix macro behavior ✅  
- `crates/reev-db/.schema/current_schema.sql`: Update schema constraints ✅

**Status**: 🟢 COMPLETED - Sol transfer tool call consolidation fully implemented

### Implementation Summary ✅
**Phase 1-3 FULLY COMPLETED**:
- **Consolidation Logic**: Added `store_tool_call_consolidated()` method that detects duplicates within 1-second window and merges input_params + output_result correctly
- **Enhanced Tracking**: Updated logging macros to use update pattern instead of creating new entries
- **Schema Updates**: Added proper constraints and indexes for deduplication
- **Test Coverage**: Created comprehensive test suite covering all consolidation scenarios

🧪 **Test Results**: All 5 consolidation tests passing
- Sol transfer consolidation with input/output merging
- Execution time precedence (non-zero preferred)
- Time window detection (within 1 second)
- Different tool separation
- Outside window handling

🔧 **Technical Details**:
- Smart merging logic: input_params from first call, output_result from second call
- Execution time consolidation: prefers actual execution time over 0ms placeholder
- Time-based detection: uses 1-second window for grouping related calls
- Database constraints: unique indexes prevent future duplicates
- Runner integration: uses consolidated method for database storage

**Expected Result**: ✅ ACHIEVED - Single consolidated row per tool execution with complete input/output data
