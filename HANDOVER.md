CLI: amount=394358118 (394.358 USDC) → ✅ SUCCESS
API: amount=1000000000 (1000 USDC) → ❌ INSUFFICIENT FUNDS  
```

---

## 🎯 **ROOT CAUSE IDENTIFIED**

**Two different Jupiter swap tool implementations** with inconsistent response formats:

1. **`jupiter_swap_flow.rs`** - Flow-aware tool with proper `swap_details`
2. **`jupiter_swap.rs`** - Standard tool without `swap_details`

**Critical Logic Flow**:
- CLI path routes to flow-aware tool
- API path routes to standard tool
- Both should use same tool for consistency

---

## 📋 **INVESTIGATION FINDINGS**

### Files Involved

#### **Primary Issue**:
- **`crates/reev-tools/src/tools/jupiter_swap_flow.rs`** - Flow-aware implementation
- **`crates/reev-tools/src/tools/jupiter_swap.rs`** - Standard implementation  
- **`crates/reev-context/src/lib.rs`** - Expects `swap_details` in `process_step_result_for_context()`

#### **Secondary Issue**:
- Tool routing logic in agent selection
- API vs CLI execution path differences

---

## 🛠️ **FIX STRATEGY**

### **Option A: Tool Unification** (Recommended)
- Merge both implementations into single flow-aware Jupiter swap tool
- Ensure consistent `swap_details` response format
- Update tool registration to use unified tool only

### **Option B: Response Format Standardization**
- Modify `jupiter_swap.rs` to also return `swap_details` structure
- Ensure both tools provide same data format

### **Option C: Routing Fix** (Quick)
- Force API path to use `jupiter_swap_flow.rs` like CLI
- Update tool discovery logic to prefer flow-aware tools

---

## 🧪 **NEXT STEPS**

### **Immediate Priority**:
1. **Investigate routing logic** - Why do CLI and API use different tools?
2. **Compare response formats** - Document exact structure differences
3. **Implement unification** - Choose Option A or B based on complexity

### **Acceptance Criteria**:
- [ ] CLI path continues working (no regression)
- [ ] API path achieves same success rate as CLI
- [ ] Both paths use identical Jupiter swap tool
- [ ] Step 2 consistently receives swap result data
- [ ] No performance regression
-   [ ] Comprehensive testing across multiple benchmarks
-   [ ] Unified tool response format for Jupiter swap tools
-   [ ] CLI path preserved (no regression)
-   [ ] API path achieves same success rate as CLI

---

## 🔍 **DEBUGGING CHECKLIST**

### **For Investigation**:
- [ ] Compare actual tool calls in CLI vs API logs
- [ ] Verify tool registration lists in both paths  
- [ ] Check response parsing in `process_step_result_for_context()`
- [ ] Test with different agents to isolate routing vs response format

### **For Implementation**:
- [ ] Preserve CLI functionality completely
- [ ] Create comprehensive tests for both paths
- [ ] Update documentation for tool usage patterns
- [ ] Verify no regression in other benchmarks

---

## 💡 **TECHNICAL NOTES**

### **Context Flow**:
```
Step 1: Jupiter Swap → swap_details.output_amount
Step 2: Context Building → process_step_result_for_context() → Correct Amount
```

### **Critical Method**:
- **`process_step_result_for_context()`** in `crates/reev-context/src/lib.rs` (lines 290-320)
- Expects: `step_result.get("swap_details").get("output_amount")`
- CLI receives: ✅ Structured data  
- API receives: ❌ Missing structured data

---

## 📞 **CONTACT POINT**

This is **tool consistency issue**, not LLM context problem. Focus on:

1. **Tool routing unification** between CLI and API paths
2. **Response format standardization** for Jupiter swap tools  
3. **Preservation of working CLI functionality**

+-   [ ] Do not modify working CLI path** - fix API path to match CLI success pattern.
+-   [ ] Preserve CLI functionality completely during unification
+-   [ ] No performance regression in existing flows
+-   [ ] Tool consistency check for other tools (spl_transfer, sol_transfer, jupiter_lend_earn_deposit)
+-   [ ] Comprehensive testing across benchmarks and agents
+-   [ ] Documentation updated with tool usage guidelines

---

**Handover Complete** - Ready for next engineer to implement unification fix.</arg_value>
</tool_call>