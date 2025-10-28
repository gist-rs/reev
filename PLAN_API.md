# API Decoupling Plan - CLI-Based Runner Communication

## Overview

Transform reev-api from directly importing and building reev-runner to calling it as a CLI process with JSON-RPC-like communication through reev-db state management.

## Current Architecture Problems

```toml
# crates/reev-api/Cargo.toml - DEPENDENCIES TO REMOVE
reev-lib = { path = "../reev-lib", features = ["database"] }
reev-db = { path = "../reev-db" }
reev-runner = { path = "../reev-runner" }           # ❌ REMOVE
reev-flow = { path = "../reev-flow", features = ["database"] }  # ❌ REMOVE
reev-tools = { path = "../reev-tools" }            # ❌ REMOVE
```

### Current Direct Dependencies (Must be eliminated)
- `reev-runner`: Direct library import, built into API binary
- `reev-flow`: Complex flow orchestration logic
- `reev-tools`: Tool implementations and utilities

### Desired Decoupled Architecture
```
reev-api (web server)
    ↓ (CLI calls, JSON-RPC)
reev-runner (standalone process)
    ↓ (state communication)
reev-db (shared state)
```

## Phase 1: Foundation - Shared Types & Communication

### 1.1 Create reev-types Crate ✅ COMPLETED
- **Purpose**: Shared type definitions only, no logic
- **Location**: `/crates/reev-types/`
- **Modules**:
  - `rpc.rs`: JSON-RPC 2.0 request/response structures
  - `execution.rs`: Execution state and status management
  - `benchmark.rs`: Benchmark and agent information types
  - `runner.rs`: CLI command and response types

### 1.2 JSON-RPC Communication Protocol
```rust
// Standard JSON-RPC 2.0 over stdin/stdout
{
  "jsonrpc": "2.0",
  "id": "uuid-v4",
  "method": "runner.run_benchmark",
  "params": {
    "benchmark_path": "benchmarks/001-sol-transfer.yml",
    "agent": "glm-4.6",
    "shared_surfpool": false
  }
}
```

### 1.3 State-Based Communication via Database
```sql
-- Execution states table for inter-process communication
CREATE TABLE execution_states (
    execution_id TEXT PRIMARY KEY,
    benchmark_id TEXT NOT NULL,
    agent TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    progress REAL,
    error_message TEXT,
    result_data JSON,
    metadata JSON
);
```

## Phase 2: CLI Process Integration

### 2.1 Runner Process Manager
**File**: `crates/reev-api/src/services/runner_manager.rs`

```rust
pub struct RunnerProcessManager {
    config: RunnerConfig,
    db: PooledDatabaseWriter,
    timeout_config: TimeoutConfig,
}

impl RunnerProcessManager {
    pub async fn execute_benchmark(&self, params: RunBenchmarkParams) -> Result<String> {
        let execution_id = params.execution_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Queue execution in database
        self.create_execution_state(&execution_id, &params).await?;
        
        // Execute CLI command
        let command = self.build_runner_command(&execution_id, &params);
        let result = self.execute_cli_command(command).await?;
        
        // Update state based on result
        self.update_execution_state(&execution_id, result).await?;
        
        Ok(execution_id)
    }
}
```

### 2.2 CLI Command Execution
```bash
# Instead of direct library call:
reev_runner::run_benchmarks(path, agent, shared_surfpool, true).await?;

# Use CLI process:
RUST_LOG=info cargo run -p reev-runner -- \
  benchmarks/001-sol-transfer.yml \
  --agent glm-4.6 \
  --execution-id runner-uuid-v4 \
  --db-path db/reev_results.db
```

### 2.3 Timeout & Error Handling
```rust
pub struct ProcessTimeout {
    pub default_seconds: u64,
    pub max_seconds: u64,
    pub status_check_interval: Duration,
}

impl ProcessTimeout {
    pub async fn execute_with_timeout<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match timeout(Duration::from_secs(self.default_seconds), future).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Process execution timed out after {} seconds", self.default_seconds)),
        }
    }
}
```

## Phase 3: API Migration Strategy

### 3.1 Migration Stages

#### Stage 1: Hybrid Mode (Current + CLI)
- Keep existing imports for fallback
- Add CLI execution as alternative path
- Test CLI path with feature flag `USE_CLI_RUNNER`

#### Stage 2: CLI Primary, Import Fallback
- Make CLI execution the default
- Keep imports for edge cases and testing
- Gradual migration of endpoints

#### Stage 3: Full CLI Only
- Remove all direct imports
- Implement proper error handling for process failures
- Complete dependency elimination

### 3.2 Endpoint Migration Order

#### Low Risk First (Read Operations)
1. `GET /api/v1/benchmarks` - List available benchmarks
2. `GET /api/v1/agents` - List available agents
3. `GET /api/v1/health` - Health check

