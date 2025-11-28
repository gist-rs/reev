-# Reev Core Implementation Issues

---

## Issue #118: Jupiter Lend Deposit Execution (COMPLETED)
-### Status: COMPLETED
-### Description:
-Jupiter lend deposit tool was returning raw instructions instead of executing transactions and returning signatures. Fixed by updating execute_jupiter_lend_deposit to execute transactions like jupiter_swap.
-
-### Tasks Completed:
-1. ✅ Fixed execute_jupiter_lend_deposit to execute transaction and return signature
-2. ✅ Added proper error handling for transaction execution
-3. ✅ Verified e2e_lend tests passing
-
----
-
-## Issue #119: E2E Lend Tests Implementation (COMPLETED)
-### Status: COMPLETED
-### Description:
-Added comprehensive end-to-end tests for Jupiter lending operations to validate the system.
-
-### Tasks Completed:
-1. ✅ Created e2e_lend.rs with test_lend_100_usdc and test_lend_all_usdc
-2. ✅ Added setup_wallet_for_lend function to common module
-3. ✅ Verified jUSDC token receipt after lending
-4. ✅ All tests passing
-
----
-
-## Issue #120: Multi-Step Swap+Lend Flow Test (COMPLETED)
-### Status: COMPLETED
-### Description:
-Updated e2e_multi_step.rs to test swap then lend operations, proving multi-step flows work correctly.
-
-### Tasks Completed:
-1. ✅ Updated test to use "swap 0.1 SOL to USDC then lend 100 USDC" prompt
-2. ✅ Added verification for jUSDC tokens after lending
-3. ✅ Verified proper context passing between steps
-4. ✅ Removed unused helper functions
-
----
-
-## Issue #117: Context Update in Multi-Step Flows (IN PROGRESS)
### Status: PARTIALLY FIXED
-### Description:
-Context updates between steps are partially working. While swap step updates context correctly, subsequent lend step may receive stale balance information.
-
### Root Cause:
-Context serialization issue when passing updated context to LLM for next step.
-
-### Tasks Required:
-1. ✅ Fixed executor to pass updated context to subsequent steps
-2. ⚠️ Investigate serialization of updated context for LLM
-3. Verify fix for all token types in multi-step flows
-
----
-
-## Issue #102: Error Recovery Engine (NOT STARTED)
-### Status: NOT STARTED
-### Description:
-Error recovery is incomplete. Need retry logic and alternative paths for failed transactions.
-
-### Tasks Required:
-1. Implement retry with exponential backoff for network errors
-2. Add alternative path selection for transaction failures
-3. Implement slippage adjustment for swap failures
-4. Create error classification system
-
----
-
-## Issue #105: RigAgent Enhancement (PARTIALLY COMPLETED)
-### Status: PARTIALLY COMPLETED
-### Description:
-RigAgent needs improved tool selection and parameter extraction for complex operations.
-
-### Tasks Remaining:
-1. Better prompt engineering for complex tool selection
-2. Parameter validation before tool execution
-3. Tool result interpretation
-4. Enhanced error handling for edge cases
-
----
-
-## Issue #106: LanguageRefiner Improvement (PARTIALLY COMPLETED)
-### Status: PARTIALLY COMPLETED
-### Description:
-LanguageRefiner needs better context awareness and multi-language support.
-
-### Tasks Remaining:
-1. Context awareness integration
-2. Multi-language support implementation
-3. Refinement quality metrics
-4. Advanced error recovery
-
----
-
-## Issue #110: Remove Unused Code (NOT STARTED)
-### Status: NOT STARTED
-### Description:
-Clean up deprecated functions in YmlGenerator following V3 plan guidelines.
-
-### Tasks Required:
-1. Remove rule-based operation parsing functions
-2. Keep builder functions for testing only
-3. Update documentation to reflect current implementation
-4. Ensure tests still pass after cleanup
-
----
-
-## Issue #112: Comprehensive Error Recovery (NOT STARTED)
-### Status: NOT STARTED
-### Description:
-Implement robust error recovery with different strategies based on error type.
-
-### Tasks Required:
-1. Create error classification system
-2. Implement retry logic with backoff
-3. Add circuit breaker for repeated failures
-4. Add recovery metrics and logging
-
----
-
-### Implementation Priority
-
-### Week 1:
-1. Issue #117: Context Update in Multi-Step Flows (IN PROGRESS)
-2. Issue #110: Remove Unused Code (NOT STARTED)
-
-### Week 2:
-3. Issue #102: Error Recovery Engine (NOT STARTED)
-4. Issue #112: Comprehensive Error Recovery (NOT STARTED)
-
-### Current State Summary:
-All e2e tests passing:
-- ✅ swap operations (including "sell all SOL")
-- ✅ transfer operations
-- ✅ lend operations (including "lend all USDC")
-- ✅ multi-step operations (swap then lend)
-
-Architecture aligned with V3 plan:
-- ✅ RigAgent handles tool selection based on refined prompts
-- ✅ Balance validation integrated and working
-- ⚠️ Error recovery implementation needed for production readiness
# Reev Core Implementation Issues

