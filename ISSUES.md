# Reev Project Issues

## Issue #322 - FIXED

**Problem**: test_protocol_registry_integration was failing because `get_for_operation` was returning a fallback protocol for unsupported operations instead of None.

**Solution**: Modified `get_for_operation` in ProtocolRegistry to return None for unsupported operations instead of returning the first registered protocol as a fallback.

**Files Changed**:
- crates/reev-core/src/protocols/registry.rs: Changed fallback behavior in get_for_operation method

**Tests Passing**: 
- test_protocol_registry_integration in benchmark_runner_tests.rs now passes
- All other tests in benchmark_runner_tests.rs continue to pass
- benchmarks_test.rs continues to pass

## Issue #321