#### Medium Risk (Stateful Operations)
4. `GET /api/v1/benchmarks/{id}/status/{execution_id}` - Status checking
5. `POST /api/v1/benchmarks/{id}/stop/{execution_id}` - Stop execution

#### High Risk (Write Operations)
6. `POST /api/v1/benchmarks/{id}/run` - Benchmark execution
7. Flow logs and transaction endpoints

### 3.3 Backward Compatibility Plan

```rust
// Feature flag for gradual migration
#[cfg(feature = "use_cli_runner")]
pub async fn run_benchmark_cli(params: RunBenchmarkParams) -> Result<String> {
    runner_manager.execute_benchmark(params).await
}

#[cfg(not(feature = "use_cli_runner"))]
pub async fn run_benchmark_direct(params: RunBenchmarkParams) -> Result<String> {
    // Existing direct library call
    reev_runner::run_benchmarks(...).await?;
}
```

## Phase 4: Testing Strategy

### 4.1 CLI Testing Framework
**File**: `tests/cli_runner_test.rs`

```rust
#[tokio::test]
async fn test_cli_benchmark_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Start runner process
    let mut runner = RunnerProcess::new(&db_path);
    
    // Execute benchmark via CLI
    let result = runner
        .execute_command(RunnerCommand::RunBenchmark {
            benchmark_path: "benchmarks/001-sol-transfer.yml".to_string(),
            agent: "deterministic".to_string(),
            shared_surfpool: false,
        })
        .await
        .expect("CLI execution failed");
    
    assert!(result.is_success());
    assert!(result.stdout.contains("completed successfully"));
}
```

### 4.2 Integration Tests with CURL.md
Add new CURL commands for testing CLI path:

```bash
# Test CLI-based benchmark execution
curl -X POST http://localhost:3001/api/v1/benchmarks/001-sol-transfer/run \
  -H "Content-Type: application/json" \
  -d '{
    "agent": "deterministic",
    "use_cli": true,
    "timeout_seconds": 120
  }'

# Check execution status (works with both paths)
curl -X GET http://localhost:3001/api/v1/benchmarks/001-sol-transfer/status/{execution_id}
```

### 4.3 Performance Benchmarking
```rust
// Compare execution times
#[tokio::test]
async fn benchmark_direct_vs_cli() {
    let iterations = 10;
    
    // Measure direct library calls
    let direct_times = measure_execution_time(|| {
        reev_runner::run_benchmarks_directly(...)
    }, iterations).await;
    
    // Measure CLI calls
    let cli_times = measure_execution_time(|| {
        execute_runner_cli(...)
    }, iterations).await;
    
    // CLI should be within 20% of direct performance
    assert!(cli_times.average() <= direct_times.average() * 1.2);
}
```

## Phase 5: Error Handling & Recovery

### 5.1 Process Failure Scenarios

#### Scenario 1: Runner Binary Not Found
```rust
pub enum RunnerError {
    BinaryNotFound(String),
    ProcessTimeout(String),
    ProcessCrashed(i32, String),
    InvalidResponse(String),
    DatabaseError(anyhow::Error),
}

impl RunnerProcessManager {
    pub async fn handle_process_failure(&self, error: RunnerError) -> Result<()> {
        match error {
            RunnerError::BinaryNotFound(path) => {
                tracing::error!("Runner binary not found at: {}", path);
                // Fallback to direct import if available
                self.try_fallback_execution().await
            }
            RunnerError::ProcessTimeout(execution_id) => {
                tracing::warn!("Process timed out: {}", execution_id);
                self.mark_execution_timeout(&execution_id).await?;
                self.cleanup_process_resources().await?;
                Ok(())
            }
            // ... other error handling
        }
    }
}
```

#### Scenario 2: Database Communication Failure
```rust
impl RunnerProcessManager {
    pub async fn execute_with_state_sync<F, R>(&self, 
        execution_id: &str, 
        operation: F
    ) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        // Set initial state
        self.set_status(execution_id, ExecutionStatus::Running).await?;
        
        match operation() {
            Ok(result) => {
                self.set_status(execution_id, ExecutionStatus::Completed).await?;
                Ok(result)
            }
            Err(error) => {
                self.set_error(execution_id, error.to_string()).await?;
                Err(error)
            }
        }
    }
}
```

### 5.2 Recovery Mechanisms
```rust
pub struct RecoveryManager {
    db: PooledDatabaseWriter,
    max_retries: u32,
    retry_delay: Duration,
}

impl RecoveryManager {
    pub async fn recover_orphaned_executions(&self) -> Result<()> {
        let orphaned = self.db
            .find_orphaned_executions(Duration::from_secs(300))
            .await?;
            
        for execution in orphaned {
            match execution.status {
                ExecutionStatus::Running => {
                    // Check if process still exists
                    if !self.process_exists(&execution.execution_id)? {
                        self.mark_execution_failed(&execution.execution_id, "Process died").await?;
                    }
                }
                ExecutionStatus::Queued => {
                    // Re-queue if too old
                    if execution.created_at.elapsed() > Duration::from_secs(60) {
                        self.retry_execution(&execution.execution_id).await?;
                    }
                }
                _ => {} // Other statuses don't need recovery
            }
        }
        
        Ok(())
    }
}
```

