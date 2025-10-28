---

## 🚨 **CURRENT INVESTIGATION: API vs CLI Flow Execution Issue**

### **Status**: 🔄 IN PROGRESS (Active Investigation)

#### **Problem Statement**:
- **CLI Path**: `cargo run --benchmarks/200-jup-swap-then-lend-deposit.yml --agent glm-4.6` ✅ WORKS
- **API Path**: Web UI run with `glm-4.6` agent ❌ FAILS on Step 2
- **Both paths call**: `reev_runner::run_benchmarks()` → `run_flow_benchmark()` → `LlmAgent`

#### **Current Bug Evidence**:
```
Step 1 (CLI): Jupiter swap → receives 394358118 USDC ✅
Step 1 (API): Jupiter swap → receives 394358118 USDC ✅

Step 2 (CLI): "Deposit all USDC received" → uses correct amount ✅  
Step 2 (API): "Deposit all USDC received" → amount=0 ❌
```

#### **Critical Finding**:
Both CLI and API use **same function** but behave differently for **step result context passing**.

---

## 🔍 **ROOT CAUSE IDENTIFIED**

### **Missing Step Context Enrichment**:

**Location**: `crates/reev-runner/src/lib.rs` - `run_flow_benchmark()` function

**Bug**: Step 2 prompt is not enriched with Step 1 results:
```rust
let step_test_case = TestCase {
    id: format!("{}-step-{}", test_case.id, step.step),
    description: step.description.clone(),
    tags: test_case.tags.clone(),
    initial_state: test_case.initial_state.clone(),
    prompt: step.prompt.clone(), // ← Original prompt only, NO Step 1 context!
    flow: None,
    ground_truth: test_case.ground_truth.clone(),
};
```

**Expected**: Step 2 should receive enriched prompt with swap amount from Step 1  
**Actual**: Step 2 receives original prompt only (no swap amount context)

#### **Why CLI Works vs API Fails**:
- **Same function**: Both call `run_flow_benchmark()` identically
- **Different environment**: Some execution context affects step result communication
- **Missing piece**: Step result → context → next step prompt enrichment

---

## 🛠️ **INVESTIGATION STATUS**

### **Confirmed Issues**:
1. ✅ **API self-killing**: Fixed with `kill_api=false`
2. ❌ **Step context loss**: Still investigating root cause
3. ✅ **Same function execution**: Confirmed CLI and API use identical code path
4. ❌ **Environment differences**: Need to identify what affects context passing

### **Key Files Under Investigation**:
- `crates/reev-runner/src/lib.rs` - `run_flow_benchmark()` step execution logic
- `crates/reev-agent/src/enhanced/common/mod.rs` - Tool selection and flow mode detection
- `crates/reev-context/src/lib.rs` - Context building with step results

### **Critical Code Path**:
```
Step 1: run_evaluation_loop() → LlmAgent → Jupiter tool → blockchain state update
Step 2: run_evaluation_loop() → LlmAgent → Should get Step 1 context ❌
```

---

## 🎯 **NEXT INVESTIGATION STEPS**

### **Immediate Priority**:
1. **Compare environment variables** between CLI and API execution
2. **Trace step result serialization** - Is Step 1 result properly captured?
3. **Verify observation building** - Does Step 2 prompt include Step 1 context?
4. **Test with deterministic agent** - Is issue agent-specific or universal?

### **Hypothesis**:
- **A. Session handling differences**: API vs CLI create different session contexts
- **B. Database state differences**: Flow logging persistence affects context building  
- **C. Tool selection differences**: Flow mode detection works differently
- **D. Agent initialization differences**: LlmAgent setup varies by execution path

---

## 📋 **DELIVERABLES NEEDED**

### **For Complete Fix**:
1. **Root cause analysis**: Identify exact difference between CLI and API execution
2. **Step context enrichment**: Ensure Step 2 receives Step 1 swap amount  
3. **Unified behavior**: Make API and CLI produce identical results
4. **Comprehensive testing**: Verify fix across all agents (local, glm-4.6, etc.)

---

## 🔍 **DEBUGGING APPROACH**

### **Step-by-Step Investigation**:
1. **Instrument step execution**: Add logging for prompt content and context building
2. **Compare raw requests**: Verify what agent actually receives in Step 2 for CLI vs API
3. **Trace result serialization**: Check if Step 1 results are properly stored and retrieved
4. **Validate context injection**: Ensure swap amounts flow between steps correctly

### **Key Questions to Answer**:
- Why does same `run_flow_benchmark()` behave differently?
- What environment/context affects step result communication?
- How does CLI successfully pass Step 1 results to Step 2 when API doesn't?
- Is the bug in prompt enrichment, context building, or result serialization?

---

## 💡 **TECHNICAL NOTES**

### **Expected Flow**:
```
Step 1: "Swap 2.0 SOL for USDC" → Jupiter swap → 394358118 USDC
Step 2: "Deposit [394358118] USDC into Jupiter" → Jupiter deposit → SUCCESS
```

### **Current Broken Flow**:
```
Step 1: "Swap 2.0 SOL for USDC" → Jupiter swap → 394358118 USDC ✅
Step 2: "Deposit USDC into Jupiter" → Jupiter deposit → amount=0 → INSUFFICIENT FUNDS ❌
```

### **Critical Method**:
- **Step result context building** between flow execution steps
- **Agent prompt enrichment** with previous step results  
- **Context serialization/deserialization** across step boundaries
- **Environment state preservation** between multi-step flows

---

## 🧪 **HANDOFF CHECKLIST**

### **Complete When**:
- [ ] Root cause identified (environment/session/tooling/context difference)
- [ ] Step context enrichment implemented and tested
- [ ] CLI and API produce identical execution results
- [ ] Fix verified across all agents (local, glm-4.6, glm-4.6-coding)
- [ ] No regression in single-step benchmarks
- [ ] Documentation updated with flow execution behavior
- [ ] Comprehensive test coverage added for multi-step flows

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