---

## Issue #122: Rule-Based Multi-Step Detection Contradicts V3 Architecture (CRITICAL)
### Status: CRITICAL ARCHITECTURAL VIOLATION
### Description:
Implementation has introduced rule-based logic for multi-step detection in unified_flow_builder.rs, directly contradicting the V3 plan which specifies that LLM should handle multi-step detection, not rule-based parsing.

### V3 Plan Requirements:
From PLAN_CORE_V3.md:
- "RigAgent should handle tool selection based on refined prompts, not a rule-based parser"
- "LLM's role is specifically for language refinement, not structure generation"
- Phase 1: LLM-based prompt refinement
- Phase 2: Rig-driven tool execution (NOT rule-based)

### Current Implementation Issue:
In unified_flow_builder.rs, rule-based logic was added:
```rust
let is_multi_step = prompt_lower.contains(" then ")
    || prompt_lower.contains(" and ")
    || prompt_lower.contains(" followed by ")
    || (prompt_lower.contains("swap") && prompt_lower.contains("lend"))
    || (prompt_lower.contains("swap") && prompt_lower.contains("transfer"))
    || (prompt_lower.contains("lend") && prompt_lower.contains("transfer"));
```

This violates the V3 architecture by:
1. Using rule-based detection instead of LLM
2. Pre-determining operations instead of letting RigAgent handle them
3. Creating complex parsing logic where LLM should make decisions

### Root Cause Analysis:
After examining all e2e tests and the implementation, it's clear that this rule-based approach is a fundamental misunderstanding of the V3 architecture. The V3 plan explicitly states:
- LLM should handle all language understanding
- No rule-based parsing should be used for operation detection
- RigAgent should determine tools based on refined prompts

### Correct V3 Architecture Approach:
1. LLM should refine prompts and naturally detect multi-step operations
2. YML generator should create simple structures with refined prompts
3. RigAgent should handle tool selection based on refined prompts
4. No rule-based parsing for operation detection

### Tasks Required:
1. Remove rule-based multi-step detection from unified_flow_builder.rs
2. Simplify flow builder to create single steps with refined prompts
3. Ensure LLM properly handles multi-step detection through prompt refinement
4. Update RigAgent to handle all operations in multi-step prompts
5. Fix e2e_multi_step.rs test expectations to align with V3 architecture

---

## Issue #121: Multi-Step Operations Not Properly Executed (CRITICAL)
### Status: CRITICAL ISSUE IDENTIFIED
### Description:
Multi-step flows are not properly executing all operations. The current implementation generates multiple steps correctly, but only executes the first operation in each step.

