# 🪸 Reev Framework API Documentation

## 🎯 Overview

The Reev framework provides a comprehensive set of APIs for evaluating Solana LLM agents through benchmark execution, scoring, and analysis. This document covers the complete API surface including Rust libraries, REST endpoints, and configuration interfaces.

## 📦 Core Components

### 1. reev-lib - Core Library

The foundational library providing core abstractions and utilities.

#### Environment Interface

```rust
use reev_lib::env::GymEnv;

// Environment trait for agent interaction
pub trait GymEnv {
    type Action;
    type Observation;

    async fn reset(&mut self, seed: Option<u64>, options: Option<Value>) -> Result<Self::Observation>;
    fn step(&mut self, actions: Vec<Self::Action>, ground_truth: &GroundTruth) -> Result<Step<Self::Observation>>;
    fn render(&self);
    fn close(&mut self) -> Result<()>;
}

// Solana-specific environment implementation
pub struct SolanaEnv {
    // Internal state management
}

impl SolanaEnv {
    pub async fn new() -> Result<Self>;
    pub async fn reset_for_benchmark(&mut self, benchmark: &TestCase) -> Result<AgentObservation>;
}
```

#### Agent Interface

```rust
use reev_lib::agent::{Agent, AgentAction, AgentObservation};

#[async_trait]
pub trait Agent {
    async fn get_action(
        &mut self,
        id: &str,
        prompt: &str,
        observation: &AgentObservation,
        fee_payer: Option<&String>,
        skip_instruction_validation: Option<bool>,
    ) -> Result<Vec<AgentAction>>;
}

// Agent action wrapper
pub struct AgentAction(pub Instruction);

// Environment observation
pub struct AgentObservation {
    pub last_transaction_status: String,
    pub last_transaction_error: Option<String>,
    pub last_transaction_logs: Vec<String>,
    pub account_states: HashMap<String, Value>,
    pub key_map: HashMap<String, String>,
}
```

#### Scoring System

```rust
use reev_lib::score::{calculate_final_score, calculate_detailed_score};

// Main scoring function
pub fn calculate_final_score(
    test_case: &TestCase,
    actions: &[AgentAction],
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> f64;

// Detailed scoring with breakdown
pub fn calculate_detailed_score(
    test_case: &TestCase,
    actions: &[AgentAction],
    initial_observation: &AgentObservation,
    final_observation: &AgentObservation,
) -> ScoringBreakdown;

// Scoring breakdown structure
pub struct ScoringBreakdown {
    pub final_score: f64,
    pub instruction_score: f64,
    pub onchain_score: f64,
    pub issues: Vec<String>,
    pub mismatches: Vec<String>,
    pub weight_achieved: f64,
    pub total_weight: f64,
}
```

### 2. reev-agent - Agent Service

HTTP-based agent service providing LLM integration.

#### Core Endpoints

```rust
// Health check
GET /health
Response: {"status": "ok", "timestamp": "..."

// Transaction generation
POST /gen/tx
Content-Type: application/json

Request:
{
    "id": "benchmark-id",
    "prompt": "natural language task",
    "context_prompt": "context information",
    "model_name": "agent-type",
    "flow_step": "step-id"  // Optional for flows
}

Response:
{
    "result": {
        "text": "[{program_id: "...", accounts: [...], data: "..."}]"
    },
    "flows": {
        "tool_calls": [...],
        "total_tool_calls": 3,
        "tool_usage": {"get_balance": 1, "swap": 2}
    }
}
```

#### Flow-Specific Endpoints

```rust
// Initialize flow session
POST /flow/init
Request: {
    "benchmark_id": "200-jup-swap-then-lend-deposit",
    "agent_type": "deterministic"
}

// Execute flow step
POST /flow/step
Request: {
    "step_id": "step_1",
    "prompt": "Swap 0.1 SOL to USDC",
    "context": {...}
}

// Get flow status
GET /flow/status/{session_id}
Response: {
    "current_step": "step_2",
    "completed_steps": ["step_1"],
    "state": {...}
}
```

### 3. reev-api - REST API Server

Main API server for web interface and external integration.

