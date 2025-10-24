# reev-flow: Multi-Step Workflow & Flow Management

The `reev-flow` crate provides comprehensive multi-step workflow orchestration, flow state management, and session logging for the Reev evaluation framework. It enables complex, multi-transaction workflows that span multiple DeFi protocols.

## 🎯 Core Features

### Workflow Orchestration
- **Multi-Step Flows**: Complex workflows across multiple protocols
- **Step Dependencies**: Conditional execution based on previous results
- **State Management**: Persistent state across workflow steps
- **Error Recovery**: Graceful handling of step failures
- **Performance Tracking**: OpenTelemetry integration for flow metrics

### Flow State Management
- **Session Persistence**: Complete workflow state tracking
- **Context Consolidation**: Account state updates between steps
- **Result Aggregation**: Combine results from multiple steps
- **Flow Visualization**: ASCII tree rendering for debugging

### Data Management
- **Database Integration**: Turso/SQLite backend for flow logs
- **Session Management**: UUID-based session tracking
- **Result Storage**: Persistent storage of workflow outcomes
- **Export Capabilities**: Web-ready flow result formats

## 🏗️ Architecture

### Core Components

```rust
/// Multi-step workflow definition
pub struct FlowBenchmark {
    pub id: String,
    pub name: String,
    pub description: String,
    pub flow: Vec<FlowStep>,
    pub tags: Vec<String>,
}

/// Individual workflow step
pub struct FlowStep {
    pub step_id: u32,
    pub description: String,
    pub prompt: String,
    pub critical: bool,
    pub timeout: Option<u64>,
    pub depends_on: Option<Vec<u32>>,
}

/// Flow execution result
pub struct ExecutionResult {
    pub step_id: u32,
    pub status: ExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
```

### Flow Engine
- **Step Scheduler**: Order steps based on dependencies
- **State Tracker**: Maintain workflow state across steps
- **Result Collector**: Aggregate results from completed steps
- **Error Handler**: Manage failures and recovery strategies

## 🛠️ Available Features

### Step Dependencies
```rust
// Define step dependencies
FlowStep {
    step_id: 2,
    depends_on: Some(vec![1]), // Step 2 depends on step 1
    critical: true,
    // ... other fields
}
```

### Flow Variants
- **Linear Flows**: Sequential step execution
- **Parallel Flows**: Independent step execution
- **Conditional Flows**: Steps based on previous results
- **Recovery Flows**: Alternative paths for failures

### Logging & Tracing
- **Session Logs**: Complete workflow execution history
- **OpenTelemetry**: Distributed tracing across steps
- **Performance Metrics**: Step execution timing and success rates
- **Error Context**: Detailed failure information

## 🧪 Testing Strategy

### Test Files (6 tests)
- `flow_execution_test.rs` - End-to-end flow execution
- `renderer_test.rs` - Flow visualization and ASCII rendering
- `database_test.rs` - Flow persistence and database operations
- `utils_test.rs` - Flow utility functions
- `website_exporter_test.rs` - Web export functionality
- `otel_test.rs` - OpenTelemetry integration

### Running Tests
```bash
# Run all flow tests
cargo test -p reev-flow

# Run specific test with output
cargo test -p reev-flow --test flow_execution_test -- --nocapture

# Test flow rendering
cargo test -p reev-flow --test renderer_test -- --nocapture
```

## 📁 Project Structure

```
src/
├── lib.rs                    # Main flow orchestration
├── types.rs                 # Flow data structures
├── database.rs               # Database integration
├── logger.rs                 # Session logging
├── renderer.rs              # ASCII tree rendering
├── utils.rs                 # Flow utilities
├── otel.rs                  # OpenTelemetry integration
└── website_exporter.rs        # Web result export
```

## 🔧 Dependencies

```toml
[dependencies]
reev-lib = { path = "../reev-lib" }
reev-db = { path = "../reev-db" }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
tracing-opentelemetry = { workspace = true }
tokio = { workspace = true, features = ["full"] }
```

## 🚀 Usage Examples

### Basic Flow Execution
```rust
use reev_flow::{FlowBenchmark, FlowStep, execute_flow};

let flow_benchmark = FlowBenchmark {
    id: "200-jup-swap-then-lend-deposit".to_string(),
    name: "Jupiter Swap then Lend".to_string(),
    description: "Swap SOL to USDC, then deposit USDC".to_string(),
    flow: vec![
        FlowStep {
            step_id: 1,
            description: "Swap SOL for USDC".to_string(),
            prompt: "Swap 1 SOL for USDC using Jupiter".to_string(),
            critical: true,
            timeout: Some(30000),
            depends_on: None,
        },
        FlowStep {
            step_id: 2,
            description: "Deposit USDC into Jupiter lending".to_string(),
            prompt: "Deposit the USDC from previous step into Jupiter lending".to_string(),
            critical: true,
            timeout: Some(30000),
            depends_on: Some(vec![1]), // Depends on step 1
        },
    ],
    tags: vec!["jupiter".to_string(), "lending".to_string()],
};

let results = execute_flow(&flow_benchmark).await?;
for result in results {
    println!("Step {}: {:?}", result.status);
}
```

### Session Logging
```rust
use reev_flow::create_session_logger;

let session_id = uuid::Uuid::new_v4();
let logger = create_session_logger(
    session_id,
    "benchmark-id",
    "agent-type",
    Some(log_directory)
)?;

// Log step execution
logger.log_step(
    1,
    "step-prompt",
    "step-result",
    ExecutionStatus::Success,
    2500
)?;
```

