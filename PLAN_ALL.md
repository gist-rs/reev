# Plan to Fix YML Prompt Regression in Transfer Functionality

## Executive Summary

This document outlines a comprehensive plan to fix the regression in the SOL transfer functionality that occurred after commit ee605e666315d8248ec003aeb1ce46bfa44f298e. The regression resulted in the loss of YML structure validation capabilities while maintaining the LLM-based approach for processing transfer requests.

## Problem Analysis

### Current State
- The system processes transfer requests through `QueryHandler` → `Planner` → `PromptProcessor` → `YmlGenerator`
- The `PromptProcessor` correctly handles the "all" keyword by calculating the transferable amount using `calculate_max_transferable_amount`
- The `YmlGenerator` creates YML flows but doesn't properly include the wallet context for validation

### The Regression
- Before the regression: YML flows contained proper structure with wallet context and validation capabilities
- After the regression: YML flows lack the complete wallet context structure needed for validation
- The connection between refined prompts and generated YML flows is weak
- **Most importantly**: The YML prompt structure sent to the LLM for generating refined prompts is missing or inadequate

## Alignment with Core Architecture Plans

This plan aligns with the core architecture documents:

### PLAN_CORE_V3.md Alignment
- **Phase 1: Prompt Refinement (LLM-focused)**: Our YML prompt structure is a key part of the prompt refinement phase
- **YML Structure**: Following the simplified YML structure defined in PLAN_CORE_V3
- **Data Flow**: Implementing the clarified data flow from user prompt to refined prompt to execution
- **Implementation Requirements**: Focusing on prompt refinement and LLM-driven processing (not rule-based)

### PLAN_CORE_BENCHMARK.md Alignment
- **Deterministic Verification**: Using SURFPOOL for consistent testing environments
- **Ground Truth YML**: Creating YML that serves as both runtime guardrails and evaluation criteria
- **Benchmark Categories**: Supporting flow generation benchmarks with our refined prompt approach
- **Evaluation Framework**: Building towards the comprehensive scoring algorithm outlined

### PLAN_PROTOCOLS.md Alignment
- **Stage 1: Minimal Protocol Interface**: Our approach supports the immediate needs protocol interface
- **Operation Types**: Working with the defined operation types (Swap, Lend, Earn, Stake, Transfer)
- **Protocol Executor**: Our refined prompts feed into the protocol execution framework

## AI Implementation Guidelines

**CRITICAL: AI Implementation Requirements**

The AI implementation of this plan MUST follow these guidelines:

1. **No Rules-Based Parsing**: Do not implement rule-based parsing as a shortcut or fallback
   - Use LLM for prompt refinement and decision making
   - Follow the V3 architecture approach which emphasizes LLM-based processing
   - Do not create simple string replacement rules for "all" keyword handling

2. **No Cheating or Shortcuts**: Do not implement workarounds just to make tests pass
   - Implement the full LLM-based pipeline as designed
   - Do not bypass the YML prompt structure to simplify implementation
   - Do not hardcode responses or fallback mechanisms

3. **Scalable Implementation**: Build for extensibility, not just the current test case
   - Design the YML prompt structure to work for all operations (transfer, swap, lend, etc.)
   - Do not create transfer-specific implementations that won't generalize
   - Follow the modular architecture principles defined in the project rules

4. **LLM-First Approach**: Trust the LLM to handle the transformation
   - Use the YML prompt to guide the LLM rather than implementing rule-based transformations
   - Allow the LLM to make decisions about how to handle the "all" keyword
   - Implement proper error handling when LLM responses are unexpected, but do not fall back to rules

The implementation should be honest about limitations and work to improve the LLM prompts rather than bypassing them with rules-based approaches.

## Plan to Fix the Regression

### 1. Define YML Prompt Structure for LLM to Generate Refined Prompts

#### 1.1 Simplified YML Prompt Structure
- Create a minimal, clear YML prompt that works for all operations (transfer, swap, etc.)
- Include only the necessary information: original prompt and usable amount
- The simplified YML prompt should look like:
  ```yaml
    original_prompt: "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    usable_amount: "4.999"
    instruction: "replace_all_with_usable_amount"
  ```
  - For swap operations: original_prompt: "swap all sol to usdc", usable_amount: "4.999"

#### 1.2 Expected LLM Response Structure
- Define the response format expected from the LLM:
  ```json
  {
    "refined_prompt": "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
  }
  ```

