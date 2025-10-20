# TOFIX.md - Current Issues to Fix

## ~~Flow Diagram Tool Name Bug~~ ✅ FIXED
- **Issue**: Flow diagram shows generic tool names (`transfer_sol`) instead of actual tool names (`sol_transfer`)
- **Current Output**: `Agent --> transfer_sol : 0.1 SOL`
- **Expected Output**: `Agent --> sol_transfer : 1 ix`
- **Root Cause**: Tool name mapping not respecting `ToolDefinition.name` from `Self::NAME.to_string()`
- **Location**: Check flow diagram generation logic for tool name extraction
- **Priority**: High - affects flow diagram accuracy
- **Fix Applied**: Updated fallback logic in `reev-runner/src/lib.rs` to use correct tool name `sol_transfer` instead of `transfer_sol`
