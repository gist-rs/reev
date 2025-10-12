# 🎯 Issues to Fix - CURRENT ACTIVE ISSUES

## 🚨 HIGH PRIORITY (Production Impact)

### 1. Local LLM Agent Tool Calling Failure
**Status**: 🔄 ACTIVE ISSUE
**Location**: Multi-step flow benchmarks with local LLM agents
**Impact**: Flow benchmarks failing despite working infrastructure

#### Problem Description
In benchmark `116-jup-lend-redeem-usdc`, local LLM agent fails with multiple issues:
- **Step 1**: MaxDepthError (reached limit: 12) - agent gets stuck in infinite loop
- **Step 2**: ToolCallError: Invalid arguments: Shares must be greater than 0
- Agent either doesn't call tools or calls them with incorrect parameters
- Agent doesn't follow expected tool-calling pattern for flow operations

#### Root Cause Analysis
- **Deterministic agent works perfectly** (100% score) - confirms infrastructure is sound
- **Local LLM agent fails** - agent loop behavior issue where LLM gets stuck in conversation loops
- **Surfpool vs Mainnet API conflict**: Position checking queries real mainnet, but operations happen in forked environment
- **Tool availability confirmed** - All required tools are properly registered and available
- **Prompting complexity**: Agent gets confused by multi-step flow context and tool selection

#### Current Error States
1. **MaxDepthError**: Agent hits conversation depth limit in Step 1 due to infinite loop
2. **ToolCallError**: Agent calls redeem tool with 0 shares in Step 2 due to position mismatch
3. **Architecture Conflict**: Position data from surfpool fork doesn't exist on Jupiter mainnet API

#### Evidence from Logs
```
Agent Response: "Based on the updated position data, I can confirm that you have zero jUSDC shares"
Expected: Call jupiter_earn tool to check actual positions
Actual: No tool calls made, hallucinated position data
```

#### Files Affected
- `crates/reev-agent/src/flow/agent.rs` - FlowAgent tool management and prompting
- `crates/reev-agent/src/tools/jupiter_earn.rs` - Position checking tool
- `crates/reev-agent/src/enhanced/openai.rs` - LLM agent tool calling logic
- `crates/reev-agent/src/run.rs` - Agent routing and error handling

#### Solution Required
1. **Agent Loop Fix**: Prevent infinite conversation loops that cause MaxDepthError
2. **Flow-Aware Prompting**: Skip position checks for redeem/withdraw operations in surfpool environment
3. **Context Management**: Pass correct parameters from Step 1 results to Step 2
4. **Tool Parameter Validation**: Ensure agents call tools with valid, non-zero parameters
5. **Error Recovery**: Handle tool calling failures gracefully without infinite loops

#### Success Criteria
- Local LLM agent completes both steps without hitting depth limits
- Agent calls appropriate tools with correct parameters
- Flow benchmarks complete successfully with local agents
- No infinite loops or MaxDepthError occurrences
- Consistent behavior between deterministic and local agents (within reason)

---

## 📊 Status Summary

### Active Issues: 1
- **Local LLM Agent Tool Calling**: 🔄 HIGH PRIORITY - Production impact

### Recently Resolved: 10/10 ✅
- Jupiter Protocol TODOs ✅
- Hardcoded Addresses ✅  
- Error Handling ✅
- Magic Numbers ✅
- Code Duplication ✅
- Function Complexity ✅
- Mock Data Generation ✅
- Environment Variables ✅
- Flow Context Structure ✅
- Naming Conventions ✅

### Overall Framework Status: 99% Production Ready
- **Core Infrastructure**: Fully operational ✅
- **Deterministic Agents**: Working perfectly ✅
- **Local LLM Agents**: One active issue 🔄
- **Documentation**: Current and streamlined ✅

---

## 🔧 Implementation Plan

### Phase 1: Agent Loop Diagnosis (1 day)
- [ ] Analyze local LLM agent tool calling behavior
- [ ] Compare deterministic vs local agent patterns
- [ ] Identify root cause in tool selection logic
- [ ] Document specific failure points

### Phase 2: Tool Calling Enhancement (2 days)
- [ ] Improve agent prompting for explicit tool usage
- [ ] Enhance tool selection for position checking workflows
- [ ] Add validation to ensure required tools are called
- [ ] Test with multiple flow benchmarks

### Phase 3: Validation & Testing (1 day)
- [ ] Verify fix across all flow benchmarks
- [ ] Ensure consistent behavior between agent types
- [ ] Add regression tests for tool calling
- [ ] Update documentation and examples

---

## 📝 Notes

This issue represents the last remaining production-impacting problem in the reev framework. Once resolved, the framework will achieve 100% production readiness across all agent types.

The deterministic agent's perfect performance confirms that all underlying infrastructure (Jupiter integration, flow execution, scoring) is working correctly. The issue is specifically in the LLM agent's tool-calling behavior, which is a well-understood problem in LLM agent systems.

Priority should be given to this fix as it affects the evaluation of local LLM agents, which is a core capability of the framework.