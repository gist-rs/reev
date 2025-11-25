# Reev Core Architecture Plan V3

## 🎯 **Why: Correcting Misunderstandings and Refining Implementation**

### **What We've Learned**:
- The YML schema and flow structures are well-designed and working
- The two-phase architecture concept is solid but implementation needs refinement
- Direct execution functions (execute_direct_sol_transfer, etc.) are correct and aligned with V2
- Rule-based YML generation is appropriate for technical accuracy
- LLM's role is specifically for language refinement, not structure generation
- Rig framework should be used for LLM-driven tool selection in Phase 2
- Current test structure provides a good foundation for end-to-end validation

### **Current Implementation Status**:
- ✅ YML schema and flow structures implemented
- ✅ Two-phase structure in place (Planner + Executor)
- ✅ Tool execution framework working with direct execution functions
- ✅ End-to-end tests validating real blockchain operations
- ⚠️ Phase 1 LLM integration incomplete (rule-based fallback primary)
- ⚠️ LLM role for prompt refinement not fully implemented
- ❌ Ground truth validation not used during execution
- ❌ Proper error recovery not implemented

### **Design Principles for V3**:
- **Clarify responsibilities**: LLM for language refinement, rules for YML structure, rig for tool selection
- **Strengthen Phase 1**: Implement proper LLM-based prompt refinement only
- **Enhance Phase 2**: Replace direct tool calls with rig framework for tool selection and calling
- **Add validation during execution**: Use ground truth for runtime validation
- **Improve error handling**: Add comprehensive error recovery strategies

## 🚀 **Refined Two-Phase Architecture**

### **Phase 1: Prompt Refinement (LLM-focused)**
```
User Prompt (any language/typos) 
   ↓
[LLM Prompt Refinement] - Refine language, extract intent
   ↓
[Rule-based YML Generation] - Generate structured YML with refined prompts
   ↓
Structured YML Flow with Refined Prompts
```

### **Phase 2: Rig-Driven Tool Execution with Validation**
```
YML Step with Refined Prompts
   ↓
[Rig Agent] - Uses refined prompt to select and call tools
   ↓
[Tool Selection] - LLM determines appropriate tools from refined prompt
   ↓
[Parameter Extraction] - LLM extracts parameters from refined prompt
   ↓
[Tool Execution] - Rig handles tool calling with extracted parameters
   ↓
[Result Validation] - Against ground truth with error recovery
```

## 📋 **YML Structure (Simplified and Focused)**

### **YML Flow with Refined Prompts**:
```yaml
# Simplified YML Flow Structure (maintaining V2 design)
flow_id: "uuid-v7"
user_prompt: "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
refined_prompt: "transfer 1 SOL to address gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
created_at: "timestamp"

# Wallet context (resolved at runtime)
subject_wallet_info:
  - pubkey: "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh"
    lamports: 4000000000 # 4 SOL
    tokens:
      - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        amount: 20000000 # 20 USDC

# Steps with refined prompts from LLM
steps:
  - step_id: "transfer_1"
    refined_prompt: "transfer 1 SOL to address gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    context: "User wants to transfer 1 SOL to the specified recipient"
    critical: true
    expected_tools: ["SolTransfer"]  # Hint for rig agent

# Ground truth for validation and guardrails
ground_truth:
  final_state_assertions:
    - type: SolBalanceChange
      pubkey: "5HNT58ajgxLSU3UxcpJBLrEEcpK19CrZx3d5C3yrkPHh"
      expected_change_lte: -1005000000 # 1 SOL + fees
      error_tolerance: 0.01
  expected_tool_calls:
    - tool_name: "SolTransfer"
      critical: true
```

## 🏗️ **Refined Implementation Architecture**

### **Core Components**:
1. **reev-core**: Core schemas and interfaces
   - Current YML schema (maintained)
   - Common types and utilities
   - Validation framework interfaces

2. **reev-planner**: Phase 1 implementation
   - LLM prompt refinement (enhanced)
   - Rule-based YML generation from refined prompts
   - Template-based flow generation for common patterns

3. **reev-executor**: Phase 2 implementation
   - Rig agent integration for tool selection and calling
   - Parameter extraction from refined prompts via LLM
   - Result validation against ground truth
   - Error recovery strategies
   - Transition from direct execution functions to rig-driven execution

4. **reev-validator**: Validation framework (new)
   - Runtime validation against ground truth
   - Parameter validation before execution
   - Error handling and recovery strategies
   - Integration with rig agent for validation feedback

### **Clarified Data Flow**:
```
User Request → [LLM Prompt Refinement] → [Rule-based YML Generation] → 
[Executor] → [Rig Agent] → [Tool Selection] → [Parameter Extraction] → 
[Tool Execution] → [Result Validation] → [Error Recovery if needed] → Final Result
```

## 🎯 **Key Implementation Requirements**

### **Phase 1: Enhanced Prompt Refinement**:
1. **LLM Integration for Refinement**:
   - Refine user prompts to clear, unambiguous language ONLY
   - Fix typos and normalize language variations
   - Do NOT extract intent or determine tools (leave for Phase 2)
   - Add minimal context for execution

2. **Template-based YML Generation**:
   - Use refined prompts with rule-based templates
   - Ensure technical accuracy in YML structure
   - Include expected_tools hints for rig agent
   - Generate appropriate ground truth for validation