### Current Behavior:
- Prompt: "swap 0.1 SOL to USDC then lend 10 USDC"
- Planner generates 2 steps correctly
- However, test fails because USDC balance doesn't decrease after "lending"
- Root cause: RigAgent is executing only the first operation from multi-step prompts

### Root Cause Analysis:
The issue is in RigAgent's execution of multi-step flows. When processing a step like "swap 0.1 SOL to USDC then lend 10 USDC", the RigAgent's LLM prompt and tool execution only handle the first operation ("swap") and ignore the second operation ("lend"). This happens because:

1. The LLM is prompted to extract a single tool operation
2. There's no mechanism to identify and execute all operations in a multi-step prompt
3. The context passing between operations is incomplete

### Why Tests Were Passing Before:
- Previous test used unrealistic amounts (swap 0.1 SOL for $15, then lend 100 USDC)
- With 100 USDC initial balance, this created a false sense of success
- When changed to realistic amounts (lend 10 USDC from ~15 USDC swap output), test failed

### Tasks Required:
1. Fix RigAgent to properly execute all operations in multi-step steps
2. Update the LLM prompt to explicitly identify and execute ALL operations
3. Ensure context is properly updated between sequential operations
4. Add validation that all operations in prompt are being executed
5. Consider implementing a sequential execution pattern within a single step

---

## Issue #118: Jupiter Lend Deposit Execution (COMPLETED)
### Status: COMPLETED
### Description:
Fixed jupiter_lend_deposit to execute transactions and return signatures instead of just returning instructions.

### Tasks Completed:
1. ✅ Updated execute_jupiter_lend_deposit to execute transaction like jupiter_swap
2. ✅ Added proper error handling for transaction execution
3. ✅ Verified e2e_lend tests passing

---

## Issue #119: E2E Lend Tests Implementation (COMPLETED)
### Status: COMPLETED
### Description:
Added comprehensive end-to-end tests for Jupiter lending operations.

### Tasks Completed:
1. ✅ Created e2e_lend.rs with test_lend_100_usdc and test_lend_all_usdc
2. ✅ Added setup_wallet_for_lend function to common module
3. ✅ Verified jUSDC token receipt after lending

---

## Issue #123: ContextResolver Using Mainnet Instead of SURFPOOL (COMPLETED)
### Status: COMPLETED
### Description:
Tests are using ContextResolver with mainnet-beta RPC while transaction execution uses SURFPOOL. This creates a fundamental inconsistency where the context resolver sees different account states than what transactions operate on.

### Current Implementation Issue:
In e2e_multi_step.rs and other tests, the context resolver is initialized with:
```rust
let context_resolver = ContextResolver::new(SolanaEnvironment {
    rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
});
```

But transactions are executed through SURFPOOL at http://localhost:8899.

### Why This Is Critical:
1. Context resolver reads real mainnet state
2. Test setup creates tokens in SURFPOOL via surfnet_setTokenAccount
3. LLM receives context from mainnet (different balances)
4. Transactions execute in SURFPOOL (different state)
5. This causes unpredictable test behavior

### Correct Approach:
Both context resolution and transaction execution must use SURFPOOL to ensure consistency:
```rust
let context_resolver = ContextResolver::new(SolanaEnvironment {
    rpc_url: Some("http://localhost:8899".to_string()),
});
```

### Tasks Completed:
1. ✅ Updated all e2e tests to use SURFPOOL URL for context resolver
2. ✅ Ensured SURFPOOL is running before context resolution (already handled in tests)
3. ✅ Updated default SolanaEnvironment to use SURFPOOL
4. ✅ Updated default SURFPOOL_RPC_URL in ContextResolver
5. ✅ Fixed jupiter_swap.rs to use environment-aware RPC URL

### Files Changed:
- crates/reev-core/tests/e2e_multi_step.rs
- crates/reev-core/tests/e2e_swap.rs
- crates/reev-core/tests/e2e_lend.rs
- crates/reev-core/tests/e2e_transfer.rs
- crates/reev-core/tests/e2e_rig_agent.rs
- crates/reev-core/src/execution/handlers/swap/jupiter_swap.rs
- crates/reev-core/src/context.rs

