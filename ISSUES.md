# Issues

## #1 - Enhanced OTEL to YML conversion not storing to database
**Status**: Partially Fixed ⚠️
**Description**: The flow diagram API endpoint `/api/v1/flows/{session_id}` was not generating proper tool call diagrams because enhanced_otel data was not being converted to YML and stored in database.

**Root Cause**: 
1. `JsonlToYmlConverter` exists but runner wasn't calling it automatically
2. Enhanced_otel JSONL files exist but are not converted to YML format
3. Session parser expects JSON format, not YAML

**Solution Implemented**:
1. Added `JsonlToYmlConverter` logic to `benchmark_executor.rs` for automatic conversion
2. Enhanced database writer trait with `store_session_log` and `store_tool_call` methods
3. Fixed tool call data structure mapping for database storage

**Files Modified**:
- `crates/reev-api/src/services/benchmark_executor.rs` - Added automatic YML conversion after runner completion
- `crates/reev-db/src/writer/mod.rs` - Added session log and tool call storage methods
- `crates/reev-db/src/pool/pooled_writer.rs` - Implemented trait methods for pooled writer

**Current Status**:
- ✅ Code compiles without errors
- ✅ Enhanced_otel to YML conversion logic implemented
- ❌ Flow diagrams still show generic results (no tool details)
- ❌ Manual testing shows YML data exists but parser doesn't read YAML format

**Testing Results**:
- Manual YML insertion to database shows `tool_count: 0` in flow diagram
- Session parser expects JSON structure, not YAML format
- Need to modify session parser to handle YAML or store as JSON format

## #2 - Session parser doesn't support YAML format from enhanced_otel conversion
**Status**: New Issue ❌
**Description**: The flow diagram API expects tool call data in JSON format, but enhanced_otel conversion produces YAML format that the session parser cannot read.

**Root Cause**:
1. `SessionParser::parse_session_content()` expects JSON structure with `tools` array
2. `JsonlToYmlConverter` produces YAML format for human readability
3. Database stores YAML but parser only understands JSON format

**Files Affected**:
- `crates/reev-api/src/handlers/flow_diagram/session_parser.rs` - Only handles JSON format

**Expected Flow**:
`run bench -> api -> agent -> runner -> otel -> enhanced_otel_{session_id}.jsonl -> api (yml) -> db -> web <- api <- mermaid <- yml <- db`

**Current Block**:
`enhanced_otel_{session_id}.jsonl -> api (yml) -> db ✅`
`web <- api <- mermaid <- yml <- db ❌` (parser doesn't read YAML)

**Next Steps**:
1. Convert enhanced_otel to JSON format instead of YAML for database storage
2. Or modify session parser to handle YAML format
3. Test flow diagram generation with actual tool call data

## #3 - Syntax errors in flows.rs compilation issues
**Status**: Fixed ✅
**Description**: Build errors in `crates/reev-api/src/handlers/flows.rs` due to unclosed delimiters and mismatched braces after implementing enhanced_otel conversion.

**Root Cause**:
1. Multiple closing braces causing "unexpected closing delimiter" errors
2. Complex nested match statements creating syntax issues
3. File structure corrupted during edits

**Solution Implemented**:
1. Reverted to clean state using `git checkout`
2. Restored proper function structure
3. Kept benchmark executor changes for enhanced_otel conversion

**Files Affected**:
- `crates/reev-api/src/handlers/flows.rs` - Restored to working state

**Testing Verified**:
- API compiles successfully without errors
- Flow diagram endpoints are accessible
- Ready for enhanced_otel conversion testing

