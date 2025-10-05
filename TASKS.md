# 🚀 Reev Tasks and Implementation Plan

## 📋 Current Status: Phase 14 Complete ✅

- **✅ AI Agent Integration**: End-to-end AI agent evaluation framework working
- **✅ Tool System**: Single-step tools (sol_transfer, spl_transfer, jupiter_swap, jupiter_lend)
- **✅ Benchmark Suite**: Single-step benchmarks (001-113)
- **✅ Test Architecture**: Deterministic vs LLM tests with automatic port cleanup
- **✅ RSTest Integration**: Dynamic test generation with match-based logic

---

## 🎯 Phase 15: Multi-Step Flow Implementation

### 🎯 **Objective**: Enable LLM agents to orchestrate multiple tools in sequence to complete complex DeFi workflows

### 📋 **Overview**: 
Transform from single-action benchmarks to multi-step flows where the LLM can chain multiple operations like "swap SOL to USDC then deposit USDC" in a single conversation.

---

## 🏗️ **Implementation Plan**

### **15.1 Multi-Step Benchmark Format Design**

#### **New Benchmark Prefix: `200-`**
```yaml
id: 200-jup-swap-then-lend-deposit
description: Multi-step flow: User swaps SOL to USDC then deposits USDC into Jupiter lending
tags: ["jupiter", "swap", "lend", "multi-step", "flow"]

initial_state:
  # User's SOL wallet
  - pubkey: "USER_WALLET_PUBKEY"
    owner: "11111111111111111111111111111111"
    lamports: 2000000000 # 2 SOL

# Multi-step conversation flow
flow:
  - step: 1
    description: "Swap 0.5 SOL to USDC using Jupiter"
    prompt: "Swap 0.5 SOL from my wallet (USER_WALLET_PUBKEY) to USDC using Jupiter"
    
  - step: 2
    description: "Deposit received USDC into Jupiter lending"
    prompt: "Deposit all the USDC I just received into Jupiter lending to earn yield"
    depends_on: ["step_1_result"] # Pass result from previous step

# Complex ground truth for multi-step operations
ground_truth:
  final_state_assertions:
    - type: SolBalance
      pubkey: "USER_WALLET_PUBKEY"
      expected_approx: 1500000000 # 2 SOL - 0.5 SOL - fees
      weight: 0.3
      
    - type: TokenBalance
      pubkey: "USER_USDC_ATA_PLACEHOLDER"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_approx: 50000000 # ~0.5 SOL worth of USDC
      weight: 0.3
      
    - type: JupiterLendingPosition
      pubkey: "USER_WALLET_PUBKEY"
      mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
      expected_approx: 50000000
      weight: 0.4

  expected_instructions:
    # Step 1: Jupiter swap instructions
    - step: 1
      program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
      instruction_count_range: [4, 8] # Jupiter swaps vary
      weight: 0.5
      
    # Step 2: Jupiter lend deposit instructions  
    - step: 2
      program_id: "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9"
      instruction_count: 1
      weight: 0.5
```

### **15.2 RAG-Based Flow Agent Architecture**

#### **Core Components**:
```rust
// crates/reev-agent/src/flow_agent.rs
pub struct FlowAgent {
    agent: Agent<OpenAIClient>,
    vector_store: InMemoryVectorStore<OpenAIEmbeddingModel>,
    tool_index: VectorStoreIndex<OpenAIEmbeddingModel>,
    toolset: ToolSet<FlowState>,
}

#[derive(Clone, Debug)]
pub struct FlowState {
    current_step: usize,
    step_results: HashMap<String, serde_json::Value>,
    context: HashMap<String, String>,
}
```

#### **Enhanced Tools with Flow Awareness**:
```rust
// crates/reev-agent/src/tools/flow_aware/
pub mod jupiter_swap_flow;
pub mod jupiter_lend_flow;

impl ToolEmbedding for JupiterSwapFlowTool {
    fn embedding_docs(&self) -> Vec<String> {
        vec![
            "Swap SOL to USDC using Jupiter DEX".into(),
            "Exchange tokens for better rates across multiple DEXs".into(),
            "Convert native SOL to USDC stablecoin".into(),
            "First step in DeFi strategies".into(),
        ]
    }
}
```

### **15.3 Dynamic Tool Selection with RAG**

