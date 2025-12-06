# Reev Project Issues

## Issue #319: Untyped serde_json::Value returns in rig_agent tools
The tool implementations in `reev/crates/reev-core/src/execution/rig_agent/tools/` were returning `serde_json::Value` which was untyped and not type-safe.

### Files affected:
- `sol_transfer.rs`: Returns JSON for SOL transfer operations
- `jupiter_swap.rs`: Returns JSON for Jupiter swap operations
- `spl_transfer.rs`: Returns JSON for SPL token transfers
- `jupiter_lend.rs`: Returns JSON for Jupiter lend operations
- `account_balance.rs`: Returns JSON for account balance queries

### Solution implemented:
1. Created `tool_results.rs` with specific result structs for each tool
2. Updated all tool functions to return these typed structs (ToolResult enum)
3. Modified trait definitions in `traits.rs` to return appropriate types
4. Updated implementation.rs to handle new ToolResult types
5. Updated mod.rs to re-export new result types

### Status: Fixed

## Issue #320: Untyped parameter access using HashMap.get() in rig_agent tools
The tool implementations in `reev/crates/reev-core/src/execution/rig_agent/tools/` were using untyped HashMap<String, String>.get() for parameter access, which is error-prone and not type-safe.

### Files affected:
- All tool implementations using `params.get("key")` pattern

### Solution implemented:
1. Created `tool_params.rs` with specific parameter structs for each tool
2. Added `from_hashmap()` methods with compile-time validation
3. Created `ToolParams` enum to centralize parameter handling
4. Updated all tool implementations to use typed parameters
5. Updated implementation.rs to use `ToolParams::from_tool_name_and_params()`

### Benefits:
- Compile-time validation of required parameters
- Better type safety for parameter values
- Clear documentation of expected parameters
- Reduced runtime errors from parameter access

### Status: Fixed

## No Current Issues