#### Benchmark Management

```rust
// Get all benchmarks
GET /api/benchmarks
Response: [{
    "id": "001-sol-transfer",
    "description": "Basic SOL transfer",
    "tags": ["system-program", "transfer"],
    "category": "transaction"
}]

// Get specific benchmark
GET /api/benchmarks/{id}
Response: {
    "id": "001-sol-transfer",
    "prompt": "Send 0.1 SOL to recipient",
    "initial_state": [...],
    "ground_truth": {...}
}

// Upsert benchmark (dynamic management)
POST /api/benchmarks/upsert
Request: {
    "id": "custom-benchmark",
    "content": "yaml-content",
    "overwrite": true
}
```

#### Agent Management

```rust
// Get available agents
GET /api/agents
Response: [{
    "name": "deterministic",
    "type": "deterministic",
    "description": "Ground truth agent",
    "capabilities": ["transactions", "flows"]
}, {
    "name": "local",
    "type": "llm",
    "description": "Local LLM agent",
    "model": "qwen3-coder-30b",
    "endpoint": "http://localhost:1234"
}]

// Execute benchmark with agent
POST /api/execute
Request: {
    "benchmark_id": "001-sol-transfer",
    "agent_name": "deterministic",
    "options": {
        "timeout": 30,
        "debug": true
    }
}

Response: {
    "execution_id": "exec-123",
    "status": "running",
    "started_at": "2025-10-16T10:00:00Z"
}
```

#### Performance Analytics

```rust
// Get performance data
GET /api/performance?agent=deterministic&benchmark=001-sol-transfer
Response: {
    "scores": [1.0, 1.0, 0.75, 1.0],
    "execution_times": [150, 180, 200, 160],
    "success_rate": 0.95,
    "average_score": 0.9375
}

// Get detailed execution trace
GET /api/execution/{execution_id}/trace
Response: {
    "steps": [{
        "step_id": "step_1",
        "prompt": "Swap 0.1 SOL to USDC",
        "actions": [...],
        "result": {...},
        "score": 1.0,
        "execution_time_ms": 250
    }],
    "final_score": 0.95,
    "total_time_ms": 500
}
```

### 4. reev-runner - CLI Interface

Command-line interface for benchmark execution and testing.

#### Core Commands

```bash
# Run single benchmark
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic

# Run with specific model
cargo run -p reev-runner -- benchmarks/100-jup-swap-sol-usdc.yml --agent glm-4.6

# Run all benchmarks
cargo run -p reev-runner -- --all --agent deterministic

# Run flow benchmarks
cargo run -p reev-runner -- benchmarks/200-jup-swap-then-lend-deposit.yml --agent local

# Debug mode with verbose logging
RUST_LOG=debug cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent deterministic
```

#### Configuration Options

```bash
# Environment variables
RUST_LOG=info                    # Log level (debug, info, warn, error)
REEV_FLOW_LOG_PATH=logs/flows    # Flow log output directory
GOOGLE_API_KEY=your-key          # Gemini API key
GLM_API_KEY=your-key            # GLM API key
GLM_API_URL="https://api.z.ai/api/coding/paas/v4"

# Command-line options
--timeout SECONDS                # Execution timeout (default: 30)
--output-format json            # Output format (text, json)
--save-trace                   # Save execution trace
--benchmark-dir PATH           # Custom benchmark directory
--agent-config PATH            # Agent configuration file
```

### 5. reev-tui - Terminal Interface

Interactive terminal user interface for real-time monitoring.

#### Key Features

```rust
// TUI Components
pub struct BenchmarkGrid {
    // Grid of benchmark cards with status
}

pub struct AgentPerformanceCard {
    // Performance metrics for specific agent
}

pub struct ExecutionTraceView {
    // Real-time execution trace display
}

pub struct TransactionLogView {
    // Transaction history and details
}

// TUI Events
pub enum TuiEvent {
    BenchmarkSelected(String),
    AgentChanged(String),
    ExecutionStarted(String),
    ExecutionCompleted(String),
    RefreshRequested,
}
```

#### Keyboard Shortcuts

