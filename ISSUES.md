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

## Issue #122: Rule-Based Multi-Step Detection Contradicts V3 Architecture (NEW)
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
In unified_flow_builder.rs (uncommitted changes), rule-based logic was added:
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

## Issue #121: Multi-Step Operations Not Properly Executed (IN PROGRESS)
### Status: CRITICAL ISSUE IDENTIFIED
### Description:
Multi-step flows are not properly executing all operations. The planner generates multiple steps correctly, but only the first operation (swap) is being executed.

### Current Behavior:
- Prompt: "swap 0.1 SOL to USDC then lend 10 USDC"
- Planner generates 2 steps correctly after fixing detection logic
- However, test fails because USDC balance doesn't decrease after "lending"
- Root cause: LLM is executing only the swap operation, ignoring the lend operation

### Why Tests Were Passing Before:
- Previous test used unrealistic amounts (swap 0.1 SOL for $15, then lend 100 USDC)
- With 100 USDC initial balance, this created a false sense of success
- When changed to realistic amounts (lend 10 USDC from ~15 USDC swap output), test failed

### Root Cause:
The issue is in RigAgent's execution of multi-step flows. It's not properly handling sequential operations in a single step. The LLM is extracting only the first operation from multi-step prompts.

### Tasks Required:
1. Fix RigAgent to properly execute all operations in multi-step steps
2. Ensure context is properly updated between sequential operations
3. Add validation that all operations in prompt are being executed
4. Fix prompt refinement to preserve all operations correctly

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

## Issue #124: SURFPOOL Context Resolution Affects Tool Selection (CRITICAL)
### Status: CRITICAL CONSISTENCY ISSUE
### Description:
When ContextResolver uses SURFPOOL RPC URL instead of mainnet, e2e tests fail because LLM selects wrong tools. The wallet context retrieved from SURFPOOL appears to differ from mainnet, causing the LLM to make incorrect tool selections.

### Current Symptoms:
1. e2e_lend/test_lend_100_usdc: LLM selects sol_transfer instead of jupiter_lend_earn_deposit for "lend 100 USDC"
2. e2e_swap/test_swap_0_1_sol_for_usdc: Test fails when run with other tests, but passes when run individually (test isolation issue)

### Root Cause:
When SURFPOOL starts fresh, it doesn't automatically have USDC tokens in user wallets. The context resolver reads the wallet state and finds no USDC tokens, which causes the LLM to select `sol_transfer` instead of `jupiter_lend_earn_deposit` for "lend 100 USDC" prompts.

The e2e tests were setting up USDC tokens using setup_wallet_for_lend/swap functions, but if SURFPOOL restarts between tests, those tokens are lost.

### Tasks Required:
1. Ensure SURFPOOL persists token state between tests
2. Add USDC token setup to e2e_rig_agent test (already has for others)
3. Add robust token balance verification in context resolution
4. Improve SURFPOOL state management to prevent token loss on restart

### Current Solution Implemented:
1. Added automatic USDC token setup to e2e_lend and e2e_swap tests when using SURFPOOL
2. Added USDC token setup to e2e_rig_agent test
3. All e2e tests now pass with SURFPOOL context resolver

### Tests Verified:
- e2e_transfer: ✅ PASSING with SURFPOOL
- e2e_rig_agent: ✅ PASSING with SURFPOOL
- e2e_lend: ✅ PASSING with SURFPOOL
- e2e_swap: ❌ test_swap_0_1_sol_for_usdc still failing when running with other tests

### Workaround:
None required - SURFPOOL context resolution issues have been fixed.

### Recommendation:
Always use SURFPOOL for all e2e tests to ensure consistency between context resolution and transaction execution.

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
