# TASKS: Context Builder Improvements

## 1. Tool Result Deserialization (COMPLETED)

**Problem**: Context builder used manual JSON extraction from tool results, which was error-prone and required verbose code.

**Solution**: Implemented typed deserialization with serde.
- Created `ExtractKeyInfo` trait in `types.rs` to standardize extraction logic
- Updated `convert_step_result_to_previous` in `mod.rs` to use typed structs
- Added compile-time guarantees for tool result structures

**Implementation Details**:
- Defined `JupiterSwapResult` and `JupiterLendResult` structs with relevant fields
- Created `TypedToolResult` enum to handle different tool result types
- Implemented `ExtractKeyInfo` trait for `TypedToolResult`, `JupiterSwapResult`, and `JupiterLendResult`
- Used `serde_json::from_value` for deserialization in `convert_step_result_to_previous`

## 2. Tool Output Standardization (PARTIALLY COMPLETED)

**Problem**: Different tools returned different JSON structures without a common format.

**Current Implementation**:
- Created `ToolResultWrapper` in `types.rs` with standard metadata fields
- Created `TypedToolResult` enum for different result types (JupiterSwap, JupiterLend, GenericOperation)
- Implemented `ExtractKeyInfo` trait for standardized extraction
- Added conversion from tool-specific `ToolResult` to `ToolResultWrapper`

**Not Yet Implemented**:
- Individual tool implementations still don't use `ToolResultWrapper` directly
- Tools in `crates/reev-core/src/execution/rig_agent/tools/` return tool-specific `ToolResult` enum
- Need to update tool implementations to return `ToolResultWrapper` directly

## 3. Type Safety in Context Builder (TODO)

**Problem**: `StepResult` struct uses generic `output: serde_json::Value` field.

**Current Implementation**:
```rust
pub struct StepResult {
    /// Step identifier
    pub step_id: String,
    /// Whether step succeeded
    pub success: bool,
    /// Error message if step failed
    pub error_message: Option<String>,
    /// Tool calls made during step
    pub tool_calls: Vec<String>,
    /// Step output
    pub output: serde_json::Value,
    /// Execution duration in milliseconds
    pub execution_time_ms: u64,
    /// Tool results (new field for standardized tool outputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<serde_json::Value>>,
}
```

*Status: Still using generic `output: serde_json::Value` field. A new `tool_results` field has been added but still uses generic JSON values. This is a breaking change that requires coordination with other parts of the system.*

## 4. Input Mint Consolidation (COMPLETED)

**Problem**: Inconsistent `input_mint` usage throughout codebase with duplicated validation logic.

**Solution**: Created type-safe token mint handling.
- Implemented `TokenMint` wrapper with validation in `token_mint.rs`
- Centralized common mint addresses in constants module
- Added consistent conversion between string and Pubkey representations

**Implementation Details**:
- Created `TokenMint` struct with address, symbol, decimals, and price_usd
- Implemented `FromStr` trait for validation
- Added helper methods for checking common mint types (is_sol, is_usdc, etc.)
- Created `mints` module with constants for common mint addresses
- Added `helpers` module with `resolve_mint_or_default` and `get_mint_symbol` functions

## Implementation Status

- ✅ **High Priority**: Refactor `convert_step_result_to_previous` (COMPLETED)
- ✅ **High Priority**: Create typed wrapper for token mints (COMPLETED)
- 🔄 **Medium Priority**: Standardize tool output structures (PARTIALLY COMPLETED)
- ⏳ **Low Priority**: Implement type-safe StepResult structure (TODO)

## Testing Considerations

- ✅ Add tests for deserialization of tool result structures
- ⏳ Ensure backward compatibility when changing tool formats
- ✅ Add tests for context builder information extraction
- ✅ Add tests for TokenMint conversion and validation
- ⏳ Add tests for tool implementations to verify they return proper typed results

## Next Steps

1. Create unit tests for `TokenMint` implementation
2. Update tool implementations in `crates/reev-core/src/execution/rig_agent/tools/` to return `ToolResultWrapper` directly
3. Consider refactoring `StepResult` to use typed results instead of generic `output: serde_json::Value`
4. Add tests for tool implementations to verify they return proper typed results