---

## Issue #124: Context Resolution Inconsistency Between Tests (PARTIALLY FIXED)
### Status: PARTIALLY FIXED
### Description:
Some e2e tests are still using mainnet RPC URL for context resolution instead of SURFPOOL, creating inconsistency between context resolution and transaction execution.

### Current Symptoms:
1. e2e_transfer.rs, e2e_swap.rs, and e2e_lend.rs are still using mainnet RPC URL in ContextResolver
2. Only e2e_multi_step.rs and e2e_rig_agent.rs have been updated to use SURFPOOL
3. This creates inconsistent behavior where some tests work and others don't

### Root Cause:
The context resolver in several tests is initialized with:
```rust
let context_resolver = ContextResolver::new(SolanaEnvironment {
    rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
});
```

But transactions are executed through SURFPOOL at http://localhost:8899.

### Tasks Required:
1. Update all remaining e2e tests to use SURFPOOL URL for context resolver
2. Ensure consistent token setup across all tests
3. Add robust token balance verification in context resolution
4. Improve test isolation to prevent interference between tests

### Tests Status:
- e2e_transfer: ❌ Using mainnet for context resolution
- e2e_rig_agent: ✅ PASSING with SURFPOOL
- e2e_lend: ❌ Using mainnet for context resolution
- e2e_swap: ❌ Using mainnet for context resolution
- e2e_multi_step: ✅ PASSING with SURFPOOL

### Recommendation:
All e2e tests should use SURFPOOL for both context resolution and transaction execution to ensure consistency.

---

## Issue #126: Duplicated USDC Token Setup Logic Across E2E Tests (NEW)
### Status: CRITICAL CODE DUPLICATION
### Description:
The AI has added duplicated USDC token setup logic across multiple e2e tests without checking existing code, creating maintenance issues and inconsistent behavior.

### Problem Areas:
1. In `e2e_transfer.rs` (lines 44-52):
```rust
// If using SURFPOOL (default), ensure USDC tokens are set up for test
if std::env::var("SURFPOOL_RPC_URL").unwrap_or_default() == "http://localhost:8899" {
    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Set up USDC tokens in SURFPOOL for the test
    let test_pubkey = get_test_keypair()?.pubkey().to_string();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    surfpool_client
        .set_token_account(
            &test_pubkey,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            100_000_000, // 100 USDC
        )
        .await?;
}
```

2. In `e2e_rig_agent.rs` (lines 42-56):
```rust
// If using SURFPOOL (default), ensure USDC tokens are set up for test
if std::env::var("SURFPOOL_RPC_URL").unwrap_or_default() == "http://localhost:8899" {
    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Set up USDC tokens in SURFPOOL for the test
    let test_pubkey = get_test_keypair()?.pubkey().to_string();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    surfpool_client
        .set_token_account(
            &test_pubkey,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            100_000_000, // 100 USDC
        )
        .await?;
}
```

3. In `e2e_swap.rs` (lines 72-81):
```rust
// If using SURFPOOL (default), ensure USDC tokens are set up for test
if std::env::var("SURFPOOL_RPC_URL").unwrap_or_default() == "http://localhost:8899" {
    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Set up USDC tokens in SURFPOOL for the test
    let test_pubkey = get_test_keypair()?.pubkey().to_string();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    surfpool_client
        .set_token_account(
            &test_pubkey,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            100_000_000, // 100 USDC
        )
        .await?;
}
```

4. In `e2e_lend.rs` (lines 50-60):
```rust
// If using SURFPOOL (default), ensure USDC tokens are set up for test
if std::env::var("SURFPOOL_RPC_URL").unwrap_or_default() == "http://localhost:8899" {
    // Set up USDC tokens in SURFPOOL for the test
    let test_pubkey = get_test_keypair()?.pubkey().to_string();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    surfpool_client
        .set_token_account(
            &test_pubkey,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            100_000_000, // 100 USDC
        )
        .await?;
}
```

