# Issues

## Issue #37 - ToolName Enum Mismatch and Missing Tools - IN PROGRESS 🟡
**Status**: PROGRESS
**Progress**: Fixed enum definitions, YML files, and created tool registry. Still need to update Rust codebase.
**Description**: ToolName enum has multiple serious issues: missing tools, wrong serialization names, and redundant tools, PLUS entire codebase uses untyped strings instead of type-safe enum
**Problems**:
- `spl_transfer` tool is missing from enum but actively used throughout codebase
- `jupiter_withdraw` serializes to wrong name - should be `jupiter_lend_earn_withdraw`
- `account_balance` serializes to wrong name - should be `get_account_balance`
- `lend_earn_tokens` serializes to wrong name - should be `get_jupiter_lend_earn_tokens`
- `JupiterLend` tool is ambiguous/unused and doesn't exist in actual tools
- `ExecuteTransaction` has no actual implementation (conceptual only)
- `JupiterPositions` is redundant with `GetPositionInfo`

### **NAMING CONSISTENCY ISSUE IN ENUM DEFINITION**
The current enum has mixed naming patterns causing confusion:
```rust
// ❌ MIXED NAMING PATTERNS - INCONSISTENT
#[derive(Debug, Clone, Display, EnumString, IntoStaticStr, VariantNames)]
pub enum ToolName {
    GetAccountBalance,           // serialize: "get_account_balance" ✅ FIXED
    GetJupiterEarnPosition,      // serialize: "get_jupiter_earn_position" ✅ RENAMED
    GetJupiterLendEarnTokens,    // serialize: "get_jupiter_lend_earn_tokens" ✅ RENAMED
    
    SolTransfer,                   // serialize: "sol_transfer" ✅
    JupiterSwap,                   // serialize: "jupiter_swap" ✅
    JupiterLendEarnWithdraw,       // serialize: "jupiter_lend_earn_withdraw" ✅ RENAMED
}

// ❌ PROBLEM: Mixed patterns between "GetXxx" and "Xxxx" variants
// Some have "Get" prefix, others don't - inconsistent naming
```

**Naming Analysis**:
- `GetAccountBalance` vs `SolTransfer` - inconsistent prefix usage
- `GetJupiterEarnPosition` vs `JupiterSwap` - mixed patterns
- `JupiterLendEarnWithdraw` follows different pattern than `GetXxx` tools
- Need consistent naming convention across ALL tools

**Root Cause of Confusion**:
The current enum mixes two different naming philosophies:
1. **Discovery tools**: Use "Get" prefix (GetAccountBalance, GetJupiterPositionInfo, GetJupiterLendEarnTokens)
2. **Transaction/Action tools**: Use direct naming (SolTransfer, JupiterSwap, JupiterLendEarnDeposit)

But `GetJupiterEarnPosition` breaks this pattern by having "Get" prefix while being action-based, and it duplicates `JupiterEarn` functionality.

**Actual Tool Implementation Analysis**:
- `GetAccountBalanceTool::NAME = "get_account_balance"` ✅
- `PositionInfoTool::NAME = "get_jupiter_position_info"` ✅ 
- `LendEarnTokensTool::NAME = "get_jupiter_lend_earn_tokens"` ✅
- `JupiterEarnTool::NAME = "jupiter_earn"` ✅ (positions + earnings, benchmark only)

**The Real Issue**:
`GetJupiterEarnPosition` doesn't exist as an actual tool! It should be either:
- Remove entirely (use `JupiterEarn` for positions + earnings)
- OR rename to match existing `PositionInfoTool` (`GetJupiterPositionInfo`)
- OR clarify if this is supposed to be a different tool entirely

### **CRITICAL ARCHITECTURAL PROBLEM: String-based Tool Names Everywhere**

The codebase violates type safety by using hardcoded strings instead of the type-safe enum:

