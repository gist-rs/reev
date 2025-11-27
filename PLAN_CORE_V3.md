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
- ✅ Tool execution framework working with direct execution functions and RigAgent
- ✅ End-to-end tests validating real blockchain operations
- ✅ LanguageRefiner implemented for prompt refinement
- ✅ YmlGenerator implemented for rule-based YML generation
- ✅ FlowValidator implemented for validation checks
- ⚠️ Ground truth validation not fully integrated in execution flow
- ⚠️ Error recovery not implemented in execution flow
- ❌ Duplicated flow creation functions in planner.rs (create_swap_then_lend_flow, etc.)
- ❌ Scalability issues with fixed operation patterns

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

### **Current YML Flow Implementation Details**:

1. **LanguageRefiner**: Handles LLM-based prompt refinement
   - Located in `src/refiner/mod.rs`
   - Uses GLM-4.6-coding model via ZAI_API_KEY
   - Refines language without extracting intent

2. **YmlGenerator**: Handles rule-based YML generation
   - Located in `src/yml_generator/mod.rs`
   - Uses pattern matching to detect operation types
   - Generates structured YML with expected_tools hints

3. **FlowValidator**: Handles validation of flows
   - Located in `src/validation.rs`
   - Validates YML structure and assertions
   - Custom assertion validators for different validation types

4. **RigAgent**: Handles tool selection and parameter extraction
   - Located in `src/execution/rig_agent/mod.rs`
   - Integrates with rig framework for LLM-driven tool selection
   - Extracts parameters from refined prompts