### **Phase 2: Rig-Driven Execution with Validation**:
1. **Replace Direct Execution with Rig Agent**:
   - Create rig agent with available tools (SolTransfer, JupiterSwap, etc.)
   - Use refined prompts for tool selection and parameter extraction
   - Maintain existing execute_direct_* functions as fallbacks
   - Gradually migrate to rig-based execution

2. **Add Runtime Validation**:
   - Validate extracted parameters against ground truth
   - Check constraints before tool execution
   - Validate results after execution
   - Provide validation feedback to rig agent

3. **Implement Error Recovery**:
   - Handle parameter validation failures
   - Retry with adjusted parameters when appropriate
   - Provide clear error messages for debugging
   - Allow rig agent to select alternative tools when needed

### **Validation Framework**:
1. **Parameter Validation**:
   - Ensure extracted parameters meet constraints
   - Validate against wallet context
   - Apply business rules for specific operations

2. **Result Validation**:
   - Compare execution results against expected outcomes
   - Handle slippage and rate variations
   - Verify final state changes

### **Error Recovery**:
   - Intelligent recovery based on error types
   - Parameter adjustments within constraints
   - Alternative execution strategies
   - Integration with rig agent for tool selection changes

## 🔄 **Error Recovery Strategy**

### **Recovery During Execution**:
1. **Parameter Validation Failures**:
   - Adjust parameters within constraints
   - Retry with modified values
   - Report specific validation errors
   - Allow rig agent to select alternative parameters

2. **Tool Execution Failures**:
   - Network errors: Retry with backoff
   - Slippage errors: Adjust parameters and retry
   - Insufficient balance: Use maximum available
   - Tool-specific errors: Allow rig agent to select alternative tools

3. **Result Validation Failures**:
   - Report specific validation failures
   - Suggest parameter adjustments
   - Provide clear next steps
   - Allow rig agent to attempt alternative approaches

### **Error Types and Responses**:
```yaml
error_responses:
  insufficient_balance:
    action: "retry_with_max"
    params: ["{{wallet.sol_balance * 0.95}}"]
  
  slippage_exceeded:
    action: "retry_with_adjusted_params"
    params: ["amount: {{amount * 0.98}}"]
  
  network_error:
    action: "retry_with_backoff"
    params: ["initial_delay: 1s", "max_retries: 3"]
  
  validation_error:
      action: "report_and_suggest"
      params: ["error_type", "suggested_fix"]
  
    tool_specific_error:
      action: "retry_with_alternative_tool"
      params: ["original_tool", "suggested_alternatives"]
  ```

## 🔄 **Migration Strategy from Current Implementation**

### **Phase 1: Enhance LLM Integration** (Week 1-2)
1. Implement proper LLM-based prompt refinement ONLY
2. Create templates for common operation types
3. Add prompt refinement tests
4. Create rig agent with available tools for Phase 2

### **Phase 2: Strengthen Rule-based YML Generation** (Week 2-3)
1. Enhance rule-based templates for YML generation
2. Add expected_tools hints for rig agent
3. Add more comprehensive ground truth generation
4. Improve wallet context resolution

### **Phase 3: Add Validation Framework** (Week 3-4)
1. Create validation components
2. Implement parameter validation
3. Add result validation against ground truth
4. Integrate validation with rig agent feedback

### **Phase 4: Implement Error Recovery** (Week 4-5)
1. Add error recovery strategies
2. Implement retry logic with backoff
3. Add comprehensive error reporting

### **Phase 5: Enhance Tests** (Week 5-6)
1. Add tests for prompt refinement
2. Add validation tests
3. Add error recovery tests

## 📊 **Success Metrics**

### **Functional Requirements**:
- Handle 90%+ of common DeFi operations with refined prompts
- Successfully recover from common error scenarios
- Generate appropriate flows for 95%+ of user prompts

### **Performance Requirements**:
- Prompt refinement < 1 second for 90% of prompts
- YML generation < 500ms for 90% of cases
- End-to-end execution < 10 seconds for simple flows

### **Quality Requirements**:
- Zero successful executions that violate constraints
- Clear error messages for all failure modes
- Comprehensive audit trails for all operations

## 📝 **Next Immediate Steps**

1. **Enhance LLM Prompt Refinement** (Week 1)
   - Implement proper LLM integration for prompt refinement ONLY
   - Create templates for refined prompts
   - Add tests for refinement quality
   - Create rig agent with available tools

2. **Strengthen Rule-based YML Generation** (Week 1-2)
   - Improve templates for YML generation
   - Add expected_tools hints for rig agent
   - Enhance ground truth generation
   - Add more comprehensive validation rules

3. **Add Validation Framework** (Week 2-3)
   - Create parameter validation components
   - Implement result validation
   - Add validation to the execution flow
   - Integrate validation with rig agent

4. **Implement Error Recovery** (Week 3-4)
   - Add error recovery strategies
   - Implement retry logic
   - Add comprehensive error reporting
   - Add alternative tool selection via rig agent

5. **Migrate to Rig-Driven Execution** (Week 4-6)
   - Implement rig agent for tool selection and calling
   - Replace direct tool calls with rig-based execution
   - Add comprehensive tests for rig-driven execution
   - Maintain direct execution functions as fallbacks

This revised plan corrects the misunderstandings about LLM vs rule-based responsibilities, maintains the strengths of the current implementation, and focuses on enhancing the existing architecture rather than replacing it.