```rust
// ❌ CURRENT - UNSAFE STRINGS EVERYWHERE
let tool_name_list = vec![
    "sol_transfer".to_string(),
    "spl_transfer".to_string(),
    "jupiter_swap".to_string(),
    "jupiter_earn".to_string(),
    "jupiter_lend_earn_deposit".to_string(),
    "jupiter_lend_earn_withdraw".to_string(),
    "jupiter_lend_earn_mint".to_string(),
    "jupiter_lend_earn_redeem".to_string(),
    "account_balance".to_string(),  // WRONG - should be "get_account_balance"
    "lend_earn_tokens".to_string(), // WRONG - should be "get_jupiter_lend_earn_tokens"
];

// ❌ HARDCODED STRING MATCHING
match tool_name.as_str() {
    "sol_transfer" => { /* ... */ },
    "spl_transfer" => { /* ... */ },
    "jupiter_swap" => { /* ... */ },
    // Hundreds of these throughout codebase!
}
```

### **RIG TOOL ALREADY PROVIDES TYPE-SAFE NAMES**

Every tool already implements `const NAME: &'static str`:

```rust
// ✅ FROM RIG TOOL TRAIT - ALREADY TYPE-SAFE
impl Tool for SolTransferTool {
    const NAME: &'static str = "sol_transfer";  // ✅ SINGLE SOURCE OF TRUTH
    // ...
}

impl Tool for JupiterEarnTool {
    const NAME: &'static str = "jupiter_earn";   // ✅ SINGLE SOURCE OF TRUTH
    // ...
}
```

**Root Cause**:
- ToolName enum created without auditing actual tool implementations
- String-based tool management defeats purpose of type-safe enum
- Rig's `Tool::NAME` constants exist but aren't used
- No shared type crate for tool management
**Required Changes**:
### Required Changes:

### 1. FIX ENUM NAMING CONSISTENCY FIRST
Before any code updates, resolve the naming inconsistency:

```rust
// ✅ OPTION 1 - Use "Get" prefix for ALL discovery tools
#[derive(Debug, Clone, Display, EnumString, IntoStaticStr)]
pub enum ToolName {
    // Discovery Tools - ALL with "Get" prefix
    GetAccountBalance,           // "get_account_balance"
    GetJupiterPositionInfo,      // "get_jupiter_position_info" 
    GetJupiterLendEarnTokens,    // "get_jupiter_lend_earn_tokens"
    
    // Transaction Tools - NO "Get" prefix (action-based)
    SolTransfer,                 // "sol_transfer"
    SplTransfer,                 // "spl_transfer"
    JupiterSwap,                 // "jupiter_swap"
    JupiterSwapFlow,             // "jupiter_swap_flow"
    
    // Jupiter Earn Tools - NO "Get" prefix (action-based)
    JupiterEarn,                 // "jupiter_earn" (positions + earnings)
    JupiterLendEarnDeposit,     // "jupiter_lend_earn_deposit"
    JupiterLendEarnWithdraw,     // "jupiter_lend_earn_withdraw"
    JupiterLendEarnMint,         // "jupiter_lend_earn_mint"
    JupiterLendEarnRedeem,       // "jupiter_lend_earn_redeem"
}
```

**Clarification on Tool Distinctions**:
- `GetAccountBalance` - General wallet balance discovery
- `GetJupiterPositionInfo` - Jupiter-specific position discovery ONLY  
- `JupiterEarn` - Jupiter-specific positions AND earnings (benchmark restricted)
- These are NOT duplicates - serve different purposes with different data returned

### 2. Move ToolName to Shared Type Crate
Create `crates/reev-types/src/tool_registry.rs` with:

```rust
// ✅ TYPE-SAFE TOOL REGISTRY
pub struct ToolRegistry;

impl ToolRegistry {
    /// Get ALL tool names using Rig's type-safe constants
    pub fn all_tools() -> Vec<&'static str> {
        vec![
            SolTransferTool::NAME,      // "sol_transfer"
            SplTransferTool::NAME,      // "spl_transfer"
            JupiterSwapTool::NAME,      // "jupiter_swap"
            JupiterEarnTool::NAME,       // "jupiter_earn"
            // ... use actual Tool::NAME constants
        ]
    }

    /// Get tool category using enum
    pub fn category(tool_name: &str) -> Option<ToolCategory> {
        ToolName::from_str_safe(tool_name)?.category().into()
    }

    /// Validate tool name against Rig constants
    pub fn is_valid_tool(tool_name: &str) -> bool {
        Self::all_tools().contains(&tool_name)
    }
}
```

