# Implementation Tasks

## Issue #39 - Production Feature Flag Implementation

### 🎯 **Objective**
Implement proper feature flag architecture to separate production LLM orchestration from development mock behaviors.

### 🏗️ **Required Implementation**

#### Step 1: Add Feature Flags to Cargo.toml
```toml
[features]
default = ["production"]
production = []                    # Clean LLM orchestration, no mocks
development = ["mock_behaviors"]     # Mock/deterministic for testing
```

#### Step 2: Update Agent Router with Feature Gates
**File**: `crates/reev-agent/src/lib.rs`
```rust
#[cfg(feature = "production")]
fn route_to_llm_agent() -> Result<String> {
    // Production: Pure LLM orchestration only
    let agent_config = AgentConfig::llm_only();
    execute_llm_agent(request, agent_config).await
}

#[cfg(feature = "mock_behaviors")]
fn route_to_llm_agent() -> Result<String> {
    // Development: Include deterministic fallbacks
    let agent_config = AgentConfig::with_deterministic_fallback();
    execute_llm_agent_with_fallback(request, agent_config).await
}
```

#### Step 3: Remove Mock Behaviors from Production
**Files**: All agent implementations
```rust
#[cfg(feature = "production")]
impl LlmAgent {
    async fn execute(&self, prompt: &str) -> Result<AgentResponse> {
        // Production: No mock responses, no deterministic fallbacks
        self.call_real_llm_api(prompt).await
    }
}

#[cfg(feature = "mock_behaviors")]
impl LlmAgent {
    async fn execute(&self, prompt: &str) -> Result<AgentResponse> {
        // Development: Mock responses allowed for testing
        if self.should_use_mock_response(prompt) {
            return self.generate_mock_response(prompt);
        }
        self.call_real_llm_api(prompt).await
    }
}
```

### ✅ **Success Criteria**
1. **Production Build**: `cargo build --release --features production` excludes all mocks
2. **Development Build**: `cargo build --features mock_behaviors` retains testing capabilities
3. **Runtime Verification**: Production mode has zero mock/deterministic code paths
4. **Testing Separation**: Mocks only compile in development builds

---

## Issue #38 - Complete Multi-Step Flow Visualization

### 🎯 **Objective**
Fix 4-step flow visualization to show complete strategy execution with parameter context and validation states.

### 📋 **Current State**
✅ **Working**: LLM routing, 4-step generation, tool execution, scoring  
❌ **Broken**: Only single tool calls visible in Mermaid diagrams

### 🏗️ **Required Implementation**

#### Step 1: Enhanced Tool Call Tracking
**File**: `reev-orchestrator/src/execution/ping_pong_executor.rs`
```rust
// Capture ALL execution steps, not just final result
impl PingPongExecutor {
    pub async fn execute_flow_plan(&mut self, flow_plan: &DynamicFlowPlan) -> Result<Vec<StepResult>> {
        let mut all_tool_calls = Vec::new();
        
        for (step_index, step) in flow_plan.steps.iter().enumerate() {
            // Execute step and capture tool call details
            let step_result = self.execute_single_step_with_otel(&step).await?;
            
            // Track ALL tool calls with parameters
            for tool_call in &step_result.tool_calls {
                all_tool_calls.push(ToolCallTrace {
                    step_index,
                    tool_name: tool_call.name.clone(),
                    parameters: tool_call.params.clone(),
                    execution_time: tool_call.duration_ms,
                    success: tool_call.success,
                    result_data: tool_call.result.clone(),
                });
            }
            
            // Store in session for diagram generation
            self.store_session_tool_calls(&flow_plan.flow_id, &all_tool_calls).await?;
        }
        
        Ok(all_tool_calls)
    }
}
```

#### Step 2: Multi-Step Session Parsing
**File**: `reev-api/src/handlers/flow_diagram/session_parser.rs`
```rust
// Parse complete tool sequence from execution logs
pub fn parse_multi_step_session(content: &str) -> Result<ParsedSession> {
    let mut tool_calls = Vec::new();
    let step_groups = group_tools_by_step(content);
    
    for (step_index, step_tools) in step_groups.iter().enumerate() {
        for tool in step_tools {
            let parsed_tool = ParsedToolCall {
                tool_name: tool["tool_name"].as_str().unwrap_or("unknown"),
                step_index: Some(step_index),
                parameters: extract_parameters(&tool),
                success: tool["success"].as_bool().unwrap_or(false),
                duration_ms: tool["duration_ms"].as_u64().unwrap_or(0),
            };
            tool_calls.push(parsed_tool);
        }
    }
    
    Ok(ParsedSession {
        tool_calls,
        session_id: extract_session_id(content),
        step_count: step_groups.len(),
    })
}
```

