# Reev Core Architecture Plan V2

## 🎯 **Why: Fixing Critical Issues with Previous Approaches**

### **Previous Implementation Problems**:
- **Language Limitation**: Rule-based pattern matching can't handle internationalization or typos
- **No Structured Planning**: Direct tool execution without proper flow planning
- **Missing Auditability**: No verifiable, benchmarkable flow definitions
- **Inefficient LLM Usage**: Either avoiding LLMs or calling them unnecessarily

### **Current Implementation Status**:
- **Architecture In Place**: Core structure implemented but functionality missing
- **LLM Integration Missing**: Phase 1 uses rule-based instead of actual LLM
- **Tool Execution Missing**: Phase 2 returns mock results instead of executing tools
- **Testing Incomplete**: Database issues prevent comprehensive validation

### **Design Principles** (Partially Implemented):
- **YML as Structured Prompt**: ✅ Parseable, auditable structures implemented
- **Two-Phase LLM Approach**: ⚠️ Structure in place but not using LLMs
- **Verifiable Flows**: ⚠️ YML schema exists but can't generate real flows
- **Language Agnostic**: ❌ Can't handle varied input without LLM integration

## 🚀 **Novel Architecture: Verifiable AI-Generated DeFi Flows**

This document describes a fundamentally new approach to AI-generated DeFi flows that differs from traditional testing:

### **Not Traditional Testing**:
- NOT unit/integration testing patterns
- NOT cargo test framework approach
- NOT mock-based testing

### **Instead: Verification-First Architecture**:
- AI generates flows that can be objectively verified
- YML serves dual purposes: runtime guardrails AND evaluation criteria
- SURFPOOL enables deterministic verification of AI-generated flows
- Each flow is both executable and verifiable

## 🔄 **Two-Phase LLM Architecture**

### **Phase 1: Refine + Plan (Single LLM Call)**
```
User Prompt (any language/typos) 
   ↓
[Refine+Plan LLM Call] 
   ↓
Structured YML Flow Plan (wallet context + steps)
```

**Purpose**: 
- Handle language variations, typos, slang ("swp" → "swap")
- Extract user intent and parameters
- Create structured execution plan
- Generate YML flow with wallet context and steps

### **Phase 2: Tool Execution (Multiple LLM Calls)**
```
Structured YML Flow Plan
   ↓
[Tool Execution LLM Calls] - one per tool
   ↓
Individual Tool Executions
```

**Purpose**:
- Generate tool-specific parameters
- Handle tool-specific context
- Execute each step with proper validation

## 📋 **YML as Structured Prompt Format**

### **Key Benefits**:
1. **Parseable & Auditable**: Machine-readable, human-verifiable
2. **LLM-Generable**: Can be generated from templates by LLMs
3. **Dual Purpose**: Runtime guardrails AND evaluation criteria
4. **Verifiable**: Can be validated against expected outcomes

### **YML Flow Structure for LLM Input**:
```yaml
# Minimal structured YML for LLM input (Phase 1)
subject_wallet_info:
  - pubkey: "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh"
    lamports: 4000000000 # 4 SOL
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000 # 20 USDC

steps:
  - prompt: "swap 2 SOL to USDC"
    context: "Need 2 SOL (50% of 4 SOL) to multiply USDC position by 1.5x"
  - prompt: "lend {SWAPPED_USDC} to jupiter"
    context: "Lend swapped USDC for yield to achieve 1.5x multiplication"
```

### **YML Ground Truth for Guardrails**:
```yaml
# Ground truth for validation and guardrails (NOT sent to LLM)
ground_truth:
  final_state_assertions:
    - type: SolBalanceChange
      pubkey: "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh"
      expected_change_gte: -200500000 # Should not use more than 2 SOL + fees
      error_tolerance: 0.01 # 1% tolerance for slippage
  expected_tool_calls:
    - tool_name: "jupiter_swap"
      critical: true
    - tool_name: "jupiter_lend_earn_deposit"
      critical: true
```

## 🏗️ **Implementation Architecture**

### **Core Components**:
1. **reev-core**: Core architecture (new crate)
   - YML flow definition schemas
   - Two-phase LLM coordination
   - Flow verification and validation