### 2. Enhanced ToolName enum with strum integration
```rust
// ✅ USE STRUM FOR TYPE-SAFE CONVERSIONS
use strum::{Display, EnumString, IntoStaticStr, VariantNames};

#[derive(Debug, Clone, Display, EnumString, IntoStaticStr, VariantNames)]
pub enum ToolName {
    // Discovery & Information Tools (Jupiter-focused)
    GetAccountBalance,           // serialize: "get_account_balance" ✅ FIXED
    GetJupiterEarnPosition,      // serialize: "get_jupiter_earn_position" ✅ RENAMED
    GetJupiterLendEarnTokens,    // serialize: "get_jupiter_lend_earn_tokens" ✅ RENAMED

    // Transaction Tools (Jupiter operations)
    SolTransfer,                   // serialize: "sol_transfer" ✅
    SplTransfer,                   // serialize: "spl_transfer" ✅ ADDED
    JupiterSwap,                   // serialize: "jupiter_swap" ✅
    JupiterSwapFlow,               // serialize: "jupiter_swap_flow" ✅
    JupiterLendEarnDeposit,       // serialize: "jupiter_lend_earn_deposit" ✅
    JupiterLendEarnWithdraw,       // serialize: "jupiter_lend_earn_withdraw" ✅ RENAMED
    JupiterLendEarnMint,          // serialize: "jupiter_lend_earn_mint" ✅
    JupiterLendEarnRedeem,        // serialize: "jupiter_lend_earn_redeem" ✅
}

impl ToolName {
    /// ✅ TYPE-SAFE VALIDATION against Rig constants
    pub fn validate_against_rig_tools(&self) -> bool {
        ToolRegistry::all_tools().contains(&self.as_str())
    }

    /// ✅ GET ALL TOOLS without string literals
    pub fn all_tools() -> Vec<Self> {
        Self::VARIANTS.iter().map(|&variant| variant).collect()
    }
}
```

### 3. Replace String-based Tool Management
```rust
// ❌ BEFORE - HARDCODED STRINGS
let tool_name_list = vec![
    "sol_transfer".to_string(),
    "spl_transfer".to_string(),
    // ... error-prone, untyped
];

// ✅ AFTER - TYPE-SAFE
let tool_name_list = ToolName::all_tools()
    .iter()
    .map(|tool| tool.to_string())
    .collect();

// ✅ TYPE-SATE MATCHING
match tool_name.parse::<ToolName>() {
    Ok(ToolName::JupiterSwap) => { /* ... */ },
    Ok(ToolName::SplTransfer) => { /* ... */ },
    Err(_) => return Err("Invalid tool name"),
}
```

### Code Updates Required:
### Files to Update (EXTENSIVE STRING->ENUM REPLACEMENT):

#### Core Type System:
1. **crates/reev-types/src/tools.rs** - Update enum with strum integration
2. **crates/reev-types/src/tool_registry.rs** - Create type-safe tool registry

#### Agent Tool Management:
3. **crates/reev-agent/src/enhanced/common/mod.rs** - Replace string tool lists
4. **crates/reev-agent/src/enhanced/openai.rs** - Tool availability checks
5. **crates/reev-agent/src/enhanced/zai_agent.rs** - Tool execution logic
6. **crates/reev-agent/src/flow/agent.rs** - Tool list creation

#### Orchestrator Logic:
7. **crates/reev-orchestrator/src/dynamic_mode.rs** - String matching → enum matching
8. **crates/reev-orchestrator/src/gateway.rs** - Tool references
9. **crates/reev-orchestrator/src/execution/ping_pong_executor.rs** - Tool validation

