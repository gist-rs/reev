# Reev Core Architecture Plan V3

## 🎯 **Why: Realigning Implementation with Vision**

### **What We've Learned**:
- The YML schema and flow structures are well-designed and working
- The two-phase architecture concept is solid but implementation needs fixes
- Direct vs. indirect execution adds complexity without clear benefits
- Rule-based approach works for simple cases but fails at language flexibility
- Current test structure provides a good foundation for end-to-end validation

### **Current Implementation Status**:
- ✅ YML schema and flow structures implemented
- ✅ Basic two-phase structure in place (Planner + Executor)
- ✅ Tool execution framework working
- ✅ End-to-end tests validating real blockchain operations
- ⚠️ LLM integration incomplete (rule-based fallback primary)
- ⚠️ Phase 2 parameter generation bypassed
- ❌ Ground truth validation missing
- ❌ Proper error recovery not implemented

### **Design Principles for V3**:
- **Simplify over complicate**: Remove unnecessary complexity like direct/indirect split
- **LLM-first with intelligent fallbacks**: LLMs should be default, not optional
- **Validation-driven execution**: Ground truth should drive execution decisions
- **Incremental implementation**: Each phase should deliver working functionality

## 🚀 **Revised Architecture: Validation-Driven Execution**

### **Core Concept**:
Instead of "direct vs. indirect" execution, we use a **validation-driven approach**:

```yaml
# YML Flow Example with Validation Parameters
steps:
  - prompt: "swap 1 SOL to USDC"
    validation:
      extract_from: "prompt"  # Extract from prompt using LLM
      required_params: ["amount", "from_token", "to_token"]
      constraints:
        max_amount: "{{wallet.sol_balance * 0.95}}"  # Leave 5% for fees
        from_token: "SOL"
        to_token: "USDC"
```

### **Simplified Two-Phase Architecture**:

**Phase 1: Flow Generation (LLM-first)**
```
User Prompt (any language/typos) 
   ↓
[LLM Flow Generation] - with structured template
   ↓
Structured YML Flow with Validation Rules
```

**Phase 2: Step Execution (Validation-driven)**
```
YML Step with Validation
   ↓
[Parameter Extraction] - LLM or rule-based based on complexity
   ↓
[Tool Execution] - with pre-execution validation
   ↓
[Result Validation] - against expected outcomes
```

## 📋 **Revised YML Structure**

### **YML Flow with Validation Parameters**:
```yaml
# Enhanced YML Flow Structure
flow_id: "uuid-v7"
user_prompt: "swap 1 sol to usdc"
created_at: "timestamp"

# Wallet context (resolved at runtime)
subject_wallet_info:
  - pubkey: "{{WALLET_PUBKEY}}"  # Resolved at runtime
    tokens:
      - mint: "So11111111111111111111111111111111111111112"
        amount: "{{SOL_BALANCE}}"
        value_usd: "{{SOL_BALANCE * SOL_PRICE}}"

# Steps with validation rules
steps:
  - step_id: "swap_1"
    prompt: "swap 1 SOL to USDC"
    context: "User wants to exchange SOL for USDC"
    
    # Validation and parameter extraction rules
    validation:
      # How to extract parameters
      extraction_method: "llm"  # Options: "llm", "rule", "prompt_parse"
      
      # What parameters are needed
      required_params:
        - name: "amount"
          source: "prompt"  # Extract from prompt
          type: "number"
          default: 1.0
          constraints:
            min: 0.001
            max: "{{wallet.sol_balance * 0.95}}"
        
        - name: "from_token"
          source: "wallet"  # Get from wallet context
          type: "token"
          default: "SOL"
        
        - name: "to_token"
          source: "prompt"  # Extract from prompt
          type: "token"
          options: ["USDC", "USDT", "SOL"]  # Allowed values
      
      # Pre-execution validation
      pre_execution:
        - check: "sufficient_balance"
          params: ["amount", "from_token"]
        
        - check: "valid_token_pair"
          params: ["from_token", "to_token"]
      
      # Post-execution validation
      post_execution:
        - check: "min_output_received"
          params: ["{{amount * expected_rate * 0.98}}"]  # 2% slippage
          tolerance: 0.01
      
      # Error handling
      error_handling:
        insufficient_balance:
          action: "retry_with_max"
          params: ["{{wallet.sol_balance * 0.95}}"]
        
        slippage_exceeded:
          action: "retry_with_adjusted_params"
          params: ["amount: {{amount * 0.98}}"]

# Ground truth for overall validation
ground_truth:
  final_state:
    - type: "token_balance_change"
      token: "SOL"
      expected_change: "-{{swap_1.amount + fees}}"
      tolerance: 0.01
    
    - type: "token_balance_change"
      token: "USDC"
      expected_change: "{{swap_1.amount * expected_rate}}"
      tolerance: 0.02
```

