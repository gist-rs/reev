# TOFIX.md - Current Issues to Fix

## ~~Flow Diagram Tool Name Bug~~ ✅ FIXED
- **Issue**: Flow diagram shows generic tool names (`transfer_sol`) instead of actual tool names (`sol_transfer`)
- **Current Output**: `Agent --> transfer_sol : 0.1 SOL`
- **Expected Output**: `Agent --> sol_transfer : 1 ix`
- **Root Cause**: Hardcoded tool name mapping in session parser
- **Location**: `crates/reev-api/src/handlers/flow_diagram/session_parser.rs:290`
- **Priority**: High - affects flow diagram accuracy
- **Fix Applied**: Updated hardcoded mapping from `"transfer_sol"` to `"sol_transfer"` to match `ToolDefinition::NAME`