#### Infrastructure:
10. **crates/reev-lib/src/otel_extraction/mod.rs** - Tool pattern matching
11. **crates/reev-lib/src/llm_agent.rs** - Tool determination logic
12. **crates/reev-tools/src/tool_names.rs** - Replace with enum constants

#### Tests & Benchmarks:
13. All test files with hardcoded tool strings
14. Any YAML files with hardcoded tool names
15. Integration tests with tool name assertions

#### String Pattern Replacements (500+ locations):
```rust
// ❌ FIND AND REPLACE THESE PATTERNS:
"sol_transfer" -> ToolName::SolTransfer
"spl_transfer" -> ToolName::SplTransfer
"jupiter_swap" -> ToolName::JupiterSwap
"jupiter_earn" -> ToolName::GetJupiterEarn
// ... all tool names

// ❌ REPLACE THESE MATCHING PATTERNS:
match tool_name.as_str() { -> match tool_name.parse::<ToolName>() {
if tool_name == "jupiter_swap" -> if let Ok(ToolName::JupiterSwap) = tool_name.parse()
tool_name.contains("jupiter") -> tool_name.parse::<ToolName>()?.is_jupiter_tool()
```

**Next Steps**:
1. Create `crates/reev-types/src/tool_registry.rs` with Rig-based tool registry
2. Update `ToolName` enum with strum integration and correct names
3. **MASSIVE SEARCH-REPLACE**: Convert all hardcoded string tool names to enum usage
4. Update all `match tool_name.as_str()` patterns to use enum parsing
5. Remove all `vec!["tool1", "tool2", ...]` patterns and use `ToolName::all_tools()`
6. Update agent tool availability checks to use enum validation
7. Update orchestrator logic to use enum-based matching
8. Fix test assertions to use enum instead of string comparison
9. Update any YAML references to use correct tool names
10. Run `cargo clippy --fix --allow-dirty` to catch remaining string usage
11. Comprehensive testing to ensure no runtime tool name mismatches
12. Update documentation to reflect type-safe tool management

**CRITICAL**: This is not just an enum fix - it's a **complete architectural refactor** from string-based to type-safe tool management throughout the entire codebase.

## Issue #35 - Jupiter Static Benchmarks Broken - NEW 🔴
**Status**: CRITICAL
**Description**: Static Jupiter benchmarks (200-series) fail with deterministic agent while dynamic benchmarks (300-series) work perfectly with LLM agents
**Problem**:
- 200-jup-swap-then-lend-deposit fails with "Transaction simulation failed: Error processing Instruction 0: custom program error: 0x1"
- Deterministic agent generates invalid Jupiter instructions
- Flow diagram shows 0 tool calls for failed static benchmarks
- Dynamic benchmarks work fine with real Jupiter execution
**Evidence**:
- 200 benchmark: Score 0, transaction simulation error, no tool calls captured
- 300 benchmark: Score 100%, successful Jupiter swap, proper tool call capture
- 001 benchmark: Score 100%, deterministic agent works fine for simple operations
- The issue is specific to Jupiter operations with deterministic agent
**Impact**:
- All Jupiter yield farming benchmarks broken in static mode
- Users cannot test multi-step Jupiter strategies with deterministic execution
- Flow visualization broken for static Jupiter operations
**Root Cause**:
Deterministic agent has hardcoded Jupiter instruction generation that produces invalid transactions for current Jupiter program state
**Next Steps**:
- Fix deterministic agent Jupiter instruction generation
- Or migrate static Jupiter benchmarks to use dynamic flow with LLM agent
- Ensure backward compatibility for existing static benchmarks

## Issue #33 - Flow Type Field Missing - NEW 🐛
**Status**: NEW
**Description**: SessionResponse lacks flow_type field needed for frontend flow selection
**Problem**:
- Frontend cannot differentiate between benchmark and dynamic execution modes
- Flow type information lost in API response serialization
- Dynamic flows cannot be properly displayed without flow context
**Root Cause**:
SessionResponse struct missing flow_type field despite database storing it
**Next Steps**:
- Add flow_type field to SessionResponse struct
- Update serialization to include flow type
- Test with both benchmark and dynamic execution modes
