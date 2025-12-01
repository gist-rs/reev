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
- **Critical Issue**: Current implementation in `PromptProcessor` contains fallback logic (lines 108-140) that bypasses the LLM with rules-based replacements

### Current Unstructured Prompt Issue
The current implementation uses an unstructured text format:
```
"Transferable amount: 4.999 SOL. send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
```

This format is ambiguous and doesn't provide clear guidance to the LLM about its task.

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
   - **CRITICAL**: Remove all fallback logic from `PromptProcessor` (lines 108-140)

3. **Scalable Implementation**: Build for extensibility, not just the current test case
   - Design the YML prompt structure to work for all operations (transfer, swap, lend, etc.)
   - Do not create transfer-specific implementations that won't generalize
   - Follow the modular architecture principles defined in the project rules

4. **LLM-First Approach**: Trust the LLM to handle the transformation
   - Use the YML prompt to guide the LLM rather than implementing rule-based transformations
   - Allow the LLM to make decisions about how to handle the "all" keyword
   - **CRITICAL**: Implement proper error handling when LLM responses are unexpected, but do not fall back to rules
   - Return errors directly instead of falling back to rules-based replacements

5. **Evaluation Platform Integrity**: This is an evaluation platform
   - The system should grade LLM responses directly without workarounds
   - No fallback mechanisms that would mask LLM performance issues
   - All LLM responses should be evaluated as-is

The implementation should be honest about limitations and work to improve the LLM prompts rather than bypassing them with rules-based approaches.

## Plan to Fix the Regression

### 1. Replace Unstructured Prompt with Structured YML Prompt

#### 1.1 Define TransferAmountRefinementRequest Structure
Create a structured YML prompt for the LLM to refine transfer amounts:

```rust
/// Structured YML prompt for LLM to refine transfer amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAmountRefinementRequest {
    /// The original user prompt
    pub original_prompt: String,
    /// Maximum transferable amount in SOL (after gas reserve)
    pub usable_amount: f64,
    /// Instruction for the LLM
    pub instruction: String,
}

impl TransferAmountRefinementRequest {
    /// Create a new refinement request
    pub fn new(original_prompt: String, usable_amount: f64) -> Self {
        Self {
            original_prompt,
            usable_amount,
            instruction: "Replace 'all' with the usable_amount in the prompt".to_string(),
        }
    }

    /// Convert to YAML string for LLM
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| anyhow!("Failed to serialize refinement request: {e}"))
    }
}
```

#### 1.2 Expected LLM Response Structure
Define the response format expected from the LLM:
```json
{
  "refined_prompt": "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
}
```

#### 1.3 Integration with calculate_max_transferable_amount
Simplified integration process:
1. Call `calculate_max_transferable_amount` to get usable amount
2. Create `TransferAmountRefinementRequest` with original_prompt and usable_amount
3. Convert to YAML using `to_yaml()`
4. Send structured YAML to LLM
5. Parse response to get refined_prompt
6. Use refined_prompt in YmlGenerator

### 2. Update PromptProcessor Implementation

#### 2.1 Remove Fallback Logic
Completely remove the fallback logic from `PromptProcessor::process_prompt` (lines 108-140). Instead of falling back to rules-based replacement, simply return the error from the LLM call.

#### 2.2 Replace Unstructured Prompt with Structured YML
Replace the current unstructured approach (line 100-102) with structured YML:

```rust
// Create structured YML prompt for LLM
let refinement_request = TransferAmountRefinementRequest::new(original_prompt.clone(), max_amount_sol);
let yml_prompt = refinement_request.to_yaml()?;

// Build LLM request for language refinement
let request = LanguageRefineRequest {
    prompt: yml_prompt, // Now contains structured YAML instead of unstructured text
};
```

#### 2.3 Update Error Handling
Replace the current error handling with a clean approach that returns errors directly:

```rust
// Send request to LLM
let response = self.send_refine_request(&request).await?;

// Parse response - expect structured JSON
let response_obj = serde_json::from_str::<LanguageRefineResponse>(&response)
    .map_err(|e| anyhow!("Failed to parse LLM response as JSON: {e}"))?;
```

### 3. Enhance YmlGenerator to Include YML Prompt Structure

#### 3.1 Improve YML Flow Generation
Modify `YmlGenerator::generate_flow` to ensure it properly includes:
- The original prompt
- The refined prompt with calculated amounts for "all" transfers
- Complete wallet context information
- Clear step descriptions with the calculated amounts

#### 3.2 Add YML Serialization for Wallet Context
Ensure the `YmlFlow` structure properly serializes to YAML with:
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

### 4. Improve the "All" Keyword Handling

#### 4.1 Better Integration with calculate_max_transferable_amount
Ensure the calculated amount from `calculate_max_transferable_amount` is properly preserved:
- In the refined prompt from `PromptProcessor`
- In the YML flow generated by `YmlGenerator`
- In the validation process
- In the expected_tool_calls with the exact calculated amount (e.g., "4.999")

