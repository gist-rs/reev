# Implementation Tasks: Dynamic Benchmark System

## 🎯 **Core Objectives**

1. **Align Code with MD Design** - Scan existing code and ensure consistency with DYNAMIC_BENCHMARK_DESIGN.md
2. **Separate Benchmark & User Flows** - Same core logic, different entry points  
3. **Eliminate Mock Data** - Real execution only, let failures fail with proper scoring
4. **Fix Tool Name Chaos** - Replace string-based tool names with enum via strum crate

---

## 📋 **Phase 1: Code Analysis & Alignment**

### **Task 1.1: Scan Existing Code vs MD Document**
```bash
# Objective: Find discrepancies between current implementation and MD design
Scope: All reev crates
Focus areas:
- Mode-based separation (benchmark vs dynamic)
- Ping-pong orchestrator implementation  
- OTEL capture at orchestrator level
- YML generation for dynamic flows
```

**Key Areas to Scan:**
- `reev-orchestrator/src/gateway.rs` - Entry point separation
- `reev-orchestrator/src/execution/ping_pong_executor.rs` - Task management
- `reev-orchestrator/src/generators/yml_generator.rs` - YML generation logic
- `reev-agent/src/run.rs` - Agent execution interface
- `reev-api/src/handlers/dynamic_flows/` - API handlers

**Expected Findings:**
- Mixed mock/production code
- Missing mode detection logic
- String-based tool names causing issues
- Benchmark code leaking into production paths

### **Task 1.2: Document Current vs Target State**
Create analysis matrix:

| Component | Current State | Target State (MD) | Gap |
|-----------|--------------|-------------------|-----|
| Mode Separation | Mixed logic | Top-level only | ❌ |
| Tool Names | String-based | Enum + strum | ❌ |
| Mock Data | Throughout | Production only | ❌ |
| YML Generation | Over-engineered | Simple generation | ❌ |
| Entry Points | Confused | Clean routing | ❌ |

---

## 📋 **Phase 2: Benchmark-First Implementation**

### **Task 2.1: Create Clean Benchmark Execution Path**
```rust
// File: crates/reev-orchestrator/src/benchmark_mode.rs
#[cfg(feature = "benchmark")]
pub mod benchmark_mode {
    use reev_runner::execute_benchmark;
    use std::path::PathBuf;
    
    pub fn execute_static_benchmark(
        benchmark_id: &str,
    ) -> Result<ExecutionResult> {
        // Get static 300-series YML file
        let yml_path = get_static_benchmark_path(benchmark_id)?;
        
        // Use existing runner (no changes needed)
        execute_benchmark(yml_path).await
    }
    
    fn get_static_benchmark_path(id: &str) -> Result<PathBuf> {
        let path = format!("benchmarks/{}.yml", id);
        Ok(PathBuf::from(path))
    }
}
```

### **Task 2.2: Create Production Dynamic Path**
```rust
// File: crates/reev-orchestrator/src/dynamic_mode.rs
#[cfg(feature = "production")]
pub mod dynamic_mode {
    use reev_runner::execute_benchmark;
    use crate::gateway::generate_dynamic_yml;
    
    pub fn execute_user_request(
        prompt: &str,
        context: &WalletContext,
    ) -> Result<ExecutionResult> {
        // Generate temporary YML file
        let yml_path = generate_dynamic_yml(prompt, context).await?;
        
        // Use same runner (identical core logic)
        execute_benchmark(yml_path).await
    }
}
```

### **Task 2.3: Top-Level Mode Router**
```rust
// File: crates/reev-orchestrator/src/mod.rs
use crate::benchmark_mode::execute_static_benchmark;
use crate::dynamic_mode::execute_user_request;

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Benchmark(String),  // Benchmark ID
    Dynamic,           // User request
}

pub fn route_execution(
    mode: ExecutionMode,
    request: UserRequest,
) -> Result<ExecutionResult> {
    match mode {
        ExecutionMode::Benchmark(id) => {
            execute_static_benchmark(&id)
        }
        ExecutionMode::Dynamic => {
            execute_user_request(&request.prompt, &request.context)
        }
    }
}
```

---

## 📋 **Phase 3: Tool Name System Overhaul**