### Flow Visualization
```rust
use reev_flow::renderer::render_flow_as_ascii_tree;

let ascii_tree = render_flow_as_ascii_tree(&flow_benchmark, &results)?;
println!("{}", ascii_tree);
```

## 🎮 Integration with Reev Architecture

Flows integrate into the Reev evaluation system:

```
Runner (loads flow benchmark)
    ↓
Flow Engine (orchestrates steps)
    ↓
Agent (executes each step)
    ↓
Tools (perform operations)
    ↓
Surfpool (processes transactions)
    ↓
Flow Logger (tracks progress)
    ↓
Database (stores results)
```

## 🔍 Advanced Features

### Error Handling & Recovery
- **Step-Level Errors**: Per-step error handling and retry logic
- **Flow-Level Recovery**: Alternative execution paths
- **Timeout Management**: Configurable timeouts per step
- **Critical Steps**: Marking essential steps for flow success

### Performance Optimization
- **Parallel Execution**: Independent steps can run in parallel
- **State Caching**: Efficient state management between steps
- **Resource Pooling**: Reuse connections and resources
- **Metrics Collection**: Real-time performance tracking

### Security & Reliability
- **Step Validation**: Input validation before execution
- **State Consistency**: Ensure coherent state transitions
- **Audit Trails**: Complete execution history
- **Recovery Points**: Checkpoints for long-running flows

## 📋 API Reference

### Flow Benchmark
```rust
pub struct FlowBenchmark {
    pub id: String,                    // Unique identifier
    pub name: String,                  // Human readable name
    pub description: String,             // Detailed description
    pub flow: Vec<FlowStep>,          // Workflow steps
    pub tags: Vec<String>,               // Categorization tags
}
```

### Flow Step
```rust
pub struct FlowStep {
    pub step_id: u32,                 // Sequential identifier
    pub description: String,            // Step description
    pub prompt: String,                // Agent prompt
    pub critical: bool,                // Success requirement
    pub timeout: Option<u64>,          // Execution timeout
    pub depends_on: Option<Vec<u32>>, // Dependency list
}
```

### Execution Result
```rust
pub enum ExecutionStatus {
    Success,
    Failed,
    Timeout,
    Skipped,
}

pub struct ExecutionResult {
    pub step_id: u32,
    pub status: ExecutionStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
```

### Main Functions
```rust
// Execute complete flow
pub async fn execute_flow(
    flow_benchmark: &FlowBenchmark,
    agent: &dyn Agent,
    environment: &mut dyn Environment
) -> Result<Vec<ExecutionResult>>

// Render flow as ASCII tree
pub fn render_flow_as_ascii_tree(
    flow: &FlowBenchmark,
    results: &[ExecutionResult]
) -> Result<String>

// Create session logger
pub fn create_session_logger(
    session_id: String,
    benchmark_id: String,
    agent_type: String,
    log_directory: Option<PathBuf>
) -> Result<FlowLogger>
```

## 🔄 Design Principles

1. **Modularity**: Each step is independent and testable
2. **Dependency Management**: Clear step dependency tracking
3. **State Consistency**: Coherent state transitions
4. **Error Recovery**: Graceful handling of failures
5. **Observability**: Complete tracing and logging
6. **Performance**: Optimized for complex workflows

## 🔧 Troubleshooting

### Common Flow Issues
- **Dependency Cycles**: Circular step dependencies
- **Timeout Issues**: Steps taking too long to execute
- **State Inconsistency**: Invalid state transitions
- **Resource Exhaustion**: Too many parallel steps

### Debugging Flow Execution
```rust
// Enable detailed logging
RUST_LOG=debug cargo test -p reev-flow

// Check flow structure
println!("Flow steps: {:?}", flow_benchmark.flow);

// Validate dependencies
validate_step_dependencies(&flow_benchmark.flow)?;

// Check execution results
for result in results {
    match result.status {
        ExecutionStatus::Failed => println!("Step {} failed: {:?}", result.step_id, result.error),
        ExecutionStatus::Success => println!("Step {} succeeded", result.step_id),
        _ => println!("Step {} status: {:?}", result.step_id, result.status),
    }
}
```

### Performance Optimization
- Minimize critical path length
- Use parallel execution for independent steps
- Optimize step timeout values
- Monitor flow execution metrics
- Cache frequently used state

## 📊 Flow Statistics

| Flow Type | Number of Steps | Use Cases |
|------------|------------------|-------------|
| Simple Linear | 2-5 | Basic multi-step operations |
| Complex Conditional | 5-10 | Advanced workflows |
| Recovery Paths | 3-8 | Error-tolerant flows |
| Parallel Execution | 2-6 | Independent operations |

## 🎯 Future Roadmap

### Planned Enhancements
- **Dynamic Flow Generation**: AI-powered workflow creation
- **Flow Templates**: Reusable workflow patterns
- **Advanced Recovery**: Intelligent error recovery strategies
- **Flow Optimization**: Performance tuning recommendations
- **Cross-Protocol Flows**: Multi-chain workflow support

### Flow Development Guide
1. Define clear step dependencies
2. Set appropriate timeouts for each step
3. Implement comprehensive error handling
4. Add validation for step inputs/outputs
5. Test flows with various scenarios
6. Monitor performance and optimize bottlenecks

## 🔄 Version History

- **v0.1.0**: Initial release with basic flow execution
- **v0.1.1**: Added dependency management and state tracking
- **v0.1.2**: Enhanced error handling and recovery
- **v0.1.3**: Added session logging and database integration
- **v0.1.4**: Improved performance and parallel execution
- **v0.1.5**: Added flow visualization and ASCII rendering