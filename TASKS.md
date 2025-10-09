# 🎯 Jupiter Tools Refactoring Plan

## 🚨 **PROBLEM IDENTIFIED**
- Current tools marked as "DEPRECATED" causing LLM confusion and MaxDepthError
- 4 distinct Jupiter API endpoints incorrectly mapped to 2 tools
- User intent mismatch between natural language and tool selection

## 📋 **IMPLEMENTATION TASKS**

### **Phase 1: Rename Tools (API Route Mapping)**
**Goal**: Align tool names with Jupiter API routes for clarity

**File Renames:**
- [ ] `jupiter_lend_deposit.rs` → `jupiter_lend_earn_deposit.rs`
- [ ] `jupiter_lend_withdraw.rs` → `jupiter_lend_earn_withdraw.rs`  
- [ ] `jupiter_mint_redeem.rs` → `jupiter_lend_earn_mint_redeem.rs` (or split into 2 files)

**Tool Renames:**
- [ ] `JupiterLendDepositTool` → `JupiterLendEarnDepositTool`
- [ ] `JupiterLendWithdrawTool` → `JupiterLendEarnWithdrawTool`
- [ ] `JupiterMintTool` → `JupiterLendEarnMintTool`
- [ ] `JupiterRedeemTool` → `JupiterLendEarnRedeemTool`

**Tool Constants:**
- [ ] `jupiter_lend_deposit` → `jupiter_lend_earn_deposit`
- [ ] `jupiter_lend_withdraw` → `jupiter_lend_earn_withdraw`
- [ ] `jupiter_mint` → `jupiter_lend_earn_mint`
- [ ] `jupiter_redeem` → `jupiter_lend_earn_redeem`

### **Phase 2: Update Tool Descriptions (Remove DEPRECATED)**
**Goal**: Clear, distinct descriptions that match user intent

**jupiter_lend_earn_deposit:**
```rust
description: "Deposit tokens into Jupiter lending to earn yield. Use when user wants to 'deposit', 'lend', or 'earn yield' on a specific amount of tokens. Works with token amounts (e.g., 50000000 for 50 USDC)."
```

**jupiter_lend_earn_withdraw:**
```rust
description: "Withdraw tokens from Jupiter lending position. Use when user wants to 'withdraw', 'remove', or 'take out' a specific amount of tokens. Works with token amounts (e.g., 50000000 for 50 USDC)."
```

**jupiter_lend_earn_mint:**
```rust
description: "Mint jTokens (shares) in Jupiter lending. Use when user wants to 'mint jTokens', 'create shares', or 'mint positions' based on share count, not token amounts. Works with share counts."
```

**jupiter_lend_earn_redeem:**
```rust
description: "Redeem/burn jTokens (shares) from Jupiter lending. Use when user wants to 'redeem jTokens', 'burn shares', or 'close positions' based on share count, not token amounts. Works with share counts."
```

### **Phase 3: Update All Imports and Registrations**
**Files to Update:**
- [ ] `crates/reev-agent/src/tools/mod.rs`
- [ ] `crates/reev-agent/src/enhanced/gemini.rs`
- [ ] `crates/reev-agent/src/enhanced/openai.rs`
- [ ] `crates/reev-agent/src/flow/agent.rs`
- [ ] Any other agent implementations

### **Phase 4: Revert Benchmark Prompts to Natural Language**
**Goal**: Use natural user language instead of tool-specific terminology

**Benchmark Updates:**
- [ ] `111-jup-lend-deposit-usdc.yml`:
  - **From**: `"Mint jUSDC by depositing 50 USDC using Jupiter. My wallet is USER_WALLET_PUBKEY."`
  - **To**: `"Lend 50 USDC using Jupiter. My wallet is USER_WALLET_PUBKEY."`

- [ ] `112-jup-lend-withdraw-sol.yml`:
  - **From**: `"Redeem jSOL to withdraw 0.1 SOL using Jupiter. My wallet is USER_WALLET_PUBKEY."`
  - **To**: `"Withdraw 0.1 SOL using Jupiter. My wallet is USER_WALLET_PUBKEY."`

- [ ] `113-jup-lend-withdraw-usdc.yml`:
  - **From**: `"Redeem jUSDC to withdraw 50 USDC using Jupiter..."`
  - **To**: `"Withdraw 50 USDC from your Solend lending position using Jupiter. My wallet is USER_WALLET_PUBKEY."`

### **Phase 5: Update Protocol Handlers**
**Goal**: Ensure protocol handlers align with new tool names

**Protocol Handler Updates:**
- [ ] Check `crates/reev-agent/src/protocols/jupiter/` handlers
- [ ] Update any references to old tool names
- [ ] Ensure handlers still work with existing Jupiter API calls

### **Phase 6: Testing and Validation**
**Goal**: Ensure all benchmarks work correctly after refactoring

**Testing Tasks:**
- [ ] Run individual benchmarks:
  - `./test_local_agent.sh --local 111-jup-lend-deposit-usdc.yml`
  - `./test_local_agent.sh --local 112-jup-lend-withdraw-sol.yml`
  - `./test_local_agent.sh --local 113-jup-lend-withdraw-usdc.yml`
- [ ] Run full benchmark suite: `./test_local_agent.sh --local`
- [ ] Verify all ERROR benchmarks are resolved
- [ ] Check that no regressions in working benchmarks

### **Phase 7: Documentation Updates**
**Goal**: Update documentation to reflect new naming

**Documentation Updates:**
- [ ] Update `TOFIX.md` with new fix details
- [ ] Update `REFLECT.md` with this refactoring session
- [ ] Update any other documentation referencing old tool names

## 🎯 **EXPECTED OUTCOMES**