### Root Cause:
The AI added this logic to multiple tests without checking if setup_wallet_for_swap/ lend functions in common.rs already handle USDC token setup. This creates:
1. Code duplication across multiple files
2. Inconsistent amounts (100 USDC hardcoded everywhere)
3. Maintenance burden when changes are needed
4. Potential conflicts between setup functions

### Correct Solution:
1. Remove all duplicated USDC token setup logic from individual test files
2. setup_wallet_for_swap already sets up 100 USDC in common.rs (lines 85-97)
3. setup_wallet_for_lend already sets up 200 USDC in common.rs (lines 117-129)
4. For transfer tests, add a new setup_wallet_for_transfer function or use existing functions

### Priority Fixes:
1. **IMMEDIATE**: Remove duplicated USDC setup code from all e2e test files
2. **HIGH**: Remove all duplicated USDC setup code from test files
3. **MEDIUM**: Add setup_wallet_for_transfer function in common.rs for transfer tests

---

## Issue #125: Fix E2E Tests to Align with V3 Architecture (NEW)
### Status: CRITICAL IMPLEMENTATION NEEDS
### Description:
After comprehensive analysis of all e2e tests, several critical misalignments with V3 architecture have been identified that need immediate fixes.

### Test-by-Test Analysis:

#### e2e_transfer.rs
**Issues:**
- Uses mainnet RPC URL for context resolution while transactions execute on SURFPOOL
- Creates custom YML prompt manually instead of using planner's flow generation