5. **Executor**: Handles flow execution with validation
   - Located in `src/executor.rs`
   - Converts YML flows to DynamicFlowPlan for execution
   - Supports both direct execution and RigAgent-based execution

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
User Request → [Planner::refine_and_plan()] → [LanguageRefiner::refine_prompt()] → 
[YmlGenerator::generate_flow()] → [FlowValidator::validate_flow()] → 
[Executor::execute_flow()] → [RigAgent::execute_step()] → [ToolExecutor] → 
[Result Validation] → [Error Recovery if needed] → Final Result
```

### **Current Implementation Classes**:

1. **Planner**: Entry point for Phase 1
   - Uses LanguageRefiner for prompt refinement
   - Uses YmlGenerator for YML generation
   - Provides `refine_and_plan()` method for flow generation

2. **LanguageRefiner**: LLM-based prompt refinement
   - Uses GLM-4.6-coding model via ZAI_API_KEY
   - Refines language without extracting intent
   - Provides `refine_prompt()` method

3. **YmlGenerator**: Rule-based YML generation
   - Uses pattern matching for operation detection
   - Generates structured YML with expected_tools hints
   - Provides `generate_flow()` method

4. **FlowValidator**: Validation framework
   - Validates YML structure and assertions
   - Custom assertion validators for different validation types
   - Provides `validate_flow()` method

5. **Executor**: Flow execution engine
   - Converts YML flows to DynamicFlowPlan for execution
   - Supports both direct execution and RigAgent-based execution
   - Provides `execute_flow()` method

6. **RigAgent**: LLM-driven tool selection
   - Integrates with rig framework for tool selection
   - Extracts parameters from refined prompts
   - Provides `execute_step()` method

7. **ToolExecutor**: Tool execution engine
   - Executes selected tools with extracted parameters
   - Handles blockchain interactions
   - Provides `execute_tool()` method

## 🎯 **Key Implementation Requirements**

### **Phase 1: Enhanced Prompt Refinement** ✅:
1. **LLM Integration for Refinement**:
   - Refine user prompts to clear, unambiguous language ONLY ✅
   - Fix typos and normalize language variations ✅
   - Do NOT extract intent or determine tools (leave for Phase 2) ✅
   - Add minimal context for execution ✅

2. **Template-based YML Generation**:
   - Use refined prompts with rule-based templates ✅
   - Ensure technical accuracy in YML structure ✅
   - Include expected_tools hints for rig agent ✅
   - Generate appropriate ground truth for validation ✅

### **Phase 2: Rig-Driven Execution with Validation** ✅:
1. **Replace Direct Execution with Rig Agent** ✅:
   - Create rig agent with available tools (SolTransfer, JupiterSwap, etc.) ✅
   - Use refined prompts for tool selection and parameter extraction ✅
   - Maintain existing execute_direct_* functions as fallbacks ✅
   - Gradually migrate to rig-based execution ✅

2. **Add Runtime Validation** ✅:
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

### **Phase 1: Remove Duplication and Clean Up** (Week 1)
1. Remove unused functions in planner.rs:
   - generate_flow_rule_based()
   - create_swap_flow()
   - create_transfer_flow()
   - create_lend_flow()
   - create_swap_then_lend_flow()
   - Related helper functions like parse_intent(), extract_swap_params(), etc.
2. Keep builder functions in yml_schema.rs for testing
3. Ensure existing tests still pass after removal
4. Update documentation to reflect current implementation

### **Phase 2: Improve Scalability** (Week 1-2) ✅
1. Refactor YmlGenerator for dynamic operation sequences:
   - Create OperationParser for flexible operation detection ✅
   - Implement composable step builders for individual operations ✅
   - Add support for arbitrary operation sequences ✅
    
2. Create a Composable Step-Based System:
   - Define individual step creators:
     ```rust
     async fn create_swap_step(&self, from: &str, to: &str, amount: f64) -> Result<YmlStep>
     async fn create_lend_step(&self, mint: &str, amount: f64) -> Result<YmlStep>
     async fn create_transfer_step(&self, mint: &str, to: &str, amount: f64) -> Result<YmlStep>
     ```
   - Parse the sequence of operations from the prompt:
     ```rust
     async fn parse_operation_sequence(&self, prompt: &str) -> Result<Vec<Operation>>
     ```

3. Implement a Unified Flow Builder:
   - Create a single flow builder that can handle any sequence of operations:
     ```rust
     async fn build_flow_from_operations(
         &self,
         prompt: &str,
         wallet_context: &WalletContext,
         operations: Vec<Operation>
     ) -> Result<YmlFlow>
     ```
   - **Why chunk approach works better than templates**: Instead of predefining all possible combinations,
     we create individual operation chunks that can be dynamically combined:
     ```yaml
     # chunk-based approach (current implementation):
     # Each operation is a self-contained chunk with its own logic
     
     # swap chunk handles any SOL→USDC swap
     # lend chunk handles any USDC→Jupiter lending
     # transfer chunk handles any SOL→recipient transfer
     
     # These chunks are combined dynamically based on user prompt:
     # "swap SOL→USDC then lend" → [swap_chunk, lend_chunk]
     # "swap SOL→USDC, transfer 1 SOL, lend remainder" → [swap_chunk, transfer_chunk, lend_chunk]
     ```

4. Refine the Operation Parsing:
   - Define Operation enum for sequence parsing:
     ```rust
     enum Operation {
         Swap { from: String, to: String, amount: f64 },
         Lend { mint: String, amount: f64 },
         Transfer { mint: String, to: String, amount: f64 },
     }
     ```
   - Parse "swap 1 SOL to USDC then lend" -> vec![
       Swap { from: "SOL", to: "USDC", amount: 1.0 },
       Lend { mint: "USDC", amount: 1.0 }  // Amount from previous step
     ]

5. Enhance LanguageRefiner for better prompt refinement ✅
6. Improve template system for YML generation ✅
7. Add tests for complex operation sequences ✅

### **Current Implementation Status** ✅

The chunk-based approach for multi-step flows is now implemented and validated:

1. **Individual Operation Chunks**: Self-contained builders for swap, lend, and transfer operations
   - Each chunk handles its own logic, parameters, and expected tools
   - Can be combined dynamically without predefining all sequences

2. **Dynamic Flow Composition**: Orchestrator combines chunks based on user prompt
   - Context is passed between steps (e.g., "swapped 4.99 SOL to 708.58 USDC")
   - No rule-based parsing of specific sequences required
   - Arbitrary operation sequences supported

3. **Validation**: Tests confirm multi-step flows work correctly
   - `single_step_chunks_test.rs` validates chunk creation and combination
   - `multi_step_flow_test.rs` validates orchestrator handling of multi-step flows
   - Generated YML matches expected structure

**Why Chunks Are More Scalable Than Templates**:
- Templates require predefining all valid operation combinations (n² complexity)
- Chunks allow any sequence by combining available building blocks (linear complexity)
- Context passing enables steps to use results of previous steps
- New operation types only require creating new chunks, not new templates

The chunk-based approach for multi-step flows is now implemented and validated:

1. **Individual Operation Chunks**: Self-contained builders for swap, lend, and transfer operations
   - Each chunk handles its own logic, parameters, and expected tools
   - Can be combined dynamically without predefining all sequences

2. **Dynamic Flow Composition**: Orchestrator combines chunks based on user prompt
   - Context is passed between steps (e.g., "swapped 4.99 SOL to 708.58 USDC")
   - No rule-based parsing of specific sequences required
   - Arbitrary operation sequences supported

3. **Validation**: Tests confirm multi-step flows work correctly
   - `single_step_chunks_test.rs` validates chunk creation and combination
   - `multi_step_flow_test.rs` validates orchestrator handling of multi-step flows
   - Generated YML matches expected structure

**Why Chunks Are More Scalable Than Templates**:
- Templates require predefining all valid operation combinations (n² complexity)
- Chunks allow any sequence by combining available building blocks (linear complexity)
- Context passing enables steps to use results of previous steps
- New operation types only require creating new chunks, not new templates

### **Phase 3: Integrate Validation** (Week 2-3)
1. Integrate FlowValidator into execution flow:
   - Add validation before execution in Executor
   - Add result validation after execution
   - Implement parameter validation against ground truth
2. Improve ground truth generation in YmlGenerator
3. Add validation error handling and reporting
4. Add comprehensive validation tests

### **Phase 4: Implement Error Recovery** (Week 3-4)
1. Implement ErrorRecoveryEngine:
   - Add parameter adjustment strategies
   - Implement retry logic with backoff
   - Add alternative tool selection via RigAgent
2. Integrate ErrorRecoveryEngine into Executor
3. Add comprehensive error reporting
4. Add error recovery tests

### **Phase 5: Enhance RigAgent Integration** (Week 4-5)
1. Improve RigAgent for better tool selection:
   - Enhance parameter extraction from refined prompts
   - Add tool selection confidence scoring
   - Implement tool selection fallbacks
2. Add RigAgent execution validation
3. Improve RigAgent error handling
4. Add comprehensive RigAgent tests

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

1. **Remove Duplication and Clean Up** (Week 1)
   - Remove unused functions in planner.rs
   - Ensure existing tests still pass after removal
   - Update documentation to reflect current implementation
   - Add tests to verify removal doesn't break functionality

2. **Improve Scalability** (Week 1-2)
   - Refactor YmlGenerator for dynamic operation sequences
   - Create OperationParser for flexible operation detection
   - Implement composable step builders for individual operations
   - Implement unified flow builder that can handle any sequence of operations
   - Add template-based flow generation for common patterns
   - Refine operation parsing to support arbitrary sequences
   - Add tests for complex operation sequences

3. **Integrate Validation** (Week 2-3)
   - Integrate FlowValidator into execution flow
   - Add validation before and after execution
   - Implement parameter validation against ground truth
   - Add comprehensive validation tests

4. **Implement Error Recovery** (Week 3-4)
   - Implement ErrorRecoveryEngine with strategy patterns
   - Integrate ErrorRecoveryEngine into Executor
   - Add comprehensive error reporting
   - Add error recovery tests

5. **Enhance RigAgent Integration** (Week 4-5)
   - Improve RigAgent for better tool selection
   - Enhance parameter extraction from refined prompts
   - Add tool selection confidence scoring
   - Add comprehensive RigAgent tests

This revised plan corrects the misunderstandings about LLM vs rule-based responsibilities, maintains the strengths of the current implementation, and focuses on enhancing the existing architecture rather than replacing it.