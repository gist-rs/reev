# Standardizing E2E Test Implementations and Creating Shared Utilities

## Alignment with PLAN_CORE_V3.md

This task plan aligns with the migration strategy outlined in PLAN_CORE_V3.md:
- Supports **Phase 1: Remove Duplication and Clean Up** (Week 1)
- Aligns with **Phase 2: Improve Scalability** (Week 1-2) ✅ 
- Uses the single-step template approach already implemented
- Extracts reusable utilities to reduce code duplication
- Supports the composable step-based system described in the plan
- Aligns with the YML structure defined in PLAN_CORE_V3.md
- Supports the validation framework from Phase 3
- Implements the ground truth structure for final_state_assertions
- Supports benchmark evaluation framework from PLAN_CORE_BENCHMARK.md

## Overview
We need to standardize end-to-end test implementations for `e2e_transfer.rs`, `e2e_swap.rs`, and `e2e_lend.rs` to ensure consistency. Additionally, we should extract common patterns to shared utilities in the main `crates/reev-core/src` directory since these will be used by other components like `reev-api` and `reev-runner` for processing user prompts and executing benchmarks.

## Implementation Strategy

We will follow a sequential refactoring approach:

1. **Phase 1: Refactor e2e_transfer.rs first**
   - Extract utilities from e2e_transfer.rs to main crate
   - Refactor e2e_transfer.rs to use the new utilities
   - Ensure e2e_transfer.rs works with the new approach

2. **Phase 2: Apply the same pattern to e2e_swap.rs and e2e_lend.rs**
   - Reuse the utilities created in Phase 1
   - Apply the same refactoring approach
   - Ensure all tests follow the same structure

This approach allows us to:
- Validate the utility design with a single test first
- Document the extraction patterns clearly for reuse
- Prepare for e2e_swap.rs refactoring by understanding the patterns from e2e_transfer.rs
- Make incremental progress while maintaining test functionality

## Current Issues

### e2e_swap.rs
- Creates a YML prompt but never uses it (stored in `_yml_prompt` variable)
- Inconsistent naming compared to `e2e_transfer.rs`
- Documentation mentions YML prompt but implementation doesn't reflect this
- Unused code suggests incomplete implementation

### e2e_transfer.rs
- Better implementation with proper YML prompt usage
- More complete documentation of the 6-step process
- Contains patterns that should be promoted to main crate for reuse

### Code Duplication
- Common patterns are duplicated across test files
- YML prompt creation, signature extraction, and test flow logic could be shared
- These utilities will be needed by other components (API, runner) beyond just tests

## Common Patterns to Extract

### 1. YML Prompt Creation
The wallet_info formatting in e2e_transfer.rs is a pattern that should be promoted to the main crate:
```rust
let wallet_info = format!(
    "subject_wallet_info:\n  - pubkey: \"{from_pubkey}\"\n    lamports: {initial_sol_balance} # {formatted_balance} SOL\n    total_value_usd: 170\n\nsteps:\n  prompt: \"{prompt}\"\n    intent: \"send\"\n    context: \"Executing a SOL transfer using Solana system instructions\"\n    recipient: \"{TARGET_PUBKEY}\""
);
```

### 2. Transaction Signature Extraction
Both tests have similar but slightly different signature extraction logic that should be unified in the main crate.

### 3. Common Execution Flow
The 6-step process is documented in tests and should be standardized in the main crate for reuse by other components.

## Tasks

## Phase 1: Refactor e2e_transfer.rs

### 1. Create Shared Utilities in Main Crate (from e2e_transfer.rs)
- [ ] Create a new module `crates/reev-core/src/yml_utils.rs` for YML prompt utilities
  - [ ] Extract wallet_info formatting from e2e_transfer.rs 
  - [ ] Create a generic function for creating subject_wallet_info YML section
  - [ ] Make it flexible enough for different transaction types (transfer, swap, lend)
  - [ ] Align with PLAN_CORE_V3.md YML structure for subject_wallet_info including lamports and tokens
  - [ ] Add documentation for YML format requirements per PLAN_CORE_V3
  - [ ] Support the flow_id, user_prompt, and refined_prompt fields from PLAN_CORE_V3
- [ ] Create a new module `crates/reev-core/src/result_utils.rs` for result processing utilities
  - [ ] Extract transaction signature extraction logic from e2e_transfer.rs
  - [ ] Create a unified function that handles all possible response formats
  - [ ] Make it robust enough to handle different tool types (transfer, swap, lend)
  - [ ] Add error handling for missing signatures
  - [ ] Align with validation framework from PLAN_CORE_V3.md Phase 3
- [ ] Create a new module `crates/reev-core/src/flow_utils.rs` for execution flow utilities
  - [ ] Extract common execution flow steps from e2e_transfer.rs
  - [ ] Create functions for each step of the 6-step process
  - [ ] Make them composable for different scenarios (test, API, runner)
  - [ ] Align with the 6-step flow from PLAN_CORE_V3.md (Phase 1 and Phase 2)
  - [ ] Support the ground truth validation structure from PLAN_CORE_V3.md
  - [ ] Implement the expected_tools and critical flags for step validation
  - [ ] Support benchmark YML structure from PLAN_CORE_BENCHMARK.md
    - [ ] Include success_criteria, weight, and required fields
    - [ ] Support expected_flow_complexity metrics
    - [ ] Implement expected_multiplication_metrics for advanced flows
    - [ ] Support expected_otel_tracking for observability

