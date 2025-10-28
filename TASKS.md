# API Decoupling Tasks - CLI-Based Runner Communication

## Phase 1: Foundation (Week 1)

### 1.1 reev-types Crate ✅ COMPLETED
**Status**: Done
**Files Created**:
- `crates/reev-types/Cargo.toml`
- `crates/reev-types/src/lib.rs`
- `crates/reev-types/src/rpc.rs` - JSON-RPC 2.0 structures
- `crates/reev-types/src/execution.rs` - Execution state management
- `crates/reev-types/src/benchmark.rs` - Shared benchmark types
- `crates/reev-types/src/runner.rs` - CLI command/response types

### 1.2 Runner Process Manager - HIGH PRIORITY
**File**: `crates/reev-api/src/services/runner_manager.rs`
**Tasks**:
- [ ] Create `RunnerProcessManager` struct with configuration
- [ ] Implement `execute_benchmark()` method for CLI execution
- [ ] Add `build_runner_command()` to construct CLI arguments
- [ ] Implement `execute_cli_command()` with timeout handling
- [ ] Add process cleanup and resource management
- [ ] Integrate with existing database writer

### 1.3 CLI Execution Wrapper - HIGH PRIORITY
**File**: `crates/reev-api/src/services/cli_executor.rs`
**Tasks**:
- [ ] Create `CliExecutor` for process management
- [ ] Implement `execute_with_timeout()` method
- [ ] Add stdout/stderr capture and parsing
- [ ] Create JSON-RPC request/response serialization
- [ ] Add process status monitoring
- [ ] Implement graceful shutdown handling

### 1.4 Execution State Management - HIGH PRIORITY
**File**: `crates/reev-db/src/writer/execution_states.rs`
**Tasks**:
- [ ] Create `execution_states` table schema
- [ ] Implement `create_execution_state()` method
- [ ] Add `update_execution_state()` with status transitions
- [ ] Create `get_execution_state()` for status queries
- [ ] Implement `cleanup_expired_states()` for maintenance
- [ ] Add indexes for performance optimization

## Phase 2: Integration (Week 1-2)

### 2.1 JSON-RPC Protocol Implementation - MEDIUM PRIORITY
**File**: `crates/reev-api/src/services/rpc_client.rs`
**Tasks**:
- [ ] Create `JsonRpcClient` for communication
- [ ] Implement `send_request()` with request ID correlation
- [ ] Add `parse_response()` for structured error handling
- [ ] Create `handle_notification()` for async updates
- [ ] Implement connection pooling for multiple requests
- [ ] Add request/response logging for debugging

### 2.2 Database Integration for State Sync - MEDIUM PRIORITY
**Files**:
- `crates/reev-api/src/services/state_sync.rs`
- `crates/reev-db/src/writer/state_sync.rs`
**Tasks**:
- [ ] Implement state synchronization between processes
- [ ] Create conflict resolution for concurrent updates
- [ ] Add state change notifications
- [ ] Implement retry logic for database failures
- [ ] Create state validation and consistency checks
- [ ] Add migration scripts for new tables

### 2.3 Error Handling and Recovery - MEDIUM PRIORITY
**File**: `crates/reev-api/src/services/recovery_manager.rs`
**Tasks**:
- [ ] Create `RunnerError` enum with specific error types
- [ ] Implement `handle_process_failure()` with fallback logic
- [ ] Add `recover_orphaned_executions()` for cleanup
- [ ] Create retry mechanisms with exponential backoff
- [ ] Implement circuit breaker pattern for failures
- [ ] Add comprehensive error logging and metrics

## Phase 3: API Migration (Week 2-3)

### 3.1 Read-Only Endpoint Migration - LOW RISK
**Files**: Update existing handlers in `crates/reev-api/src/handlers/`
**Tasks**:
- [ ] Migrate `list_benchmarks()` to CLI-based discovery
- [ ] Update `list_agents()` to query runner process
- [ ] Enhance `health_check()` with runner availability
- [ ] Add `debug_benchmarks()` with CLI integration
- [ ] Update agent performance queries
- [ ] Test all migrated endpoints with CURL.md