```
q/Quit: Exit application
r: Refresh data
1-4: Switch between tabs
↑/↓: Navigate lists
Enter: Select item/Start execution
Space: Pause/Resume execution
d: Toggle debug mode
s: Save current state
```

## 🔧 Configuration

### Benchmark Configuration

```yaml
# Standard benchmark structure
id: unique-benchmark-id
description: Clear description of purpose
tags: ["protocol", "operation", "category"]

initial_state:
  - pubkey: "PLACEHOLDER_NAME"
    owner: "PROGRAM_PUBKEY"
    lamports: 1000000000
    data: {...}

prompt: "Natural language task description"

ground_truth:
  skip_instruction_validation: false  # true for API protocols
  final_state_assertions:
    - type: SolBalance
      pubkey: "PLACEHOLDER"
      expected: 100000000
      weight: 1.0
  expected_instructions:
    - program_id: "PROGRAM_PUBKEY"
      program_id_weight: 0.5
      accounts: [...]
      data: "..."
      weight: 1.0
```

### Flow Configuration

```yaml
# Multi-step flow structure
flow:
  - step: 1
    description: "First action description"
    prompt: "Specific instruction for this step"
    critical: true
    timeout: 30
    depends_on: []

  - step: 2
    description: "Second action"
    prompt: "Follow-up instruction"
    critical: true
    timeout: 30
    depends_on: ["step_1_result"]

ground_truth:
  min_score: 0.6
  success_criteria:
    - type: "steps_completed"
      required: 2
      weight: 0.5
    - type: "no_critical_errors"
      required: true
      weight: 0.5
```

### Agent Configuration

```json
{
  "agents": {
    "deterministic": {
      "type": "deterministic",
      "description": "Ground truth agent for testing"
    },
    "local": {
      "type": "llm",
      "endpoint": "http://localhost:1234",
      "model": "qwen3-coder-30b-a3b-instruct-mlx",
      "timeout": 30
    },
    "gemini-2.5-flash-lite": {
      "type": "llm",
      "provider": "google",
      "model": "gemini-2.5-flash-lite",
      "api_key_env": "GOOGLE_API_KEY"
    },
    "glm-4.6": {
      "type": "llm",
      "provider": "zhipuai",
      "model": "glm-4.6",
      "api_key_env": "GLM_API_KEY",
      "api_url_env": "GLM_API_URL"
    }
  }
}
```

## 🧪 Testing APIs

### Unit Testing

```rust
use reev_lib::testing::*;

// Test individual components
#[tokio::test]
async fn test_instruction_scoring() {
    let score = calculate_instruction_score(&test_case, &actions, &key_map);
    assert!(score >= 0.0 && score <= 1.0);
}

// Test environment setup
#[tokio::test]
async fn test_environment_reset() {
    let mut env = SolanaEnv::new().await?;
    let observation = env.reset(Some(42), None).await?;
    assert!(!observation.account_states.is_empty());
}
```

### Integration Testing

```rust
// Full benchmark execution test
#[tokio::test]
async fn test_benchmark_execution() {
    let (mut env, test_case, initial_obs) = setup_env_for_benchmark(&path).await?;
    let actions = generate_test_actions(&test_case);
    let result = env.step(actions, &test_case.ground_truth)?;

    let score = calculate_final_score(&test_case, &actions, &initial_obs, &result.observation);
    assert!(score > 0.8); // Expect high score for test case
}
```

### Flow Testing

```rust
// Multi-step flow execution test
#[tokio::test]
async fn test_flow_execution() {
    let mut flow_agent = FlowAgent::new("deterministic").await?;
    let benchmark = load_flow_benchmark("200-jup-swap-then-lend-deposit.yml")?;

    flow_agent.load_benchmark(&benchmark).await?;
    let results = flow_agent.execute_flow(&benchmark).await?;

    assert_eq!(results.len(), 2); // Two steps
    assert!(results.iter().all(|r| r.status == ExecutionStatus::Success));
}
```

## 📊 Monitoring & Observability

### Metrics Collection

