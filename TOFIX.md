# TOFIX.md - Current Issues to Fix

## ~~Flow Diagram Tool Name Bug~~ ✅ COMPLETELY RESOLVED
- **Issue**: Flow diagram shows generic tool names (`transfer_sol`) instead of actual tool names (`sol_transfer`)
- **Current Output**: `Agent --> transfer_sol : 0.1 SOL`
- **Expected Output**: `Agent --> sol_transfer : 1 ix`
- **Root Cause**: Hardcoded tool name mapping in session parser
- **Location**: `crates/reev-api/src/handlers/flow_diagram/session_parser.rs:290`
- **Priority**: High - affects flow diagram accuracy
- **Fix Applied**: 
  1. Created `reev-tools/src/tool_names.rs` with centralized tool name constants
  2. Updated `reev-tools/src/tools/native.rs` to use shared constants (`SOL_TRANSFER`, `SPL_TRANSFER`)
  3. Added `reev-tools` dependency to `reev-api`
  4. Updated session parser to import and use `reev_tools::tool_names::tool_name_from_program_id()`
  5. Now tool names come from the source of truth in `reev-tools` instead of being hardcoded
- **Architecture**: Proper decoupling achieved - `reev-tools` owns tool definitions, `reev-api` consumes them