#### 1.3 Integration with calculate_max_transferable_amount
- Simplified integration process:
  1. Call `calculate_max_transferable_amount` to get usable amount
  2. Construct minimal YML prompt with original_prompt and usable_amount
  3. Send to LLM
  4. Parse response to get refined_prompt
  5. Use refined_prompt in YmlGenerator

### 2. Enhance YmlGenerator to Include YML Prompt Structure

#### 2.1 Improve YML Flow Generation
- Modify `YmlGenerator::generate_flow` to ensure it properly includes:
  - The original prompt
  - The refined prompt with calculated amounts for "all" transfers
  - Complete wallet context information
  - Clear step descriptions with the calculated amounts

#### 2.2 Add YML Serialization for Wallet Context
- Ensure the `YmlFlow` structure properly serializes to YAML with:
  ```yaml
  flow_id: "uuid"
  user_prompt: "send all sol to address"
  refined_prompt: "send 4.999 sol to address"
  subject_wallet_info:
    pubkey: "wallet_address"
    lamports: 5000000000
    tokens: []
    total_value_usd: 500.0
  steps:
    - step_id: "step_uuid"
      prompt: "send 4.999 sol to address"
      context: "Transfer 4.999 SOL to address"
      refined_prompt: "send 4.999 sol to address"
      expected_tools: [sol_transfer]
      expected_tool_calls:
        - tool_name: sol_transfer
          critical: true
          expected_parameters:
            recipient: "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
            amount: "4.999"
            token: "SOL"
  ```

### 3. Improve the "All" Keyword Handling

#### 3.1 Better Integration with calculate_max_transferable_amount
- Ensure the calculated amount from `calculate_max_transferable_amount` is properly preserved:
  - In the refined prompt from `PromptProcessor`
  - In the YML flow generated by `YmlGenerator`
  - In the validation process
  - In the expected_tool_calls with the exact calculated amount (e.g., "4.999")

#### 3.2 YML Structure for "All" to Specific Amount Conversion
- When converting "send all sol to address" to "send 4.999 sol to address", the YML should include:
  ```yaml
  flow_id: "uuid"
  user_prompt: "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
  refined_prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
  subject_wallet_info:
    pubkey: "wallet_address"
    lamports: 5000000000
    total_value_usd: 500.0
  steps:
    - step_id: "step_uuid"
      prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
      context: "Transfer 4.999 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
      refined_prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
      expected_tools: [sol_transfer]
      expected_tool_calls:
        - tool_name: sol_transfer
          critical: true
          expected_parameters:
            recipient: "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
            amount: "4.999"
            token: "SOL"
  ```
- The `calculate_max_transferable_amount` should calculate 4.999 SOL from the wallet balance of 5.0 SOL, reserving 0.001 SOL for gas
- The calculated amount (4.999) should be consistently used in:
  - The refined_prompt
  - The step prompt
  - The step context
  - The expected_tool_calls parameters

#### 3.3 Enhanced Refinement Process
- Improve the refinement process to ensure:
  - The "all" keyword is consistently replaced with the calculated amount
  - The refined prompt is used consistently throughout the pipeline
  - The calculated amount is included in all relevant parts of the YML flow
  - The expected_tool_calls explicitly include the calculated amount for validation
#### 2.4 Full Example YML for "All" to Specific Amount Conversion

**Input:** "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
**Process:** 
1. `calculate_max_transferable_amount` calculates 4.999 SOL from wallet balance of 5.0 SOL (reserving 0.001 SOL for gas)
2. `PromptProcessor` creates refined prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
3. `YmlGenerator` creates YML flow with this structure:

```yaml
flow_id: "123e4567-e89b-12d3-a456-426614174000"
user_prompt: "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
refined_prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
subject_wallet_info:
  pubkey: "wallet_address"
  lamports: 5000000000
  tokens: []
  total_value_usd: 500.0
steps:
  - step_id: "step_001"
    prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    context: "Transfer 4.999 SOL to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    refined_prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
    expected_tools: [sol_transfer]
    expected_tool_calls:
      - tool_name: sol_transfer
        critical: true
        expected_parameters:
          recipient: "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
          amount: "4.999"
          token: "SOL"
    critical: true
    estimated_time_seconds: 30
ground_truth:
  expected_outcomes:
    - type: "transfer"
      success: true
      details:
        recipient: "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
        amount: "4.999"
        token: "SOL"
metadata:
  operation_type: "transfer"
  complexity: "simple"
  estimated_gas_fee: "0.001 SOL"
created_at: "2023-12-07T12:00:00Z"
```

