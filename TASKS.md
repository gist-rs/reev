# TASKS: Context Builder Improvements

## 1. Improve Tool Result Deserialization in Context Builder (COMPLETED)

**Previous Implementation in `crates/reev-core/src/execution/context_builder/mod.rs`**:
```rust
// Extract swap information
if let Some(jupiter_swap) = tool_result.get("jupiter_swap") {
    if let (
        Some(input_mint),
        Some(output_mint),
        Some(input_amount),
        Some(output_amount),
    ) = (
        jupiter_swap.get("input_mint").and_then(|v| v.as_str()),
        jupiter_swap.get("output_mint").and_then(|v| v.as_str()),
        jupiter_swap.get("input_amount").and_then(|v| v.as_u64()),
        jupiter_swap.get("output_amount").and_then(|v| v.as_u64()),
    ) {
        // Manual extraction code...
    }
}
```

**Issues with Previous Approach**:
1. Error-prone - typos in field names won't be caught at compile time
2. Harder to maintain - changes to tool output structures require manual updates
3. Less type-safe - no compile-time guarantees about structure
4. Requires verbose code to extract nested values

**Implemented Solution**:
1. ✅ Defined typed structs for each tool's result in `types.rs`
2. ✅ Used serde to deserialize the JSON output into these structs
3. ✅ Work with the typed structs instead of raw JSON
4. ✅ Created `ExtractKeyInfo` trait to standardize extraction logic

**Implementation Details**:

1. Created `types.rs` with:
   - `JupiterSwapResult` struct with all relevant fields
   - `JupiterLendResult` struct with all relevant fields
   - `ExtractKeyInfo` trait with methods for extracting key information, balance changes, constraints, and available tokens
   - Implemented the trait for both result types

2. Updated `convert_step_result_to_previous` in `mod.rs` to:
   - Use typed deserialization with `serde_json::from_value`
   - Call the trait methods instead of manual field extraction
   - Fix type mismatches for symbol assignment

3. The context builder now has type-safe access to tool results with compile-time guarantees.

## 2. Standardize Tool Output Structure (PARTIALLY COMPLETED)

**Problem**: Different tools return different JSON structures without a common format.

**Current Issues**:
- Jupiter swap returns `JupiterSwapResponse`
- Jupiter lend returns a different structure
- No standard error format across tools

**Partially Implemented Solution**:
1. ✅ Defined a common `ToolResultWrapper` that all tools can use
2. ✅ Each specific tool result implements a common trait (`ExtractKeyInfo`)
3. ✅ Included standard metadata like execution time, tool name, etc.
4. 🔄 Next: Update individual tools to use `ToolResultWrapper` (not yet implemented)

**Implementation Details**:

1. Created `ToolResultWrapper` in `types.rs` with:
   - Standard fields for all tool results
   - Method to convert to typed result enum
   - Support for additional metadata

2. Created `TypedToolResult` enum to handle different result types:
   - `JupiterSwap(JupiterSwapResult)`
   - `JupiterLend(JupiterLendResult)`
   - `Other(String, serde_json::Value)` for unknown types

3. **Remaining Work**: Update individual tool implementations to return `ToolResultWrapper` instead of their current response formats.

## 3. Improve Type Safety in Context Builder (TODO)

**Problem**: The `StepResult` struct uses a generic `output: serde_json::Value` field.

**Recommended Solution**:
1. Replace with a more structured approach
2. Add type-specific deserialization based on tool calls made

**Example Implementation**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub tool_calls: Vec<String>,
    // Replace generic output with structured approach
    pub tool_results: HashMap<String, serde_json::Value>,
    pub execution_time_ms: u64,
}
```

**Status**: Not yet implemented. This would be a breaking change and requires coordination with other parts of the system.

## 4. Consolidate `input_mint` Usage Analysis
## 4. Consolidate `input_mint` Usage Analysis (COMPLETED)

**Previous Usage Throughout the Codebase**:

1. **Context Builder**: Manual extraction from JSON in `convert_step_result_to_previous`
   ```rust
   jupiter_swap.get("input_mint").and_then(|v| v.as_str())
   ```

2. **Jupiter Swap Tool**: Used in `JupiterSwapArgs` struct and validation
   ```rust
   pub struct JupiterSwapArgs {
       pub input_mint: String,
       // ...
   }
   ```

3. **Protocol Validation**: Required parameter validation in `PLAN_PROTOCOLS.md`
   ```rust
   if !operation.parameters.contains_key("input_mint") {
       return Err(ValidationError::MissingParameter("input_mint".to_string()));
   }
   ```

4. **Benchmarks**: Hardcoded values in agent implementations
   ```rust
   let input_mint = native_mint::ID; // SOL mint
   ```

**Previous Issues with Usage**:

1. **Type Inconsistency**: Sometimes `input_mint` is a `String`, sometimes `Pubkey`
2. **Validation Duplication**: Similar validation logic in multiple places
3. **Hardcoded Values**: Mint addresses often hardcoded instead of using constants

**Implemented Solutions**:

1. ✅ Created `TokenMint` typed wrapper for token mints with validation in `token_mint.rs`
2. ✅ Centralized common mint addresses in a constants module (`mints`)
3. ✅ Implemented consistent conversion between string and Pubkey representations

**Implementation Details**:

1. Created `token_mint.rs` with:
   - `TokenMint` struct with address, symbol, decimals, and price_usd
   - `FromStr` trait implementation for validation
   - Conversion methods to/from `Pubkey`
   - Helper methods for checking common mint types (is_sol, is_usdc, etc.)

2. Created `mints` constants module with:
   - Constants for common mint addresses (SOL, USDC, USDT, RAY)
   - Helper functions to create pre-populated `TokenMint` instances

3. Created `helpers` module with:
   - `resolve_mint_or_default` function for handling key_map lookups and validation
   - `get_mint_symbol` function for display purposes

The context builder and other parts of the system can now use these typed wrappers for better type safety and validation.

## Implementation Status

1. ✅ **High Priority**: Refactor `convert_step_result_to_previous` to use typed deserialization (COMPLETED)
2. ✅ **High Priority**: Create typed wrapper for token mints and centralize constants (COMPLETED)
3. 🔄 **Medium Priority**: Standardize tool output structures across all tools (PARTIALLY COMPLETED)
4. ⏳ **Low Priority**: Implement more type-safe StepResult structure (TODO)

## Testing Considerations

- ✅ Add tests for deserialization of each tool's result structure (can now be added)
- ⏳ Ensure backward compatibility when changing tool output formats (needs validation)
- ✅ Add tests to verify that context builder correctly extracts information from typed structs (can now be added)
- ✅ Add tests for TokenMint conversion and validation (can now be added)

## Next Steps

1. Create unit tests for the new typed structures in `types.rs`
2. Create unit tests for the `TokenMint` implementation in `token_mint.rs`
3. Update tool implementations to use `ToolResultWrapper` for better standardization
4. Consider refactoring `StepResult` to use typed results (breaking change)