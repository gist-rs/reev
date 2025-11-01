# Issues

## #1 - Session parser YAML support for enhanced_otel flow diagrams ✅ RESOLVED
**Status**: Fixed ✅
**Description**: The flow diagram API endpoint `/api/v1/flows/{session_id}` was generating generic diagrams instead of tool-specific visualizations because session parser couldn't read enhanced_otel YAML data.

**Root Cause**: 
1. `SessionParser::parse_session_content()` looked for incorrect data structures (`tool_calls` array directly under session log)
2. Parser only supported JSON format but database stored enhanced_otel data as YAML
3. Missing `parse_enhanced_otel_yml_tool()` method to handle RFC3339 timestamps from enhanced_otel format

**Solution Implemented**:
1. ✅ Removed incorrect parsing paths that don't exist in session logs
2. ✅ Enhanced session parser to parse YAML content from `log_content` using `serde_yaml::from_str`
3. ✅ Added `parse_enhanced_otel_yml_tool()` to handle enhanced_otel YAML format with proper timestamp parsing
4. ✅ Fixed brace structure and conditional flow in parser

**Files Modified**:
- `crates/reev-api/src/handlers/flow_diagram/session_parser.rs` - Enhanced YAML parsing support
- `crates/reev-api/src/services/benchmark_executor.rs` - Fixed test to use proper test files

**Testing Results**:
- ✅ Enhanced_otel JSONL to YML conversion pipeline working correctly
- ✅ Session parser successfully reads YAML format from database storage
- ✅ Flow diagram API now generates proper tool call visualizations:
   ```
   stateDiagram
       [*] --> Prompt
       Prompt --> Agent : Execute task
       Agent --> sol_transfer : 1 ix
       state sol_transfer {
           WALLET1 --> WALLET2 : 0.1 SOL
       }
       sol_transfer --> [*]
   ```
- ✅ Complete pipeline working: `run bench -> api -> agent -> runner -> otel -> enhanced_otel.jsonl -> yml -> db -> web`

## #2 - Performance: Database query optimization needed - ⚠️ NEW ISSUE
**Status**: Active Issue ❌
**Description**: Flow diagram API queries are taking longer than expected, especially for sessions with many tool calls.

**Root Cause**:
1. No indexes on frequently queried columns in session_logs table
2. JSON parsing happening at query time instead of storage time
3. Large YAML content being transferred in database queries

**Current Observations**:
- ✅ Parser now correctly extracts tool calls from YAML data
- ✅ Flow diagrams generate correctly when data is present
- ⚠️ Query response times increase with session count
- ⚠️ No pagination in flow diagram API for large datasets

**Files Affected**:
- Database schema for session_logs table
- Flow diagram API query logic

**Next Steps**:
1. 🔍 Add database indexes for session_id and timestamp columns
2. 🔍 Consider storing parsed tool calls in separate table for faster queries
3. 🔍 Implement pagination for flow diagram API
4. 🔍 Add response caching for frequently accessed sessions