**Key points:**
1. The original prompt with "all" is preserved in `user_prompt`
2. The refined prompt with the calculated amount (4.999) is in `refined_prompt`
3. The calculated amount is consistently used in all transfer-related fields
4. The expected_tool_calls explicitly include the calculated amount for validation
5. The ground_truth includes the expected outcome with the calculated amount

#### 2.3 Enhanced Refinement Process
- Improve the refinement process to ensure:
  - The "all" keyword is consistently replaced with the calculated amount
  - The refined prompt is used consistently throughout the pipeline
  - The calculated amount is included in all relevant parts of the YML flow
  - The expected_tool_calls explicitly include the calculated amount for validation

### 4. Add YML Validation Capabilities

#### 4.1 Structure Validation
- Add functionality to validate the generated YML flows:
  - Ensure the YML structure is valid
  - Verify that the refined prompt matches the expected operation
  - Check that the wallet context is properly included

#### 4.2 Content Validation
- Validate the content of the YML flows:
  - Ensure transfer amounts match between refined prompt and YML steps
  - Verify that calculated amounts for "all" transfers are consistent
  - Check that wallet context information is accurate

### 5. Update Test Coverage

#### 5.1 Update e2e_transfer.rs Test
- Enhance the existing test to verify:
  - The generated YML flow has the correct structure
  - The wallet context is properly included
  - The refined prompt contains the calculated amount for "all" transfers

#### 5.2 Add New Tests
- Create additional tests to verify:
  - YML structure validation
  - "All" keyword handling across the entire pipeline
  - Integration between `PromptProcessor` and `YmlGenerator`
  - YML prompt structure sent to LLM

## Implementation Steps

### Phase 1: YML Prompt Structure Definition
1. Define the YML prompt structure for the LLM to generate refined prompts
2. Implement the construction of YML prompts using `calculate_max_transferable_amount`
3. Define the expected LLM response structure and parsing logic
4. Test the YML prompt generation with sample "all" transfer requests

### Phase 2: YML Structure Enhancement
1. Modify `YmlGenerator::generate_flow` to properly include the refined prompt and wallet context in the YML structure
2. Add YML serialization support for the `YmlFlow` structure
3. Ensure the `calculate_max_transferable_amount` function is properly used throughout the pipeline

### Phase 3: Validation Implementation
1. Implement YML structure validation
2. Add content validation for transfer amounts and wallet context
3. Ensure validation works for both specific amounts and "all" transfers

### Phase 4: Test Enhancement
1. Update the `e2e_transfer.rs` test to verify the YML structure and prompt handling
2. Add additional tests for YML validation and "all" keyword handling
3. Ensure all tests pass with the enhanced implementation

### Phase 5: Integration Testing
1. Run full end-to-end tests with the enhanced implementation
2. Verify that the YML structure is correctly generated and validated
3. Ensure the transfer functionality works correctly for both specific amounts and "all" transfers

## Expected Outcomes

After implementing this plan:

1. The YML flows will contain the complete wallet context structure needed for validation
2. The "all" keyword will be properly handled with the calculated amounts preserved throughout the pipeline
3. The generated YML flows will be properly structured and validated
4. The transfer functionality will work correctly for both specific amounts and "all" transfers
5. The test suite will validate the YML structure and content

## Success Criteria

1. The `e2e_transfer.rs` test passes for both specific amounts and "all" transfers
2. The generated YML flows contain the proper structure with wallet context
3. The refined prompts contain the calculated amounts for "all" transfers
4. The YML validation works correctly for both structure and content
5. No regression in the transfer functionality

## Future Considerations

1. This plan focuses on fixing the regression for transfer functionality
2. Similar enhancements may be needed for other operations (swap, lend, etc.)
3. The YML validation capabilities could be extended to support more complex validation rules
- The approach could be generalized to support more complex multi-step operations



## Conclusion

This plan addresses the regression in the transfer functionality by enhancing the YML structure generation, improving the "all" keyword handling, adding validation capabilities, and updating the test suite. The key addition is the YML prompt structure for the LLM, which provides clear guidance on how to transform prompts with "all" keywords into prompts with specific amounts calculated using `calculate_max_transferable_amount`.

The implementation will restore the functionality while maintaining the current architecture and adding the YML validation capabilities that were lost in the regression. Most importantly, it will do so using a true LLM-first approach that aligns with the core architecture plans, avoiding the pitfalls of rules-based implementations that are not scalable.

By following the AI implementation guidelines outlined above, the solution will be robust, extensible, and true to the V3 architecture principles of LLM-driven processing.
