# Reev Core Tasks

## Current Focus: Implement Structured LLM Response System

### Background

Currently, the system uses rule-based parsing to extract operations from refined prompts. This approach has limitations:
- Brittle regex matching that can fail with slight variations in prompts
- Difficult to extend with new operation types
- Inconsistent parameter extraction across different operations

The proposed solution is to have the LLM return structured data that includes:
1. Refined prompt text
2. Detected action type
3. Extracted addresses and parameters
4. Confidence metrics

### Implementation Plan

#### Phase 1: Define Data Structures

**Why**: Create the foundation for structured data handling
**Where**: `crates/reev-core/src/prompt_processor/types.rs`

**Tasks**:
1. Create `StructuredRefinedPrompt` struct with fields:
   - `refined_prompt`: String
   - `action`: `PromptAction` enum
   - `subject_pubkey`: Option<String>
   - `target_pubkey`: Option<String>
   - `parameters`: `PromptParameters` struct
   - `confidence`: f32

2. Create `PromptAction` enum with variants:
   - Transfer
   - Swap
   - Lend
   - Earn
   - Borrow
   - Unknown

3. Create `PromptParameters` struct with:
   - `amount`: Option<String>
   - `input_mint`: Option<String>
   - `output_mint`: Option<String>
   - Additional flexible parameters via `HashMap<String, serde_json::Value>`

#### Phase 2: Update Prompt Processing

**Why**: Modify LLM interaction to return structured data
**Where**: `crates/reev-core/src/prompt_processor/mod.rs`

**Tasks**:
1. Create new system prompt instructing LLM to respond with structured JSON
2. Update `process_prompt` to return `StructuredRefinedPrompt` instead of current `RefinedPrompt`
3. Add robust error handling:
   - Try parsing as `StructuredRefinedPrompt`
   - If fails, fall back to current approach
   - Log fallback cases for monitoring

#### Phase 3: Implement Validation Logic

**Why**: Ensure extracted data is accurate and reliable
**Where**: `crates/reev-core/src/prompt_processor/validation.rs`

**Tasks**:
1. Create validation functions:
   - Verify extracted pubkeys appear in original prompt
   - Validate action matches prompt intent
   - Check parameter consistency

2. Implement confidence scoring based on:
   - Match accuracy
   - Completeness of extraction
   - Internal consistency

3. Create fallback strategies:
   - If validation fails, retry with refined prompt
   - After 2 retries, fall back to current approach
   - Log all fallbacks for continuous improvement

#### Phase 4: Update Execution Flow

**Why**: Integrate structured data into execution pipeline
**Where**: `crates/reev-core/src/execution/rig_agent/mod.rs`

**Tasks**:
1. Modify `execute_step_with_rig_and_history` to use structured fields
2. Remove rule-based extraction from `extract_tool_calls`
3. Use structured `action` field directly for operation selection
4. Use extracted parameters directly instead of parsing text

#### Phase 5: Update Types and Interfaces

**Why**: Ensure type consistency across the system
**Where**: Multiple files

**Tasks**:
1. Update `FlowStep` in `crates/reev-core/src/benchmark/runner/types.rs`
2. Update YmlStep to include structured data
3. Update relevant test files to work with new structure

#### Phase 6: Comprehensive Testing

**Why**: Ensure reliability and prevent regressions
**Where**: `crates/reev-core/tests/`

**Tasks**:
1. Create new test file `structured_llm_test.rs` with:
   - Test for each action type
   - Tests for validation logic
   - Tests for fallback mechanisms

2. Update existing tests:
   - `e2e_transfer.rs`
   - `e2e_swap.rs`
   - `e2e_lend.rs`

3. Add property-based tests for validation logic

#### Phase 7: Documentation and Monitoring

**Why**: Ensure maintainability and observability
**Where**: Documentation and monitoring code

**Tasks**:
1. Update inline documentation
2. Add structured logging for debugging
3. Add metrics for:
   - Success rate of structured extraction
   - Frequency of fallbacks
   - Average confidence scores

### Implementation Details

#### Example LLM Prompt

```
You are an expert at analyzing blockchain operation prompts. 
Please analyze the user prompt and respond with structured JSON containing:

1. refined_prompt: A clearer version of the original prompt
2. action: The blockchain operation type (transfer, swap, lend, earn, borrow)
3. subject_pubkey: The wallet performing the action (from context if not in prompt)
4. target_pubkey: The destination address (for transfers/operations to others)
5. parameters: {
   amount: The amount to transfer/swap/lend,
   input_mint: The input token mint address,
   output_mint: The output token mint address
}
6. confidence: Your confidence in this extraction (0.0-1.0)

Example response:
{
  "refined_prompt": "send 1 sol to gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
  "action": "transfer",
  "subject_pubkey": "3F42CLVYyxuMYNTBRKuCQ6o3XnzPky6raWTPHtW8myLr",
  "target_pubkey": "gistmeAhMG7AcKSPCHis8JikGmKT9tRRyZpyMLNNULq",
  "parameters": {
    "amount": "1",
    "input_mint": "So11111111111111111111111111111111111111112"
  },
  "confidence": 0.95
}
```

#### Validation Logic Example

```rust
pub fn validate_structured_response(
    response: &StructuredRefinedPrompt,
    original_prompt: &str,
    wallet_context: &WalletContext,
) -> ValidationResult {
    let mut issues = Vec::new();
    
    // Check if target_pubkey appears in original prompt
    if let Some(target) = &response.target_pubkey {
        if !original_prompt.contains(target) {
            issues.push(format!(
                "Extracted target_pubkey {} not found in original prompt",
                target
            ));
        }
    }
    
    // Check if action matches prompt intent
    match response.action {
        PromptAction::Transfer => {
            if !original_prompt.to_lowercase().contains("transfer") &&
               !original_prompt.to_lowercase().contains("send") {
                issues.push("Action 'transfer' doesn't match prompt intent".to_string());
            }
        },
        // Additional validations for other actions...
        _ => {}
    }
    
    if issues.is_empty() {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid(issues)
    }
}
```

### Success Criteria

1. Improved operation extraction accuracy from ~85% to >95%
2. Reduced execution time by eliminating regex parsing
3. Enhanced extensibility for new operation types
4. Comprehensive test coverage (>90%)
5. Zero regressions in existing functionality

### Timeline

- Phase 1-2: 2-3 days
- Phase 3-4: 3-4 days
- Phase 5-6: 2-3 days
- Phase 7: 1-2 days
- Total: 8-12 days

### Dependencies

- Completion of current rig_agent refactoring
- Access to LLM API for testing
- Production environment for monitoring setup