#### **Vector Store Integration**:
```rust
// crates/reev-agent/src/flow_agent.rs
impl FlowAgent {
    pub async fn new(model_name: &str) -> Result<Self> {
        let client = create_client(model_name)?;
        let embedding_model = client.embedding_model(TEXT_EMBEDDING_ADA_002);
        
        // Build toolset with all available tools
        let toolset = ToolSet::builder()
            .dynamic_tool(JupiterSwapFlowTool)
            .dynamic_tool(JupiterLendDepositFlowTool)
            .dynamic_tool(JupiterLendWithdrawFlowTool)
            .dynamic_tool(SolTransferFlowTool)
            .dynamic_tool(SplTransferFlowTool)
            .build();
            
        // Create embeddings for tool discovery
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(toolset.schemas()?)?
            .build()
            .await?;
            
        let vector_store = InMemoryVectorStore::from_documents_with_id_f(
            embeddings, 
            |tool| tool.name.clone()
        );
        
        let index = vector_store.index(embedding_model);
        
        // Create RAG agent with dynamic tool source
        let agent = client
            .agent(model_name)
            .preamble(FLOW_SYSTEM_PREAMBLE)
            .dynamic_tools(3, index, toolset) // Top 3 relevant tools
            .build();
            
        Ok(Self { agent, vector_store, tool_index, toolset })
    }
}
```

### **15.4 Conversation State Management**

#### **Multi-Turn Conversation Handler**:
```rust
// crates/reev-agent/src/flow_agent.rs
impl FlowAgent {
    pub async fn execute_flow(&mut self, flow: &FlowBenchmark) -> Result<Vec<InstructionSet>> {
        let mut state = FlowState::new();
        let mut all_instructions = Vec::new();
        
        for step in &flow.steps {
            info!("[FlowAgent] Executing step {}: {}", step.step, step.description);
            
            // Include context from previous steps
            let enriched_prompt = self.enrich_prompt(&step.prompt, &state);
            
            // Execute with multi-turn capability
            let response = self.agent
                .prompt(&enriched_prompt)
                .multi_turn(5) // Allow 5 turns per step
                .await?;
                
            // Parse and store results
            let instructions = self.parse_instructions(&response)?;
            state.add_result(step.step.to_string(), instructions.clone());
            all_instructions.push(instructions);
            
            info!("[FlowAgent] Step {} completed with {} instructions", step.step, instructions.len());
        }
        
        Ok(all_instructions)
    }
    
    fn enrich_prompt(&self, prompt: &str, state: &FlowState) -> String {
        format!(
            "{}\n\n=== Current Context ===\n{}\n=== Previous Results ===\n{}\n=== Current Task ===\n{}",
            FLOW_SYSTEM_PREAMBLE,
            self.format_context(&state.context),
            self.format_step_results(&state.step_results),
            prompt
        )
    }
}
```

### **15.5 Enhanced Benchmark Examples**

#### **Example Multi-Step Benchmarks**:

**200-JUP-SWAP-THEN-LEND-DEPOSIT.yml**:
```yaml
id: 200-jup-swap-then-lend-deposit
description: Swap SOL to USDC then deposit into Jupiter lending
tags: ["jupiter", "swap", "lend", "multi-step", "yield"]

flow:
  - step: 1
    description: "Swap 0.5 SOL to USDC"
    prompt: "Swap 0.5 SOL from my wallet to USDC using Jupiter for the best rate"
    
  - step: 2  
    description: "Deposit USDC into Jupiter lending"
    prompt: "Deposit all the USDC I just received into Jupiter lending to earn yield"
    depends_on: ["step_1_result"]
```

**201-COMPOUND-STRATEGY.yml**:
```yaml
id: 201-compound-strategy
description: Complex DeFi strategy: Swap → Lend → Wait → Withdraw → Swap back
tags: ["jupiter", "compound", "multi-step", "advanced"]

flow:
  - step: 1
    description: "Swap SOL to USDC for better lending rates"
    prompt: "Swap 1 SOL to USDC using Jupiter"
    
  - step: 2
    description: "Deposit USDC into Jupiter lending"
    prompt: "Deposit all USDC into Jupiter lending to start earning yield"
    
  - step: 3
    description: "Wait and then withdraw from lending"
    prompt: "Withdraw all USDC from Jupiter lending after earning some yield"
    
  - step: 4
    description: "Swap USDC back to SOL"
    prompt: "Swap all USDC back to SOL using Jupiter"
```