**Fixes Needed:**
1. Update context resolver to use SURFPOOL URL (http://localhost:8899)
2. Remove manual YML prompt creation, use planner.generate_flow() instead

#### e2e_rig_agent.rs
**Issues:**
- Uses mainnet RPC URL for context resolution while transactions execute on SURFPOOL

**Fixes Needed:**
1. Update context resolver to use SURFPOOL URL (http://localhost:8899)

#### e2e_swap.rs
**Issues:**
- Uses mainnet RPC URL for context resolution while transactions execute on SURFPOOL
- Creates custom YML prompt manually instead of using planner's flow generation

**Fixes Needed:**
1. Update context resolver to use SURFPOOL URL (http://localhost:8899)
2. Remove manual YML prompt creation, use planner.generate_flow() instead

#### e2e_lend.rs
**Issues:**
- Uses mainnet RPC URL for context resolution while transactions execute on SURFPOOL
- Creates custom YML prompt manually instead of using planner's flow generation

**Fixes Needed:**
1. Update context resolver to use SURFPOOL URL (http://localhost:8899)
2. Remove manual YML prompt creation, use planner.generate_flow() instead

#### e2e_multi_step.rs
**Issues:**
- Correctly uses SURFPOOL for context resolution, but test relies on the broken multi-step implementation

**Fixes Needed:**
1. Update test expectations once multi-step execution is fixed
2. Add validation for all operations in multi-step flows

### Root Cause Analysis:
1. **Inconsistent Context Resolution**: Some tests use mainnet RPC while executing on SURFPOOL, causing mismatches
2. **Manual YML Creation**: Tests manually create YML prompts instead of using planner's flow generation
3. **Broken Multi-Step Flow**: Multi-step operations only execute first operation due to fundamental issue in RigAgent

### V3 Architecture Compliance Requirements:
1. **Consistent Environment**: All tests must use SURFPOOL for both context resolution and transaction execution
2. **LLM-Driven Flow**: Tests must use planner.generate_flow() for flow generation, not manual YML creation
3. **Proper Multi-Step**: Multi-step tests must validate that ALL operations are executed, not just the first

### Implementation Plan:

#### Phase 1: Fix Critical Code Duplication and Architecture Violations

**1. Fix Issue #126: Remove Duplicated USDC Token Setup Logic (IMMEDIATE)**
```rust
// In e2e_transfer.rs, e2e_rig_agent.rs, e2e_swap.rs, e2e_lend.rs
// REMOVE all duplicated USDC setup code blocks:
/*
if std::env::var("SURFPOOL_RPC_URL").unwrap_or_default() == "http://localhost:8899" {
    // Ensure SURFPOOL is running
    ensure_surfpool_running().await?;

    // Set up USDC tokens in SURFPOOL for the test
    let test_pubkey = get_test_keypair()?.pubkey().to_string();
    let surfpool_client = jup_sdk::surfpool::SurfpoolClient::new("http://localhost:8899");
    surfpool_client
        .set_token_account(
            &test_pubkey,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            100_000_000, // 100 USDC
        )
        .await?;
}
*/

// RELY on existing setup_wallet_for_swap/lend functions in common.rs instead
// These functions already handle USDC setup correctly:
// - setup_wallet_for_swap: 100 USDC (common.rs lines 85-97)
// - setup_wallet_for_lend: 200 USDC (common.rs lines 117-129)
```

**2. Fix Issue #122: Remove Rule-Based Multi-Step Detection (IMMEDIATE)**
```rust
// In unified_flow_builder.rs, replace rule-based detection with LLM-only approach
// REMOVE these lines:
let is_multi_step = prompt_lower.contains(" then ")
    || prompt_lower.contains(" and ")
    || prompt_lower.contains(" followed by ")
    || (prompt_lower.contains("swap") && prompt_lower.contains("lend"))
    || (prompt_lower.contains("swap") && prompt_lower.contains("transfer"))
    || (prompt_lower.contains("lend") && prompt_lower.contains("transfer"));

// REPLACE with LLM-based detection or always single-step approach
// as per V3 plan where RigAgent handles all operations
```

**2. Fix Issue #121: Multi-Step Execution (IMMEDIATE)**
```rust
// In rig_agent/mod.rs, update prompt_agent function to handle multi-step operations
// ADD explicit instructions to LLM to identify and execute ALL operations
system_prompt = "You are an AI assistant for Solana DeFi operations. 
IMPORTANT: For prompts with multiple operations (e.g., 'swap then lend'), 
you must identify and execute ALL operations in sequence, not just the first one."
```

#### Phase 2: Align E2E Tests with V3 Architecture

**3. Update Context Resolution to Use SURFPOOL (HIGH PRIORITY)**
```rust
// In e2e_transfer.rs, e2e_swap.rs, e2e_lend.rs
// REPLACE:
let context_resolver = ContextResolver::new(SolanaEnvironment {
    rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
});

// WITH:
let context_resolver = ContextResolver::new(SolanaEnvironment {
    rpc_url: Some("http://localhost:8899".to_string()),
});
```

**4. Remove Manual YML Creation (MEDIUM PRIORITY)**
```rust
// In all e2e test files
// REMOVE manual YML prompt creation:
let wallet_info = format!(
    "subject_wallet_info:\n  - pubkey: \"{from_pubkey}\"\n    lamports: {initial_sol_balance} # {formatted_balance} SOL\n    total_value_usd: 170\n\nsteps:\n  prompt: \"{prompt}\"\n    intent: \"send\"\n    context: \"Executing a SOL transfer using Solana system instructions\"\n    recipient: \"{TARGET_PUBKEY}\""
);

// REPLACE with planner-generated flow:
let flow = planner
    .refine_and_plan(prompt, &from_pubkey.to_string())
    .await?;
```

#### Phase 3: Validation and Testing

**5. Add Multi-Step Operation Validation**
```rust
// In e2e_multi_step.rs
// ADD validation for all operations:
fn validate_multi_step_execution(result: &FlowResult) -> Result<()> {
    // Verify swap operation was executed
    let swap_executed = result.step_results.iter().any(|r| 
        r.tool_calls.contains(&"jupiter_swap".to_string())
    );
    
    // Verify lend operation was executed
    let lend_executed = result.step_results.iter().any(|r| 
        r.tool_calls.contains(&"jupiter_lend_earn_deposit".to_string())
    );
    
    if !swap_executed || !lend_executed {
        return Err(anyhow!("Not all operations in multi-step flow were executed"));
    }
    
    Ok(())
}
```

### Priority Fixes:
1. **CRITICAL**: Fix Issue #122 (remove rule-based multi-step detection)
2. **CRITICAL**: Fix Issue #121 (multi-step execution)
3. **HIGH**: Update all e2e tests to use SURFPOOL for context resolution
4. **MEDIUM**: Remove manual YML creation from tests, use planner.generate_flow()
5. **MEDIUM**: Add validation for all operations in multi-step flows

---

## Issue #110: Remove Unused Code (NOT STARTED)
### Status: NOT STARTED
### Description:
Clean up deprecated functions in YmlGenerator following V3 plan guidelines.

### Tasks Required:
1. Remove rule-based operation parsing functions
2. Keep builder functions for testing only
3. Update documentation to reflect current implementation

---

## Issue #102: Error Recovery Engine (NOT STARTED)
### Status: NOT STARTED
### Description:
Error recovery is incomplete. Need retry logic and alternative paths for failed transactions.

### Tasks Required:
1. Implement retry with exponential backoff for network errors
2. Add alternative path selection for transaction failures
3. Implement slippage adjustment for swap failures
4. Create error classification system

---

## Issue #105: RigAgent Enhancement (PARTIALLY COMPLETED)
### Status: PARTIALLY COMPLETED
### Description:
RigAgent needs improved multi-step operation handling.

### Tasks Remaining:
1. Fix multi-step execution to handle all operations in prompt
2. Improve parameter validation before tool execution
3. Better prompt engineering for complex tool selection

---

## Issue #106: LanguageRefiner Improvement (PARTIALLY COMPLETED)
### Status: PARTIALLY COMPLETED
### Description:
LanguageRefiner needs better context awareness and multi-language support.

### Tasks Remaining:
1. Context awareness integration
2. Multi-language support implementation
3. Refinement quality metrics

---

## Issue #112: Comprehensive Error Recovery (NOT STARTED)
### Status: NOT STARTED
### Description:
Implement robust error recovery with different strategies based on error type.

### Tasks Required:
1. Create error classification system
2. Implement retry logic with backoff
3. Add circuit breaker for repeated failures
4. Add recovery metrics and logging

---

### Implementation Priority

### Week 1:
1. ~~Issue #124: SURFPOOL Context Resolution Affects Tool Selection (COMPLETED)~~
2. Issue #122: Rule-Based Multi-Step Detection Contradicts V3 Architecture (CRITICAL)
3. Issue #121: Multi-Step Operations Not Properly Executed (CRITICAL)
4. Issue #110: Remove Unused Code (NOT STARTED)

### Week 2:
4. Issue #102: Error Recovery Engine (NOT STARTED)
5. Issue #112: Comprehensive Error Recovery (NOT STARTED)

### Current State Summary:
E2e tests status:
- ✅ swap operations (including "sell all SOL")
- ✅ transfer operations
- ✅ lend operations (single-step only)
- ❌ multi-step operations (only first operation executes)

Architecture alignment with V3 plan:
- ✅ RigAgent handles tool selection based on refined prompts
- ✅ Balance validation integrated and working
- ❌ CRITICAL VIOLATION: Rule-based multi-step detection contradicts V3 architecture
- ❌ Multi-step operations broken - critical issue

### Critical Implementation Note:
-Issue #122 must be addressed before any other multi-step related fixes. The rule-based approach fundamentally violates the V3 architecture and will cause continuous conflicts with the intended LLM-driven approach.
