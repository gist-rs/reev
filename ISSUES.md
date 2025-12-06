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

## Issue #323 - FIXED

**Problem**: Multiple tests failing after ProtocolRegistry changes
- protocol_core_tests.rs test_protocol_registry failed
- benchmarks_test.rs failed

**Solution**: Updated tests to match new ProtocolRegistry behavior
- Modified tests in registry.rs and protocol_core_tests.rs to expect None for unsupported operations
- All tests now pass after removing fallback behavior in get_for_operation

**Files Changed**:
- crates/reev-core/src/protocols/registry.rs: Fixed test to expect None for unsupported operations
- crates/reev-core/tests/protocol_core_tests.rs: Fixed test to expect None for unsupported operations

**Tests Passing**: 
- All tests in protocol_core_tests.rs now pass
- All tests in registry.rs now pass
- benchmarks_test.rs continues to pass

## Issue #321