### **Task 3.1: Define Tool Enums with Strum**
```rust
// File: crates/reev-types/src/tools.rs
use strum::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Display, EnumString, IntoStaticStr)]
pub enum ToolName {
    #[strum(serialize = "account_balance")]
    AccountBalance,
    
    #[strum(serialize = "jupiter_swap")]
    JupiterSwap,
    
    #[strum(serialize = "jupiter_lend")]
    JupiterLend,
    
    #[strum(serialize = "jupiter_withdraw")]
    JupiterWithdraw,
    
    #[strum(serialize = "jupiter_positions")]
    JupiterPositions,
}

impl ToolName {
    pub fn requires_wallet(&self) -> bool {
        matches!(self, ToolName::AccountBalance | ToolName::JupiterPositions)
    }
    
    pub fn estimated_time_ms(&self) -> u64 {
        match self {
            ToolName::AccountBalance => 5000,
            ToolName::JupiterSwap => 30000,
            ToolName::JupiterLend => 45000,
            ToolName::JupiterWithdraw => 25000,
            ToolName::JupiterPositions => 10000,
        }
    }
}
```

### **Task 3.2: Update All String-Based Tool References**
**Files to Update:**
- `reev-orchestrator/src/gateway.rs` - Step creation
- `reev-orchestrator/src/execution/ping_pong_executor.rs` - Tool execution
- `reev-agent/src/run.rs` - Agent tool calls
- `reev-api/src/handlers/flow_diagram/` - Visualization

**Before:**
```rust
let tool_name = "jupiter_swap".to_string();  // Error-prone
```

**After:**
```rust
let tool_name = ToolName::JupiterSwap;  // Type-safe
let serialized = tool_name.to_string();  // When needed
```

### **Task 3.3: Add Strum Dependency**
```toml
# File: crates/reev-types/Cargo.toml
[dependencies]
strum = { version = "0.25", features = ["derive"] }
```

---

## 📋 **Phase 4: Eliminate Mock Data**

### **Task 4.1: Remove Mock Responses**
```rust
// Remove from: reev-orchestrator/src/gateway.rs
// BAD - Mock data in production:
if prompt.contains("mock") {
    return Ok(MOCK_RESPONSE);
}

// GOOD - Real execution only:
let result = execute_with_agent(prompt, context).await?;
if result.is_err() {
    // Let it fail - record actual failure, not mock success
    scoring::record_failure(&result);
    return result;
}
```

### **Task 4.2: Real-Only Scoring System**
```rust
// File: crates/reev-scoring/src/real_scoring.rs
pub fn score_execution(result: &ExecutionResult) -> Score {
    match result.status {
        Status::Completed => calculate_success_score(result),
        Status::Failed => calculate_failure_score(result),  // Real failure scoring
        Status::Partial => calculate_partial_score(result),
    }
}

// NO mock success inflation
fn calculate_failure_score(result: &ExecutionResult) -> Score {
    Score {
        total: 0.0,
        details: vec![
            "Step 1 failed: Tool execution error",
            "Step 2 failed: Dependency failure",
            // Real failure reasons only
        ],
    }
}
```

### **Task 4.3: Update Error Handling**
```rust
// Replace hidden mock fallbacks with proper error propagation
pub async fn execute_flow_plan(plan: &FlowPlan) -> Result<ExecutionResult> {
    for step in plan.steps {
        let result = execute_step(&step).await?;
        
        // Real error handling - no mock recovery
        if !result.success {
            return Err(anyhow::anyhow!(
                "Step '{}' failed: {}",
                step.name,
                result.error.unwrap_or("Unknown error")
            ));
        }
    }
    
    Ok(ExecutionResult::success(result))
}
```

---

## 📋 **Phase 5: Simple Dynamic YML Generation**

### **Task 5.1: Basic YML Generator**
```rust
// File: crates/reev-orchestrator/src/dynamic_generator.rs
use reev_types::tools::ToolName;

pub struct DynamicYmlGenerator;

impl DynamicYmlGenerator {
    pub async fn generate_simple_yml(
        prompt: &str,
        context: &WalletContext,
    ) -> Result<PathBuf> {
        let intent = analyze_simple_intent(prompt)?;
        
        let yml_content = match intent.primary_tool {
            ToolName::JupiterSwap => generate_swap_yml(prompt, context),
            ToolName::JupiterLend => generate_lend_yml(prompt, context),
            ToolName::JupiterWithdraw => generate_withdraw_yml(prompt, context),
            _ => generate_default_flow_yml(prompt, context),
        };
        
        write_temp_yml_file(&yml_content).await
    }
}

fn generate_swap_yml(prompt: &str, context: &WalletContext) -> String {
    format!(
        r#"
id: dynamic-swap-{timestamp}
description: "Dynamic user swap request"
tags: ["dynamic", "swap", "jupiter"]

prompt: "{prompt}"

initial_state: [{generate_wallet_state(context)}]

ground_truth:
  final_state_assertions:
    - type: "TokenAccountBalance"
      pubkey: "USER_USDC_ATA"
      expected_gte: 1
      weight: 1
"#,
        timestamp = chrono::Utc::now().timestamp(),
        prompt = prompt,
    )
}
```