### 3.2 Status and Control Endpoints - MEDIUM RISK
**Files**: Update existing handlers in `crates/reev-api/src/handlers/`
**Tasks**:
- [ ] Migrate `get_execution_status()` to state-based queries
- [ ] Update `stop_benchmark()` with process termination
- [ ] Add execution progress tracking
- [ ] Implement real-time status updates
- [ ] Create status history and audit trail
- [ ] Add timeout handling for long-running operations

### 3.3 Write Operation Migration - HIGH RISK
**Files**: Update existing handlers in `crates/reev-api/src/handlers/`
**Tasks**:
- [ ] Migrate `run_benchmark()` to CLI execution
- [ ] Update flow log generation and retrieval
- [ ] Migrate transaction log endpoints
- [ ] Add execution trace collection
- [ ] Implement concurrent execution management
- [ ] Create rollback mechanisms for failed operations

## Phase 4: Testing and Validation (Week 3-4)

### 4.1 CLI Testing Framework - HIGH PRIORITY
**File**: `tests/cli_runner_test.rs`
**Tasks**:
- [ ] Create `CliRunnerTest` framework
- [ ] Implement `test_cli_benchmark_execution()`
- [ ] Add `test_timeout_handling()`
- [ ] Create `test_error_scenarios()`
- [ ] Implement `test_concurrent_executions()`
- [ ] Add performance comparison tests

### 4.2 Integration Tests with CURL.md - MEDIUM PRIORITY
**File**: Update `CURL.md` with new test commands
**Tasks**:
- [ ] Add CLI-based benchmark execution commands
- [ ] Create status checking test scenarios
- [ ] Add error handling test commands
- [ ] Implement batch operation tests
- [ ] Create performance benchmark commands
- [ ] Add regression test suite

### 4.3 Performance and Load Testing - MEDIUM PRIORITY
**File**: `tests/performance_test.rs`
**Tasks**:
- [ ] Create `benchmark_direct_vs_cli()` comparison
- [ ] Implement `test_concurrent_load()` scenarios
- [ ] Add memory usage profiling
- [ ] Create response time regression tests
- [ ] Implement resource usage monitoring
- [ ] Add scalability testing framework

## Phase 5: Cleanup and Optimization (Week 4)

### 5.1 Dependency Removal - HIGH PRIORITY
**Files**: `crates/reev-api/Cargo.toml`, `crates/reev-api/src/main.rs`
**Tasks**:
- [ ] Remove `reev-runner` dependency from Cargo.toml
- [ ] Remove `reev-flow` dependency from Cargo.toml
- [ ] Remove `reev-tools` dependency from Cargo.toml
- [ ] Add `reev-types` dependency
- [ ] Update imports in `main.rs` and handlers
- [ ] Clean up unused code and imports

### 5.2 Configuration Management - MEDIUM PRIORITY
**Files**: `crates/reev-api/src/config/`
**Tasks**:
- [ ] Create `RunnerConfig` structure
- [ ] Add environment variable handling
- [ ] Implement configuration validation
- [ ] Create development/production presets
- [ ] Add configuration hot-reloading
- [ ] Document all configuration options

### 5.3 Monitoring and Observability - LOW PRIORITY
**Files**: `crates/reev-api/src/metrics/`
**Tasks**:
- [ ] Create `RunnerMetrics` collection
- [ ] Add Prometheus metrics export
- [ ] Implement performance dashboards
- [ ] Create alerting for process failures
- [ ] Add distributed tracing support
- [ ] Document monitoring procedures

## Phase 6: Documentation and Deployment (Week 4)