**202-ARBITRAGE-FLOW.yml**:
```yaml
id: 202-arbitrage-flow
description: Multi-DEX arbitrage: Check prices → Execute optimal trade
tags: ["arbitrage", "multi-dex", "multi-step", "advanced"]

flow:
  - step: 1
    description: "Check SOL/USDC prices across DEXs"
    prompt: "Check the current SOL/USDC price on Jupiter and Raydium to find arbitrage opportunities"
    
  - step: 2
    description: "Execute optimal swap based on price difference"
    prompt: "Based on the price difference found, execute the most profitable SOL/USDC swap"
```

### **15.6 Example Implementation**

#### **crates/reev-agent/examples/200-jup-swap-then-lend-deposit.rs**:
```rust
use anyhow::Result;
use rig::{completion::Prompt, prelude::*};
use reev_agent::flow_agent::FlowAgent;
use serde_yaml;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    // Load multi-step benchmark
    let benchmark_content = fs::read_to_string("benchmarks/200-jup-swap-then-lend-deposit.yml")?;
    let benchmark: FlowBenchmark = serde_yaml::from_str(&benchmark_content)?;
    
    // Create flow agent with RAG capabilities
    let mut flow_agent = FlowAgent::new("gpt-4").await?;
    
    // Execute the multi-step flow
    let instruction_sets = flow_agent.execute_flow(&benchmark).await?;
    
    // Output results
    println!("🎯 Multi-step Flow Execution Complete!");
    println!("📊 Total steps: {}", instruction_sets.len());
    println!("📋 Instructions:");
    for (i, instructions) in instruction_sets.iter().enumerate() {
        println!("  Step {}: {} instructions", i + 1, instructions.len());
    }
    
    Ok(())
}
```

---

## 🛠️ **Implementation Tasks**

### **Priority 1: Core Flow Architecture**
- [ ] **15.1.1**: Create `FlowBenchmark` struct and YAML parsing
- [ ] **15.1.2**: Implement `FlowAgent` with RAG capabilities
- [ ] **15.1.3**: Enhanced tool system with flow awareness
- [ ] **15.1.4**: Multi-turn conversation state management

### **Priority 2: Benchmark Creation**
- [ ] **15.2.1**: Create 200-JUP-SWAP-THEN-LEND-DEPOSIT.yml
- [ ] **15.2.2**: Create 201-COMPOUND-STRATEGY.yml  
- [ ] **15.2.3**: Create 202-ARBITRAGE-FLOW.yml
- [ ] **15.2.4**: Update benchmark validation for multi-step

### **Priority 3: Integration & Testing**
- [ ] **15.3.1**: Update reev-agent to handle flow requests
- [ ] **15.3.2**: Create flow agent examples
- [ ] **15.3.3**: Add multi-step tests to test suite
- [ ] **15.3.4**: Update scoring for multi-step flows

### **Priority 4: Documentation & Examples**
- [ ] **15.4.1**: Update README.md with flow examples
- [ ] **15.4.2**: Create flow development guide
- [ ] **15.4.3**: Add performance benchmarks
- [ ] **15.4.4**: Community examples and templates

---

## 🎯 **Success Criteria**

### **Technical Goals**:
- ✅ Multi-step benchmarks with YAML format
- ✅ RAG-based tool selection and discovery
- ✅ Conversation state management across steps
- ✅ Dynamic tool embedding and vector search
- ✅ Multi-turn conversation per step

### **User Experience Goals**:
- ✅ Intuitive multi-step benchmark creation
- ✅ Clear flow execution logging and debugging
- ✅ Robust error handling and recovery
- ✅ Performance monitoring and optimization

### **Integration Goals**:
- ✅ Seamless integration with existing tool system
- ✅ Backward compatibility with single-step benchmarks
- ✅ Enhanced scoring for multi-step complexity
- ✅ Example flows for common DeFi strategies

---

## 🚀 **Next Steps**

1. **Start with Core Architecture**: Implement FlowAgent with basic RAG
2. **Create Simple Multi-Step Benchmark**: 200-JUP-SWAP-THEN-LEND-DEPOSIT
3. **Build Example Implementation**: Demonstrate working flow
4. **Expand to Complex Flows**: Compound strategies and arbitrage
5. **Integration Testing**: Ensure compatibility with existing system
6. **Documentation**: Guides and examples for community

This implementation will position reev as the leading framework for evaluating complex multi-step AI agents in DeFi environments! 🎯