## Phase 6: Configuration & Deployment

### 6.1 Configuration Changes
```toml
# crates/reev-api/Cargo.toml - Final state
[dependencies]
reev-lib = { path = "../reev-lib", features = ["database"] }
reev-db = { path = "../reev-db" }
reev-types = { path = "../reev-types" }  # ✅ NEW: Shared types only
# ❌ REMOVED: reev-runner, reev-flow, reev-tools

[features]
default = ["cli_runner"]
cli_runner = []           # Use CLI-based runner
direct_runner = []        # Use direct library calls (testing only)
```

### 6.2 Environment Variables
```bash
# Runner configuration
RUNNER_BINARY_PATH="cargo run -p reev-runner --"
RUNNER_WORKING_DIR="."
RUNNER_TIMEOUT_SECONDS=300
RUNNER_MAX_CONCURRENT=5

# Feature flags
USE_CLI_RUNNER=true
ENABLE_FALLBACK_MODE=false
```

### 6.3 Deployment Considerations

#### Development Environment
```bash
# Use cargo run for quick iteration
export RUNNER_BINARY_PATH="cargo run -p reev-runner --"
export USE_CLI_RUNNER=true
```

#### Production Environment
```bash
# Use pre-compiled binary for performance
export RUNNER_BINARY_PATH="/usr/local/bin/reev-runner"
export RUNNER_WORKING_DIR="/app"
export RUNNER_TIMEOUT_SECONDS=600
```

## Phase 7: Monitoring & Observability

### 7.1 Process Metrics
```rust
pub struct RunnerMetrics {
    pub process_start_count: u64,
    pub process_success_count: u64,
    pub process_failure_count: u64,
    pub process_timeout_count: u64,
    pub average_execution_time_ms: f64,
    pub concurrent_processes: u32,
}

impl RunnerProcessManager {
    pub fn collect_metrics(&self) -> RunnerMetrics {
        RunnerMetrics {
            process_start_count: self.metrics.process_start_count.get(),
            process_success_count: self.metrics.process_success_count.get(),
            process_failure_count: self.metrics.process_failure_count.get(),
            process_timeout_count: self.metrics.process_timeout_count.get(),
            average_execution_time_ms: self.metrics.calculate_average_time(),
            concurrent_processes: self.active_processes.len() as u32,
        }
    }
}
```

### 7.2 Health Check Enhancement
```rust
pub async fn enhanced_health_check() -> HealthStatus {
    let mut status = HealthStatus::healthy();
    
    // Check runner binary availability
    if !runner_manager.is_runner_available().await {
        status.add_warning("Runner binary not found");
    }
    
    // Check database connectivity
    if !db.is_healthy().await {
        status.add_error("Database connection failed");
    }
    
    // Check process manager state
    let metrics = runner_manager.collect_metrics().await;
    if metrics.concurrent_processes > metrics.max_concurrent {
        status.add_warning("High process concurrency");
    }
    
    status
}
```

## Implementation Tasks (TASKS.md)

### High Priority
1. **Create reev-types crate** ✅ COMPLETED
2. **Implement RunnerProcessManager** 
3. **Add CLI execution wrapper**
4. **Create execution state management**
5. **Implement JSON-RPC protocol**
6. **Add timeout and error handling**

### Medium Priority
7. **Migrate read-only endpoints**
8. **Add CLI testing framework**
9. **Update CURL.md with CLI tests**
10. **Implement recovery mechanisms**

### Low Priority
11. **Migrate write endpoints**
12. **Remove direct dependencies**
13. **Add performance monitoring**
14. **Update deployment configuration**

## Success Criteria

### Functional Requirements
- ✅ All existing API endpoints work with CLI runner
- ✅ No regression in benchmark execution results
- ✅ Graceful error handling and recovery
- ✅ Performance within 20% of direct library calls

### Architectural Requirements
- ✅ Eliminate reev-runner, reev-flow, reev-tools dependencies
- ✅ Clean separation via reev-types
- ✅ State-based communication through reev-db
- ✅ Modular, testable components

### Operational Requirements
- ✅ Proper logging and monitoring
- ✅ Configurable timeouts and limits
- ✅ Development and production deployment strategies
- ✅ Comprehensive test coverage

## Timeline

### Week 1: Foundation
- Create reev-types crate ✅
- Implement basic RunnerProcessManager
- Add CLI execution wrapper

### Week 2: Integration
- Implement JSON-RPC protocol
- Add execution state management
- Create comprehensive tests

### Week 3: Migration
- Migrate read-only endpoints
- Add CURL.md tests
- Implement error handling

### Week 4: Completion
- Migrate write endpoints
- Remove dependencies
- Performance testing and optimization