### 6.1 Documentation Updates - MEDIUM PRIORITY
**Files**: Update existing documentation
**Tasks**:
- [ ] Update `ARCHITECTURE.md` with new architecture
- [ ] Update `PLAN.md` with completion status
- [ ] Create deployment guide for CLI runner
- [ ] Update `README.md` with new features
- [ ] Document migration steps for users
- [ ] Create troubleshooting guide

### 6.2 Deployment Preparation - LOW PRIORITY
**Files**: Deployment configurations
**Tasks**:
- [ ] Create Docker configurations for runner separation
- [ ] Add environment variable templates
- [ ] Create deployment scripts
- [ ] Add health check endpoints
- [ ] Create monitoring setup
- [ ] Document rollback procedures

## Success Criteria Checklist

### Functional Requirements
- [ ] All existing API endpoints work with CLI runner
- [ ] No regression in benchmark execution results
- [ ] Graceful error handling and recovery
- [ ] Performance within 20% of direct library calls
- [ ] All CURL.md tests pass with new implementation

### Architectural Requirements
- [ ] Eliminate reev-runner, reev-flow, reev-tools dependencies
- [ ] Clean separation via reev-types
- [ ] State-based communication through reev-db
- [ ] Modular, testable components
- [ ] No compilation warnings or errors

### Operational Requirements
- [ ] Proper logging and monitoring
- [ ] Configurable timeouts and limits
- [ ] Development and production deployment strategies
- [ ] Comprehensive test coverage (>90%)
- [ ] Documentation completeness

## Issues and Risks

### High Risk Items
1. **Performance Degradation**: CLI calls may be slower than direct library calls
2. **Process Management Complexity**: Managing multiple concurrent processes
3. **Error Handling**: Process failures vs. library call failures
4. **State Synchronization**: Race conditions between processes

### Mitigation Strategies
1. **Performance**: Implement process pooling and connection reuse
2. **Process Management**: Use robust process lifecycle management
3. **Error Handling**: Comprehensive error categorization and recovery
4. **State Sync**: Use database transactions and proper locking

### Rollback Plan
- Keep direct library calls as fallback option
- Feature flags to switch between implementations
- Database schema changes are backward compatible
- API contract remains unchanged

## Testing Requirements

### Unit Tests
- All new modules must have >90% test coverage
- Mock process execution for reliable testing
- Test error scenarios and edge cases
- Performance benchmarks for critical paths

### Integration Tests
- End-to-end API functionality tests
- CLI process integration tests
- Database state synchronization tests
- Error handling and recovery tests

### Manual Testing
- CURL.md command verification
- Web interface functionality tests
- Performance load testing
- User acceptance testing

## Dependencies and Blockers

### External Dependencies
- Cargo build system for CLI execution
- Process management libraries
- Database connection pooling
- JSON-RPC protocol implementation

### Internal Dependencies
- reev-types crate completion ✅
- Database schema updates
- Existing handler modifications
- Test framework setup

### Timeline Constraints
- Week 1: Foundation components
- Week 2: Integration and testing
- Week 3: API migration
- Week 4: Cleanup and deployment

## Next Steps

1. **Immediate**: Start with RunnerProcessManager implementation
2. **Week 1**: Complete foundation components and basic CLI execution
3. **Week 2**: Integrate with database and implement state management
4. **Week 3**: Migrate API endpoints progressively
5. **Week 4**: Remove dependencies and optimize performance

## Notes for Implementation

### Code Organization
- Keep each module under 320 lines as per project rules
- Use proper error handling with `Result` types
- Follow Rust naming conventions and best practices
- Add comprehensive logging for debugging

### Testing Strategy
- Test each component in isolation before integration
- Use feature flags for gradual migration
- Maintain backward compatibility during transition
- Document all test scenarios and expected outcomes

### Performance Considerations
- Monitor CLI execution overhead
- Implement process pooling for efficiency
- Cache frequently accessed data
- Use async/await for concurrent operations