2. **reev-planner**: Phase 1 implementation
   - Prompt refinement and intent analysis
   - YML flow generation from structured context
   - Language and typo handling

3. **reev-executor**: Phase 2 implementation
   - Tool-specific LLM calls
   - Parameter generation and validation
   - Step-by-step execution

4. **reev-orchestrator**: Current role simplified
   - Flow execution from YML
   - Recovery and error handling
   - OpenTelemetry integration

### **Data Flow**:
```
User Request → [Planner] → YML Flow → [Executor] → Tool Calls → [Orchestrator] → Verified Execution
```

## 🎯 **Key Implementation Requirements**

### **Phase 1: Refine+Plan Requirements**:
1. **Wallet Context**: Handle production vs benchmark wallet resolution
2. **LLM Integration**: Single call to generate structured YML
3. **Validation**: Ensure generated YML is valid and complete
4. **Context Injection**: Include wallet state and prices in YML

### **Phase 2: Tool Execution Requirements**:
1. **Tool-Specific Prompts**: Tailored prompts for each tool
2. **Parameter Validation**: Ensure generated parameters match tool schema
3. **Step-by-Step Execution**: Execute one step at a time with validation
4. **Error Recovery**: Handle failures with proper recovery strategies

### **YML Flow Requirements**:
1. **Schema Definition**: Clear schemas for all YML structures
2. **Validation Rules**: Comprehensive validation for generated YML
3. **Dual Purpose**: Support both runtime guardrails and evaluation criteria
4. **Error Tolerance**: Include tolerance ranges for slippage and rate issues

## 🔄 **Wallet Context Handling**

### **Production Mode**:
- User must login with Solana first
- `subject_wallet_info` is provided directly from authenticated session
- No automatic wallet detection or generation

### **Benchmark Mode** (Feature Flag: `benchmark`):
- `USER_WALLET_PUBKEY` placeholder triggers `surfnet_setAccount` via SURFPOOL
- Generated wallet is funded with specified tokens
- Enables deterministic testing with controlled initial state

## 🔄 **Context Building and Error Recovery**

### **Context Building**:
- Re-fetch wallet info before each step
- Build contextual prompt for each step
- Include error tolerance ranges (1% for slippage/rate issues)

### **Error Recovery**:
- Atomic flow with `flow[step]` like `tx[ix]` structure
- Network errors: Retry once
- Slippage errors (within 1%): Retry with refined parameters
- Critical step failures: Stop flow with proper error reporting

## 🎯 **Success Criteria**

### **Functional Requirements**:
- ✅ Handle any language or typos in user prompts
- ✅ Generate valid, structured YML flows
- ✅ Execute flows with proper verification
- ✅ Apply ground truth guardrails during execution

### **Performance Requirements**:
- ✅ Phase 1 planning < 2 seconds
- ✅ Phase 2 tool calls < 1 second each
- ✅ Complete flow execution < 10 seconds
- ✅ 90%+ success rate on common flows

### **Quality Requirements**:
- ✅ Comprehensive audit trails
- ✅ Verifiable outcomes with guardrails
- ✅ Clear error handling and recovery
- ✅ Deterministic behavior in benchmark mode

## 🔄 **Migration Strategy**

### **From Current Implementation**:
1. Keep reev-orchestrator for execution
2. Add reev-core and reev-planner for planning
3. Migrate flow generation to YML-based approach
4. Maintain backward compatibility where possible

### **From Original PLAN_CORE.md**:
1. Keep the verification concept from 18-step process
2. Simplify to two-phase LLM approach
3. Replace fixed steps with structured YML flows
4. Maintain audit trail requirements with structured YML
5. Add explicit language support and typo handling

## 📝 **Next Steps**

1. Implement reev-core crate with YML schemas
2. Implement Phase 1 LLM integration for structured YML generation
3. Implement Phase 2 tool execution with parameter validation
4. Add ground truth guardrails for flow verification
5. Implement error recovery with atomic flow logic
6. Create benchmark mode with SURFPOOL integration

## 📊 **Related Documents**

- PLAN_CORE_BENCHMARK.md: Detailed planning for benchmark evaluation criteria and testing strategies