### 2. Refactor e2e_transfer.rs to Use New Utilities
- [ ] Update e2e_transfer.rs to use the new yml_utils module
- [ ] Update e2e_transfer.rs to use the new result_utils module
- [ ] Update e2e_transfer.rs to use the new flow_utils module
- [ ] Ensure e2e_transfer.rs follows the same 6-step process
- [ ] Verify that all functionality still works

### 3. Document Patterns for Reuse
- [ ] Document the extraction patterns used for e2e_transfer.rs
- [ ] Create guidelines for applying the same patterns to other tests
- [ ] Prepare documentation for how to handle differences between tests

## Phase 2: Apply to e2e_swap.rs and e2e_lend.rs

### 4. Refactor e2e_swap.rs Using Same Patterns
- [ ] Update e2e_swap.rs to use the yml_utils module (fixing the unused _yml_prompt issue)
- [ ] Update e2e_swap.rs to use the result_utils module (unifying signature extraction)
- [ ] Update e2e_swap.rs to use the flow_utils module (standardizing 6-step process)
- [ ] Ensure e2e_swap.rs follows the same structure as e2e_transfer.rs
- [ ] Verify that all functionality still works

### 5. Refactor e2e_lend.rs Using Same Patterns
- [ ] Update e2e_lend.rs to use the yml_utils module
- [ ] Update e2e_lend.rs to use the result_utils module
- [ ] Update e2e_lend.rs to use the flow_utils module
- [ ] Ensure e2e_lend.rs follows the same structure as other tests
- [ ] Verify that all functionality still works

### 3. Standardize YML Prompt Handling
- [ ] Fix `e2e_swap.rs` to properly use the YML prompt
- [ ] Ensure all tests follow the same pattern for YML prompt creation and usage
- [ ] Make sure the 6-step process documentation matches the actual implementation in all tests
- [ ] Ensure API and runner also follow the same YML prompt format

### 4. Unify Execution Flow
- [ ] Ensure all tests resolve wallet context in the same way
- [ ] Make the planner initialization identical between tests
- [ ] Standardize executor initialization and flow execution
- [ ] Use the shared signature extraction function

### 5. Clean Up Code
- [ ] Remove unused variables in all test files
- [ ] Ensure consistent error handling patterns across tests and crates
- [ ] Make logging patterns identical between tests and other components
- [ ] Standardize comments and documentation style across all files

### 6. Improve Documentation
- [ ] Ensure all tests have the same level of documentation
- [ ] Make the 6-step process documentation match actual implementation
- [ ] Add consistent header documentation to all test files
- [ ] Ensure examples and usage instructions are identical in format

### 7. Verify Functionality
- [ ] Ensure all tests pass after standardization
- [ ] Verify that the shared utilities work correctly for all test cases
- [ ] Check that all tests handle edge cases in the same way
- [ ] Test API endpoints with new utilities
- [ ] Test runner benchmarks with new utilities
- [ ] Run full test suite to ensure no regressions

## Expected Outcome
After completing these tasks:

### Shared Utilities in Main Crate
- `yml_utils.rs` module for standardized YML prompt creation
- `result_utils.rs` module for unified result and signature processing
- `flow_utils.rs` module for standardized execution flow steps
- All utilities documented and ready for use by tests, API, and runner

### Standardized Test Files
- All e2e tests (`e2e_transfer.rs`, `e2e_swap.rs`, `e2e_lend.rs`) will have:
  - Consistent structure and implementation
  - Identical patterns for YML prompt handling
  - Same execution flow for wallet context, planning, and execution
  - Unified documentation and comments
  - Significantly reduced code duplication

### Updated Components
- `reev-api` will use standardized YML creation and result processing
- `reev-runner` will use standardized YML creation and result processing
- All components will follow the same patterns for handling prompts and results
- Benchmarks can use standardized YML structure for evaluation
- Results can be evaluated against ground truth using unified framework

### Benefits of Sequential Approach
- Validate utility design with a single test first, reducing risk
- Clear patterns established for reuse in other tests
- Incremental progress while maintaining test functionality
- Better understanding of edge cases before applying to all tests
- Easier to debug issues with a smaller scope first
- Supports the modular architecture outlined in PLAN_CORE_V3.md
- Aligns with the migration strategy to remove duplication

### Overall Benefits
- Single source of truth for YML prompt creation and result processing
- Consistent behavior across tests, API, and runner
- Faster development of new features by reusing common patterns
- Single place to update when changing core functionality
- Better separation of concerns with dedicated utility modules
- Alignment with benchmark evaluation framework from PLAN_CORE_BENCHMARK.md
- Support for complex scoring algorithms and success criteria
- Foundation for deterministic verification of AI-generated flows
