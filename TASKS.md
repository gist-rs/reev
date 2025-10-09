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

**Problem Identified**
- LLM currently calls `jupiter_earn.positions` to check before acting (smart but consumes depth)
- All account balance and token information is available in benchmark YAML files
- LLM shouldn't need to "discover" information that's already provided in setup
- Need two scenarios: (1) with context for direct action, (2) without extended depth for discovery

**Core Requirements**
1. **Prerequisite Validation**: LLM must receive wallet/account balance info in context
2. **Direct Action**: If prerequisites match request (e.g., transfer 1 SOL when user has 1 SOL), execute directly
3. **Discovery Tools**: If context is incomplete, LLM must use provided tools to query account information
4. **Agent Library**: Must provide tools for balance/position discovery when context is insufficient

### **Context Enhancement Tasks**

**Phase 5.1: Parse Account Information from YAML**
- [x] Create context builder module: `crates/reev-agent/src/context/`
- [x] Parse `initial_state` from benchmark YAML files
- [x] Extract token account balances and positions
- [x] Format account information for LLM context

**Phase 5.2: Context Integration**
- [x] Update enhanced agents to include context in system prompt
- [x] Format as structured account information:
  ```
  USER_WALLET_PUBKEY: 5 SOL balance
  USER_USDC_ATA: 100 USDC balance  
  USER_L_USDC_ATA: 50 L-USDC shares (Solend position)
  ```
- [x] Include token mint addresses and amounts

**Phase 5.3: Prerequisite Validation & Discovery Tools**
- [ ] **Balance Validation**: Implement logic to check if requested amount is available
- [x] **Discovery Tools**: Create tools for querying account balances and positions
- [x] **Agent Library Tools**: Add `get_account_balance`, `get_position_info`, and `get_lend_earn_tokens` tools
- [x] **Conditional Logic**: LLM should use context if available, otherwise call discovery tools
- [ ] **Validation Check**: Before execution, verify prerequisites are met

**Phase 5.4: Two-Tier Depth Strategy**
- [x] **With Context**: Use normal depth (3) when account info provided
- [x] **Without Context**: Use extended depth (5-7) for discovery scenarios
- [x] Detect context availability and adjust conversation depth accordingly
- [ ] Create benchmarks for both scenarios to validate approach

**Phase 5.5: Smart Tool Selection**
- [ ] Update tool descriptions to reference available context
- [ ] Add "if you don't see account info below, check balances first"
- [ ] Optimize LLM decision-making based on provided context
- [ ] Add prerequisite validation prompts in system messages

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
- [x] Test new context-enhanced LLM agents
- [x] Validate both depth strategies work correctly
- [ ] Test prerequisite validation logic
- [x] Test discovery tools when context is insufficient
- [ ] Compare performance: with vs without context
- [x] Ensure no regressions in existing benchmarks
- [ ] Create benchmarks for both scenarios (with/without context)

**Phase 7: Documentation Updates**
- [ ] Update `TOFIX.md` with context enhancement results
- [ ] Update `REFLECT.md` with LLM behavior insights
- [ ] Document prerequisite validation strategy
- [ ] Document discovery tools usage patterns
- [ ] Create best practices guide for context design

---
+
## 🎯 **NEW EXPECTED OUTCOMES**

### **After Context Enhancement:**
- **With Context**: Direct action after prerequisite validation, minimal tool calls, high success rates
- **Without Context**: Discovery tools to gather prerequisites, then execute, robust fallback behavior  
- **Intelligent Adaptation**: LLM checks context first, uses discovery tools if needed
- **Prerequisite Validation**: Always verify sufficient balance/position before execution
- **Improved Efficiency**: 60-80% reduction in unnecessary API calls
- **Better Conversations**: More natural LLM interactions with financial context

### **Performance Targets:**
- Direct action scenarios: 95%+ success with 1-2 tool calls (validation only)
- Discovery scenarios: 85%+ success with 3-5 tool calls (discovery + execution)
- Overall benchmark improvement: 15-25% higher scores
- Reduced MaxDepthError instances: 90% reduction
- Prerequisite validation success: 100% (reject invalid requests early)

---

### **Key Principle Evolved:**
Always validate prerequisites first. Use context if available, otherwise use discovery tools to gather required information before executing any financial operation.