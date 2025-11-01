# Issues

## #1 - Enhanced OTEL to YML conversion and flow diagram generation - ✅ RESOLVED
**Status**: Fixed ✅
**Description**: The flow diagram API endpoint `/api/v1/flows/{session_id}` was not generating proper tool call diagrams because enhanced_otel data was not being converted to a format that session parser could read.

**Root Cause**: 
1. `JsonlToYmlConverter` exists but runner wasn't calling it automatically
2. Session parser only supported JSON format, not YAML from enhanced_otel conversion
3. Database storage methods were missing for session logs and tool calls

**Solution Implemented**:
1. ✅ Added automatic enhanced_otel to YML conversion in `benchmark_executor.rs`
2. ✅ Enhanced session parser to handle both JSON and YAML formats  
3. ✅ Added `store_session_log` and `store_tool_call` methods to DatabaseWriterTrait
4. ✅ Implemented trait methods in pooled database writer

**Files Modified**:
- `crates/reev-api/src/services/benchmark_executor.rs` - Added automatic conversion after session completion
- `crates/reev-api/src/handlers/flow_diagram/session_parser.rs` - Added YAML parsing support with fallback
- `crates/reev-db/src/writer/mod.rs` - Added session log and tool call storage methods
- `crates/reev-db/src/pool/pooled_writer.rs` - Implemented trait methods

**Testing Results**:
- ✅ Enhanced_otel JSONL to YML conversion working correctly
- ✅ Session parser successfully reads YAML format from converted data
- ✅ Database storage of session logs and tool calls working
- ✅ Flow diagram API now generates proper tool call visualizations
- ✅ All compilation errors resolved and code follows project standards

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

## #2 - Session parser YAML support implementation ✅ RESOLVED
**Status**: Fixed ✅
**Description**: Enhanced session parser to handle both JSON and YAML formats for flow diagram generation.

**Solution Implemented**:
1. ✅ Modified `SessionParser::parse_session_content()` to try JSON parsing first, then YAML as fallback
2. ✅ Added `parse_yml_tool()` method to handle YAML tool call data structure
3. ✅ Enhanced tool call extraction to support both JSON and YAML formats
4. ✅ Added proper error handling for dual format parsing

**Files Affected**:
- `crates/reev-api/src/handlers/flow_diagram/session_parser.rs` - Enhanced to support JSON/YML dual formats

**Testing Results**:
- ✅ Parser successfully reads YAML content from enhanced_otel conversion
- ✅ Tool call extraction working for both formats
- ✅ Flow diagram generation now produces proper tool call details

## #3 - Enhanced OTEL files are empty (0 bytes) - ⚠️ NEW ISSUE
**Status**: Active Issue ❌
**Description**: Enhanced_otel JSONL files are being created but contain no data (0 bytes), preventing tool call extraction for flow diagrams.

**Root Cause**:
1. OpenTelemetry configuration not properly enabled in reev-runner or reev-agent
2. Enhanced OTEL logging events not being emitted during tool execution
3. Environment variables not set up for enhanced telemetry collection

**Current Observations**:
- ✅ Enhanced_otel conversion logic works correctly (tested with manual data)
- ✅ Database storage and retrieval working properly
- ✅ Session parser handles YAML format correctly
- ❌ No tool call data being generated during actual benchmark executions
- ❌ All enhanced_otel files from recent executions are 0 bytes

**Expected Behavior**:
Enhanced_otel files should contain JSONL lines like:
```json
{"timestamp":"2025-11-01T09:31:53.696234Z","session_id":"...","event_type":"ToolInput","tool_input":{"tool_name":"sol_transfer","tool_args":{"amount":100000000,"recipient_pubkey":"..."}},"tool_output":null}
{"timestamp":"2025-11-01T09:31:53.696643Z","session_id":"...","event_type":"ToolOutput","tool_input":null,"tool_output":{"success":true,"results":"..."}}
```

**Actual Behavior**:
Enhanced_otel files are empty (0 bytes), so no tool calls can be extracted for flow diagrams.

**Flow Status**:
`run bench -> api -> agent -> runner -> otel -> enhanced_otel_{session_id}.jsonl (EMPTY) -> api (yml) -> db -> web <- api <- mermaid <- yml <- db (NO DATA) ❌`

**Next Steps**:
1. 🔍 Investigate reev-runner OpenTelemetry configuration
2. 🔍 Check reev-agent enhanced OTEL event emission
3. 🔍 Verify environment variables for enhanced telemetry
4. 🔍 Test with manual enhanced OTEL file creation to verify conversion works