#### Step 3: Enhanced State Diagram Generation
**File**: `reev-api/src/handlers/flow_diagram/state_diagram_generator.rs`
```rust
// Generate comprehensive multi-step flow diagram
impl StateDiagramGenerator {
    pub fn generate_multi_step_diagram(session: &ParsedSession) -> FlowDiagram {
        let mut diagram_lines = Vec::new();
        diagram_lines.push("stateDiagram".to_string());
        
        // Step 1: Account Discovery
        diagram_lines.push("    [*] --> AccountDiscovery".to_string());
        diagram_lines.push("    AccountDiscovery --> ContextAnalysis : \"Extract requirements\"".to_string());
        
        // Add dynamic steps based on execution
        for (index, tool_call) in session.tool_calls.iter().enumerate() {
            let step_name = match tool_call.step_index {
                0 => "BalanceCheck",
                1 => "JupiterSwap", 
                2 => "JupiterLend",
                3 => "PositionValidation",
                _ => "UnknownStep",
            };
            
            let transition = format!("    {} --> {} : \"{}\"", 
                get_previous_step_name(index),
                step_name,
                create_transition_label(tool_call)
            );
            diagram_lines.push(transition);
            
            // Add parameter notes
            add_parameter_notes(&mut diagram_lines, step_name, tool_call);
        }
        
        // Add color coding
        add_style_definitions(&mut diagram_lines);
        
        FlowDiagram {
            diagram: diagram_lines.join("\n"),
            metadata: create_metadata(session),
            tool_calls: session.tool_calls.clone(),
        }
    }
}

fn create_transition_label(tool_call: &ParsedToolCall) -> String {
    match tool_call.tool_name.as_str() {
        "get_account_balance" => "Current: 4 SOL, 20 USDC".to_string(),
        "jupiter_swap" => format!("Swap {:.3} SOL → USDC", 
            lamports_to_sol(tool_call.parameters.get("amount").unwrap_or(&0))),
        "jupiter_lend_earn_deposit" => format!("Deposit {} USDC for yield",
            lamports_to_usdc(tool_call.parameters.get("amount").unwrap_or(&0))),
        _ => "Execute operation".to_string(),
    }
}
```

### 📊 **Expected Output**

#### Target Mermaid Diagram:
```mermaid
stateDiagram
    [*] --> AccountDiscovery
    AccountDiscovery --> ContextAnalysis : "Extract 50% SOL requirement"
    ContextAnalysis --> BalanceCheck : "Current: 4 SOL, 20 USDC"
    BalanceCheck --> JupiterSwap : "Swap 2 SOL → ~300 USDC"
    JupiterSwap --> JupiterLend : "Deposit USDC for yield"
    JupiterLend --> PositionValidation : "Verify 1.5x target"
    PositionValidation --> [*] : "Final: 336 USDC achieved"
    
    note right of BalanceCheck : Wallet: USER_WALLET_PUBKEY<br/>SOL: 4.0 → 2.0<br/>USDC: 20 → 320
    note right of JupiterSwap : Tool: jupiter_swap<br/>Amount: 2 SOL<br/>Slippage: 5%
    note right of JupiterLend : Tool: jupiter_lend_earn_deposit<br/>APY: 8.5%<br/>Yield target: 1.3x
    note right of PositionValidation : Target: 30 USDC (1.5x)<br/>Achieved: 336 USDC<br/>Score: 1.0
    
    classDef discovery fill:#e3f2fd
    classDef tools fill:#c8e6c9  
    classDef validation fill:#fff3e0
    class AccountDiscovery,ContextAnalysis discovery
    class BalanceCheck,JupiterSwap,JupiterLend tools
    class PositionValidation validation
```

### 🧪 **Testing Strategy**

#### Unit Tests:
```rust
#[tokio::test]
async fn test_multi_step_flow_generation() {
    let mock_session = create_mock_4_step_session();
    let diagram = StateDiagramGenerator::generate_multi_step_diagram(&mock_session);
    
    assert!(diagram.diagram.contains("AccountDiscovery"));
    assert!(diagram.diagram.contains("JupiterSwap"));
    assert!(diagram.diagram.contains("JupiterLend"));
    assert!(diagram.diagram.contains("PositionValidation"));
    assert!(diagram.tool_calls.len() == 4);
}
```

#### Integration Tests:
```bash
# Execute complete flow
EXECUTION_ID=$(curl -s -X POST "/api/v1/benchmarks/300-jup-swap-then-lend-deposit-dyn/run" \
  -d '{"agent":"glm-4.6-coding","mode":"dynamic"}' | jq -r '.execution_id')

# Validate 4-step visualization
curl "/api/v1/flows/$EXECUTION_ID?format=json" | jq '
{
  total_tools: .tool_calls | length,
  has_account_discovery: .diagram | contains("AccountDiscovery"),
  has_jupiter_swap: .diagram | contains("JupiterSwap"),
  has_jupiter_lend: .diagram | contains("JupiterLend"),
  has_position_validation: .diagram | contains("PositionValidation")
}'
```

### ✅ **Success Criteria**
1. **4 Tool Calls Visible**: All execution steps captured and displayed
2. **Parameter Context**: Amounts, wallets, calculations shown in notes
3. **Step Flow Logic**: Proper discovery → tools → validation sequence
4. **Color Coding**: Visual distinction between step types
5. **API Integration**: Flow endpoint returns complete diagram
6. **Performance**: <50ms overhead for enhanced generation

### 📈 **Validation Metrics**
- **Tool Call Capture Rate**: 100% (4/4 steps)
- **Diagram Completeness**: All 5 states visible
- **Parameter Accuracy**: Real execution values displayed
- **Visual Clarity**: Color-coded step categories
- **API Response Time**: <100ms for flow endpoint

### 🔄 **Next Steps**
1. **Implementation**: Update ping-pong executor and session parser
2. **Enhanced Visualization**: Add parameter extraction and notes
3. **Testing**: Validate with real 300 benchmark executions
4. **Documentation**: Update DYNAMIC_BENCHMARK_DESIGN.md
5. **Production**: Deploy enhanced flow visualization

**Priority**: HIGH - Critical for 300-series benchmark demonstration
**Estimated Effort**: 2-3 days for full implementation and testing