# Issues

## #1 - Enhanced OTEL to YML conversion not storing to database
**Status**: Fixed ✅
**Description**: The flow diagram API endpoint `/api/v1/flows/{session_id}` was not generating proper tool call diagrams because enhanced_otel data was not being converted to YML and stored in the database.

**Root Cause**: 
1. `JsonlToYmlConverter` was failing to match ToolInput with ToolOutput events
2. Runner wasn't storing converted YML to database after benchmark execution

**Solution Implemented**:
1. Fixed `JsonlToYmlConverter` to match tool inputs/outputs by sequence order
2. Enhanced runner to convert enhanced_otel JSONL to YML and store to database
3. Fixed database path resolution to use absolute paths

**Files Modified**:
- `crates/reev-flow/src/jsonl_converter/mod.rs` - Fixed tool input/output matching
- `crates/reev-runner/src/lib.rs` - Added YML conversion and database storage

**Testing Verified**:
- Flow diagrams now show tool calls with proper details
- Example: `Agent --> sol_transfer : 1 ix` with wallet addresses and amounts
