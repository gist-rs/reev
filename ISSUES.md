# Reev Project Issues

## Current Issues (last 10)

### 1. Inconsistent Function Naming in E2E Tests (FIXED)
**Status**: Fixed
**Description**: E2E test functions had inconsistent naming conventions despite using the same implementation approach
- `execute_transfer_with_rig_agent` in e2e_transfer.rs
- `execute_swap_with_planner` in e2e_swap.rs  
- `execute_lend_with_planner` in e2e_lend.rs

**Fix**: Renamed all functions to `execute_XXX_with_standardized_flow` to accurately reflect that they all use the standardized 6-step flow
**Files Modified**: e2e_transfer.rs, e2e_swap.rs, e2e_lend.rs

## Current Status

### Standardized E2E Tests
- **Phase 1**: ✅ Completed - Refactored e2e_transfer.rs to use shared utilities
- **Phase 2**: ✅ Completed - Applied same patterns to e2e_swap.rs and e2e_lend.rs
- **Naming Consistency**: ✅ Fixed - All execution functions now use standardized naming

### Shared Utilities Created
- `yml_utils.rs` - Standardized YML prompt creation with PLAN_CORE_V3.md compliance
- `result_utils.rs` - Unified transaction signature extraction for all tool types
- `flow_utils.rs` - Standardized 6-step flow execution process
- `utils/mod.rs` - Module index and re-exports

### Architecture Alignment
- YML structure follows PLAN_CORE_V3.md specification exactly
- 6-step flow implementation matches Phase 1 and Phase 2 requirements
- Supports validation framework from Phase 3
- Ground truth structure implemented for evaluation

### Next Steps for Future Work
- Phase 3-5 from TASKS.md (Integrate Validation, Error Recovery, etc.)
- Apply utilities to other components (API, runner)
- Implement benchmark YML structure from PLAN_CORE_BENCHMARK.md
- Add more comprehensive validation framework

## Verification
- All e2e tests pass successfully with standardized utilities
- Function names now accurately reflect implementation approach
- No clippy warnings in the project
- All borrow checker issues resolved