## 🏗️ **Implementation Architecture V3**

### **Core Components**:
1. **reev-core**: Core schemas and interfaces
   - Enhanced YML schema with validation rules
   - Common types and utilities
   - Validation framework interfaces

2. **reev-planner**: Phase 1 implementation
   - LLM flow generation with templates
   - Rule-based fallback for simple cases
   - Flow validation before execution

3. **reev-executor**: Phase 2 implementation
   - Parameter extraction (LLM or rule-based)
   - Pre-execution validation
   - Tool execution with error recovery
   - Post-execution validation

4. **reev-validator**: Validation framework
   - Parameter validation
   - State change validation
   - Error handling and recovery strategies

### **Revised Data Flow**:
```
User Request → [Planner] → Validated YML Flow → [Executor] → 
[Parameter Extraction] → [Pre-execution Validation] → [Tool Execution] → 
[Post-execution Validation] → [Error Recovery if needed] → Final Result
```

## 🎯 **Key Implementation Requirements**

### **Phase 1: Enhanced Flow Generation**:
1. **LLM-first with Templates**: Use structured templates for LLM generation
2. **Intelligent Fallbacks**: Rule-based for simple prompts, LLM for complex
3. **Flow Validation**: Validate generated flows before execution
4. **Template Library**: Create templates for common flow patterns

### **Phase 2: Validation-driven Execution**:
1. **Parameter Extraction**: Choose LLM or rule-based based on complexity
2. **Pre-execution Validation**: Validate parameters before tool calls
3. **Tool Execution**: Execute with proper error handling
4. **Post-execution Validation**: Verify results against expectations
5. **Error Recovery**: Handle failures with configured strategies

### **Validation Framework**:
1. **Parameter Validation**: Ensure parameters meet constraints
2. **State Validation**: Verify state changes match expectations
3. **Business Rule Validation**: Apply domain-specific rules
4. **Error Recovery**: Intelligent recovery based on error types

## 🔄 **Error Recovery Strategy**

### **Recovery Hierarchy**:
1. **Parameter Adjustments**: Modify parameters within constraints
2. **Alternative Tools**: Try alternative approaches if available
3. **Retry with Different Parameters**: Try again with different values
4. **Partial Success**: Report partial success if applicable
5. **Graceful Failure**: Fail with clear explanation

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
  
  tool_unavailable:
    action: "try_alternative_tool"
    params: ["alternative: manual_swap"]
```

## 🔄 **Migration Strategy from Current Implementation**

### **Phase 1: Enhance YML Schema** (Week 1)
1. Add validation rules to YML structures
2. Update flow generation to include validation parameters
3. Create validation rule templates for common scenarios

### **Phase 2: Refine Planner** (Week 2-3)
1. Implement LLM-first flow generation with templates
2. Add flow validation before returning flows
3. Enhance rule-based fallback for simple cases

### **Phase 3: Redesign Executor** (Week 3-4)
1. Remove direct/indirect execution distinction
2. Implement validation-driven execution
3. Add parameter extraction with LLM/rule-based selection

### **Phase 4: Implement Validator** (Week 4-5)
1. Create validation framework components
2. Implement pre and post-execution validation
3. Add error recovery strategies

### **Phase 5: Enhance Tests** (Week 5-6)
1. Update tests to work with new validation rules
2. Add comprehensive error handling tests
3. Add performance benchmarks

## 📊 **Success Metrics**

### **Functional Requirements**:
- Handle 90%+ of common DeFi operations without manual intervention
- Successfully recover from common error scenarios
- Generate appropriate flows for 95%+ of user prompts

### **Performance Requirements**:
- Flow generation < 2 seconds for 90% of prompts
- Parameter extraction < 500ms for 90% of steps
- End-to-end execution < 10 seconds for simple flows

### **Quality Requirements**:
- Zero successful executions that violate constraints
- Clear error messages for all failure modes
- Comprehensive audit trails for all operations

## 📝 **Next Immediate Steps**

1. **Create Enhanced YML Schema** (Week 1)
   - Add validation rules to current YmlStep structure
   - Define validation rule templates for common scenarios
   - Update test YML files to use new structure

2. **Implement Validation Framework** (Week 2)
   - Create parameter validation components
   - Implement pre and post-execution validation
   - Add error recovery strategies

3. **Refine Planner** (Week 2-3)
   - Make LLM-first generation the default
   - Create flow generation templates
   - Add flow validation before execution

4. **Update Executor** (Week 3-4)
   - Replace direct/indirect with validation-driven approach
   - Implement parameter extraction with LLM/rule selection
   - Add comprehensive error handling