#### 4.2 YML Structure for "All" to Specific Amount Conversion
When converting "send all sol to address" to "send 4.999 sol to address", the YML should include:
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

#### 4.3 Enhanced Refinement Process
Improve the refinement process to ensure:
- The "all" keyword is consistently replaced with the calculated amount
- The refined prompt is used consistently throughout the pipeline
- The calculated amount is included in all relevant parts of the YML flow
- The expected_tool_calls explicitly include the calculated amount for validation

#### 4.4 Full Example YML for "All" to Specific Amount Conversion

**Input:** "send all sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
**Process:** 
1. `calculate_max_transferable_amount` calculates 4.999 SOL from wallet balance of 5.0 SOL (reserving 0.001 SOL for gas)
2. `PromptProcessor` creates structured YML prompt and sends to LLM
3. LLM returns refined prompt: "send 4.999 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq"
4. `YmlGenerator` creates YML flow with this structure:

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

### 5. Add YML Validation Capabilities

#### 5.1 Structure Validation
Add functionality to validate the generated YML flows:
- Ensure the YML structure is valid
- Verify that the refined prompt matches the expected operation
- Check that the wallet context is properly included

#### 5.2 Content Validation
Validate the content of the YML flows:
- Ensure transfer amounts match between refined prompt and YML steps
- Verify that calculated amounts for "all" transfers are consistent
- Check that wallet context information is accurate

### 6. Update Test Coverage

#### 6.1 Update e2e_transfer.rs Test
Enhance the existing test to verify:
- The generated YML flow has the correct structure
- The wallet context is properly included
- The refined prompt contains the calculated amount for "all" transfers

#### 6.2 Add New Tests
Create additional tests to verify:
- TransferAmountRefinementRequest serialization to YAML
- YML structure validation
- "All" keyword handling across the entire pipeline
- Integration between `PromptProcessor` and `YmlGenerator`
- Proper error handling without fallbacks

## Implementation Steps

### Phase 1: Replace Unstructured Prompt with Structured YML
1. Define `TransferAmountRefinementRequest` struct with YAML serialization
2. Implement the `to_yaml()` method
3. Update `PromptProcessor::process_prompt` to use structured YML instead of unstructured text
4. Remove all fallback logic from `PromptProcessor::process_prompt` (lines 108-140)
5. Implement clean error handling that returns errors directly

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
2. Add tests for `TransferAmountRefinementRequest` serialization
3. Add tests to verify that no fallback logic is used
4. Ensure all tests pass with the enhanced implementation

### Phase 5: Integration Testing
1. Run full end-to-end tests with the enhanced implementation
2. Verify that the YML structure is correctly generated and validated
3. Ensure the transfer functionality works correctly for both specific amounts and "all" transfers
4. Verify that LLM responses are evaluated directly without fallbacks

## Expected Outcomes

After implementing this plan:

1. The YML flows will contain the complete wallet context structure needed for validation
2. The "all" keyword will be properly handled with the calculated amounts preserved throughout the pipeline
3. The generated YML flows will be properly structured and validated
4. The transfer functionality will work correctly for both specific amounts and "all" transfers
5. The test suite will validate the YML structure and content
6. **CRITICAL**: No fallback logic remains in the codebase, ensuring true LLM evaluation

## Success Criteria

1. The `e2e_transfer.rs` test passes for both specific amounts and "all" transfers
2. The generated YML flows contain the proper structure with wallet context
3. The refined prompts contain the calculated amounts for "all" transfers
4. The YML validation works correctly for both structure and content
5. No regression in the transfer functionality
6. **CRITICAL**: All fallback logic has been removed from `PromptProcessor`
7. **CRITICAL**: The system uses structured YML prompts instead of unstructured text
8. **CRITICAL**: LLM responses are evaluated directly without workarounds

## Future Considerations

1. This plan focuses on fixing the regression for transfer functionality
2. Similar enhancements may be needed for other operations (swap, lend, etc.)
3. The YML validation capabilities could be extended to support more complex validation rules
4. The approach could be generalized to support more complex multi-step operations
5. The structured YML prompt approach can be extended to other operations beyond transfers

## Conclusion

This plan addresses the regression in transfer functionality by:
1. Replacing unstructured prompts with structured YML prompts
2. Removing all fallback logic to ensure true LLM evaluation
3. Enhancing YML structure generation
4. Improving "all" keyword handling
5. Adding validation capabilities
6. Updating the test suite

The key changes are:
- Introducing `TransferAmountRefinementRequest` with structured YAML format
- Removing fallback logic from `PromptProcessor`
- Using structured YML prompts to guide the LLM

The implementation will restore functionality while maintaining the current architecture and adding the YML validation capabilities that were lost in the regression. Most importantly, it will do so using a true LLM-first approach that aligns with the core architecture plans and evaluation platform requirements, avoiding the pitfalls of rules-based implementations that are not scalable.