### **Task 5.2: Intent Analysis (Simple)**
```rust
// Simple intent detection - no over-engineering
pub fn analyze_simple_intent(prompt: &str) -> Result<UserIntent> {
    let prompt_lower = prompt.to_lowercase();
    
    let (primary_tool, parameters) = if prompt_lower.contains("swap") {
        (ToolName::JupiterSwap, extract_amount(prompt))
    } else if prompt_lower.contains("lend") {
        (ToolName::JupiterLend, extract_amount(prompt))
    } else if prompt_lower.contains("withdraw") {
        (ToolName::JupiterWithdraw, extract_amount(prompt))
    } else {
        (ToolName::JupiterSwap, extract_amount(prompt)) // Default
    };
    
    Ok(UserIntent {
        primary_tool,
        parameters,
        complexity: "simple",
    })
}
```

---

## 📋 **Phase 6: Integration & Testing**

### **Task 6.1: Update API Endpoints**
```rust
// File: crates/reev-api/src/handlers/dynamic_flows/mod.rs
pub async fn execute_direct(
    Json(request): ExecuteDirectRequest,
) -> Result<Json<ExecutionResponse>> {
    let context = resolve_wallet_context(&request.wallet).await?;
    
    // Use dynamic mode execution (same core logic)
    let result = dynamic_mode::execute_user_request(&request.prompt, &context).await?;
    
    Ok(Json(ExecutionResponse {
        execution_id: generate_execution_id(),
        status: result.status,
        flow_id: result.flow_id,
        tool_calls: result.tool_calls,
    }))
}
```

### **Task 6.2: Update CLI Commands**
```bash
# Benchmark execution (300-series)
reev-runner --benchmark 300-jup-swap-then-lend-deposit-dyn

# Dynamic execution (user requests)  
reev-runner --dynamic --prompt "swap 1 SOL to USDC"

# Both use same core runner
```

### **Task 6.3: Integration Tests**
```rust
// File: tests/integration/separated_modes_test.rs
#[tokio::test]
async fn test_benchmark_mode_uses_static_file() {
    let result = benchmark_mode::execute_static_benchmark("300-jup-swap-then-lend-deposit-dyn").await;
    assert!(result.is_ok());
    // Verify static file was used
}

#[tokio::test]
async fn test_dynamic_mode_generates_yml() {
    let prompt = "swap 1 SOL to USDC";
    let context = create_mock_context();
    
    let result = dynamic_mode::execute_user_request(prompt, &context).await;
    assert!(result.is_ok());
    // Verify temp YML was generated and cleaned up
}
```

---

## 🎯 **Implementation Order**

### **Priority 1: Foundation (Days 1-2)**
1. **Task 3.1-3.3**: Tool name enum + strum (fixes compilation issues)
2. **Task 1.1**: Code scan vs MD (understand current gaps)
3. **Task 2.3**: Mode router (establishes clean separation)

### **Priority 2: Core Logic (Days 3-5)**
1. **Task 2.1**: Benchmark mode execution
2. **Task 5.1-5.2**: Simple dynamic YML generation  
3. **Task 4.1-4.3**: Remove all mock data

### **Priority 3: Integration (Days 6-7)**
1. **Task 6.1**: Update API endpoints
2. **Task 6.2**: Update CLI commands
3. **Task 6.3**: Integration tests

---

## ✅ **Success Criteria**

### **Code Quality:**
- [ ] Zero mock data in production paths
- [ ] All tool names use enum + strum (no strings)
- [ ] Clean mode separation (feature flags or config)
- [ ] Single core execution logic shared by both modes

### **Functionality:**
- [ ] Benchmark mode: Static 300-series YML files work
- [ ] Dynamic mode: Natural language → temporary YML → execution
- [ ] Same runner/agent/OTEL pipeline for both modes
- [ ] Real scoring: failures fail properly, no inflated scores

### **Architecture:**
- [ ] Code matches DYNAMIC_BENCHMARK_DESIGN.md exactly
- [ ] Decoupled components with clear interfaces
- [ ] Type-safe tool handling throughout
- [ ] Proper error propagation, no hidden mocks

---

## 🚨 **Critical Notes**

1. **Don't Over-Engineer**: Dynamic YML generation should be SIMPLE (like 100/200 series)
2. **Reuse Everything**: Runner, agent, OTEL, DB, scoring - ZERO changes needed
3. **Real Data Only**: No mock responses, let real failures happen and score them
4. **Type Safety**: Strum enums for tool names eliminate string typos
5. **Clean Separation**: Mode detection only at top level, same core logic beneath

**The goal is battle-tested reliability, not fancy complexity.**