```rust
use reev_lib::metrics::*;

// Performance metrics
pub struct PerformanceMetrics {
    pub execution_time_ms: u64,
    pub instruction_count: u32,
    pub success_rate: f64,
    pub average_score: f64,
}

// Real-time monitoring
pub struct MetricsCollector {
    pub agent_performance: HashMap<String, PerformanceMetrics>,
    pub benchmark_stats: HashMap<String, BenchmarkStats>,
    pub system_health: SystemHealth,
}
```

### Flow Tracing

```rust
use reev_flow::{FlowLogger, FlowTracer};

// Initialize flow tracing
let flow_logger = FlowLogger::new(
    benchmark_id.to_string(),
    agent_type.to_string(),
    output_path,
);

// Log flow events
flow_logger.log_llm_request(step_id, prompt, tools);
flow_logger.log_tool_call(tool_call_info);
flow_logger.log_llm_response(step_id, response, tools, result);

// Generate execution trace
let flow_log = flow_logger.finalize().await?;
let trace = render_flow_as_ascii_tree(&flow_log)?;
```

### OpenTelemetry Integration

```rust
use opentelemetry::trace::Tracer;

// Instrumented execution
#[tracing::instrument(
    name = "benchmark_execution",
    fields(
        benchmark_id = %test_case.id,
        agent_name = %agent_name,
        execution_time = tracing::field::Empty
    )
)]
async fn execute_benchmark_instrumented(
    test_case: &TestCase,
    agent: &mut dyn Agent,
) -> Result<ExecutionResult> {
    let start = std::time::Instant::now();

    // Execute benchmark
    let result = execute_benchmark(test_case, agent).await?;

    let execution_time = start.elapsed();
    tracing::info!(execution_time_ms = execution_time.as_millis());

    Ok(result)
}
```

## 🚀 Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReevError {
    #[error("Environment setup failed: {0}")]
    EnvironmentSetup(String),

    #[error("Agent execution failed: {0}")]
    AgentExecution(String),

    #[error("Benchmark validation failed: {0}")]
    BenchmarkValidation(String),

    #[error("Scoring calculation failed: {0}")]
    ScoringCalculation(String),

    #[error("Flow execution failed: {0}")]
    FlowExecution(String),

    #[error("Database operation failed: {0}")]
    DatabaseOperation(String),
}
```

### Error Recovery

```rust
// Retry logic for transient failures
pub async fn execute_with_retry<T, F, Fut>(
    operation: F,
    max_retries: u32,
    delay: Duration,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(delay).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 📚 Examples

### Basic Agent Usage

```rust
use reev_lib::{SolanaEnv, LlmAgent};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup environment
    let mut env = SolanaEnv::new().await?;
    let benchmark = load_benchmark("001-sol-transfer.yml")?;

    // Reset environment
    let initial_obs = env.reset_for_benchmark(&benchmark).await?;

    // Create agent
    let mut agent = LlmAgent::new("deterministic").await?;

    // Get action from agent
    let actions = agent.get_action(
        &benchmark.id,
        &benchmark.prompt,
        &initial_obs,
        None,
        None,
    ).await?;

    // Execute action
    let result = env.step(actions, &benchmark.ground_truth)?;

    // Calculate score
    let score = calculate_final_score(
        &benchmark,
        &actions,
        &initial_obs,
        &result.observation,
    );

    println!("Final score: {}", score);
    env.close()?;

    Ok(())
}
```

### Flow Execution

```rust
use reev_lib::flow::FlowAgent;

#[tokio::main]
async fn main() -> Result<()> {
    // Create flow agent
    let mut flow_agent = FlowAgent::new("local").await?;

    // Load flow benchmark
    let benchmark = load_flow_benchmark("200-jup-swap-then-lend-deposit.yml")?;

    // Execute flow
    flow_agent.load_benchmark(&benchmark).await?;
    let results = flow_agent.execute_flow(&benchmark).await?;

    // Process results
    for (i, result) in results.iter().enumerate() {
        println!("Step {}: {:?}", i + 1, result.status);
    }

    Ok(())
}
```

This comprehensive API documentation provides complete coverage of the Reev framework's interfaces, enabling developers to effectively integrate and extend the system for Solana LLM agent evaluation.