### **Before Fix:**
- 3 ERROR benchmarks due to MaxDepthError
- Confusing DEPRECATED warnings
- LLM tool selection confusion

### **After Fix:**
- 0 ERROR benchmarks
- Clear tool descriptions matching user intent
- Perfect API route mapping
- Natural language benchmark prompts
- LLM can easily match "lend" → `jupiter_lend_earn_deposit`, "withdraw" → `jupiter_lend_earn_withdraw`

## 🚀 **IMPLEMENTATION PRIORITY**

1. **HIGH**: Phase 1-2 (Tool names and descriptions) - Core issue
2. **MEDIUM**: Phase 3-4 (Imports and benchmarks) - Integration
3. **LOW**: Phase 5-7 (Handlers and docs) - Maintenance

## 📝 **NOTES**

### **Jupiter API Mapping:**
- `/earn/deposit` → `jupiter_lend_earn_deposit` (amount-based)
- `/earn/withdraw` → `jupiter_lend_earn_withdraw` (amount-based)  
- `/earn/mint` → `jupiter_lend_earn_mint` (shares-based)
- `/earn/redeem` → `jupiter_lend_earn_redeem` (shares-based)

### **User Intent Mapping:**
- "lend", "deposit", "earn yield" → `jupiter_lend_earn_deposit`
- "withdraw", "remove", "take out" → `jupiter_lend_earn_withdraw`
- "mint jtokens", "create shares" → `jupiter_lend_earn_mint`
- "redeem jtokens", "burn shares" → `jupiter_lend_earn_redeem`

### **Key Principle:**
Keep all 4 tools distinct - no deprecation, no redirects. Let LLM choose based on user language and intent.

---
+
## 🏆 **CURRENT STATUS**
+- ✅ Deterministic infrastructure complete (13/13 working)
+- ✅ Jupiter tools refactoring complete (Phases 1-4 done)
+- ✅ LLM tool selection fixed (removed MaxDepthError)
+- 🔄 Next: Context enhancement for smarter LLM decisions
+- 📋 Phase 5-7 tasks outlined below
+- 🎯 Clear implementation path forward

---

## 📋 **PHASE 5: CONTEXT ENHANCEMENT (New Requirement)**
**Goal**: Provide LLM with prerequisite wallet/account context to reduce unnecessary tool calls

### **Problem Identified**
- LLM currently calls `jupiter_earn.positions` to check before acting (smart but consumes depth)
- All account balance and token information is available in benchmark YAML files
- LLM shouldn't need to "discover" information that's already provided in setup
- Need two scenarios: (1) with context for direct action, (2) without extended depth for discovery

### **Context Enhancement Tasks**

**Phase 5.1: Parse Account Information from YAML**
- [ ] Create context builder module: `crates/reev-agent/src/context/`
- [ ] Parse `initial_state` from benchmark YAML files
- [ ] Extract token account balances and positions
- [ ] Format account information for LLM context

**Phase 5.2: Context Integration**
- [ ] Update enhanced agents to include context in system prompt
- [ ] Format as structured account information:
  ```
  USER_WALLET_PUBKEY: 5 SOL balance
  USER_USDC_ATA: 100 USDC balance  
  USER_L_USDC_ATA: 50 L-USDC shares (Solend position)
  ```
- [ ] Include token mint addresses and amounts

**Phase 5.3: Two-Tier Depth Strategy**
- [ ] **With Context**: Use normal depth (3) when account info provided
- [ ] **Without Context**: Use extended depth (5-7) for discovery scenarios
- [ ] Detect context availability and adjust conversation depth accordingly
- [ ] Create benchmarks for both scenarios to validate approach

**Phase 5.4: Smart Tool Selection**
- [ ] Update tool descriptions to reference available context
- [ ] Add "if you don't see account info below, check positions first"
- [ ] Optimize LLM decision-making based on provided context

### **Expected Benefits**
- Reduce unnecessary tool calls by 60-80%
- Higher success rates for direct action scenarios
- Better LLM understanding of user's current financial position
- Support for both informed and discovery use cases
- More natural conversation flow

### **Implementation Priority**
1. **HIGH**: Phase 5.1-5.2 (Context parsing and integration)
2. **MEDIUM**: Phase 5.3 (Two-tier depth strategy)
3. **LOW**: Phase 5.4 (Tool description optimization)

---
+
## 📋 **PHASE 6-7: COMPLETION & DOCUMENTATION**
*(Existing phases 5-7 from original plan)*

**Phase 6: Testing and Validation**
- [ ] Test new context-enhanced LLM agents
- [ ] Compare performance: with vs without context
- [ ] Validate both depth strategies work correctly
- [ ] Ensure no regressions in existing benchmarks

**Phase 7: Documentation Updates**
- [ ] Update `TOFIX.md` with context enhancement results
- [ ] Update `REFLECT.md` with LLM behavior insights
- [ ] Document two-tier depth strategy for future reference
- [ ] Create best practices guide for context design

---
+
## 🎯 **NEW EXPECTED OUTCOMES**

### **After Context Enhancement:**
- **With Context**: Direct action, minimal tool calls, high success rates
- **Without Context**: Extended depth for discovery, robust fallback behavior  
- **Intelligent Adaptation**: LLM adjusts approach based on available information
- **Improved Efficiency**: 60-80% reduction in unnecessary API calls
- **Better Conversations**: More natural LLM interactions with financial context

### **Performance Targets:**
- Direct action scenarios: 95%+ success with 1-2 tool calls
- Discovery scenarios: 85%+ success with 3-5 tool calls
- Overall benchmark improvement: 15-25% higher scores
- Reduced MaxDepthError instances: 90% reduction

---

### **Key Principle Evolved:**
Provide LLM with necessary context upfront, but build robust discovery mechanisms for when context is incomplete.