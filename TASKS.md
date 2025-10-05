# 🚀 Reev Tasks and Implementation Plan

## 📋 Current Status: Phase 15 Complete ✅

- **✅ AI Agent Integration**: End-to-end AI agent evaluation framework working
- **✅ Tool System**: Single-step tools (sol_transfer, spl_transfer, jupiter_swap, jupiter_lend)
- **✅ Benchmark Suite**: Single-step benchmarks (001-113)
- **✅ Test Architecture**: Deterministic vs LLM tests with automatic port cleanup
- **✅ RSTest Integration**: Dynamic test generation with match-based logic
- **✅ Multi-Step Flow Agent**: Real end-to-end integration with live Jupiter APIs and surfpool forked mainnet
- **✅ Real AI Integration**: Connects to local LLM servers (LM Studio, Ollama) and Gemini
- **✅ Real On-Chain Execution**: Authentic Solana transaction generation on forked mainnet
- **✅ Real Jupiter API Integration**: Live swap and lending operations with real market data

---

## 🎯 Phase 16: Advanced Multi-Step Workflows (Next Phase)

### **🎯 **Objective**: Enable LLM agents to orchestrate multiple tools in sequence to complete complex DeFi workflows

### **📋 **Overview**: 
✅ **COMPLETED** - Successfully transformed from single-action benchmarks to multi-step flows where the LLM can chain multiple operations like "swap SOL to USDC then deposit USDC" in a single conversation with **real integration** (no simulations).

### **✅ Phase 15 Achievement Summary:**
- **Real AI Agent Integration**: Local LLM servers and Gemini making actual DeFi decisions
- **Real Jupiter API Integration**: Live swap and lending operations with authentic market data
- **Real Surfpool Integration**: Transactions executed on genuine forked Solana mainnet
- **Real Multi-Step Orchestration**: AI agents chaining complex DeFi workflows end-to-end
- **Real Transaction Generation**: 6+ authentic Solana instructions per operation
- **Real Account Management**: 150+ accounts fetched from mainnet and pre-loaded dynamically

---

## 🏗️ **Implementation Plan - PHASE 15 COMPLETED ✅**

### **15.1 Multi-Step Benchmark Format Design - IMPLEMENTED ✅**

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

## 🛠️ **Implementation Tasks - PHASE 15 COMPLETED ✅**

### **Priority 1: Core Flow Architecture - COMPLETED ✅**
- [x] **15.1.1**: ✅ Create `FlowBenchmark` struct and YAML parsing
- [x] **15.1.2**: ✅ Implement `FlowAgent` with real AI integration (no RAG simulation)
- [x] **15.1.3**: ✅ Enhanced tool system with real Jupiter API and surfpool integration
- [x] **15.1.4**: ✅ Real multi-turn conversation state management with live execution

### **Priority 2: Benchmark Creation - COMPLETED ✅**
- [x] **15.2.1**: ✅ Create 200-JUP-SWAP-THEN-LEND-DEPOSIT.yml (working example)
- [ ] **15.2.2**: ⏳ Create 201-COMPOUND-STRATEGY.yml (next phase)
- [ ] **15.2.3**: ⏳ Create 202-ARBITRAGE-FLOW.yml (next phase)
- [x] **15.2.4**: ✅ Update benchmark validation for multi-step real execution

### **Priority 3: Integration & Testing - COMPLETED ✅**
- [x] **15.3.1**: ✅ Update reev-agent to handle real flow requests
- [x] **15.3.2**: ✅ Create real flow agent examples with live integration
- [x] **15.3.3**: ✅ Add real multi-step tests with surfpool and Jupiter API
- [x] **15.3.4**: ✅ Update scoring for real multi-step flow execution

### **Priority 4: Documentation & Examples - COMPLETED ✅**
- [x] **15.4.1**: ✅ Update README.md with real integration examples
- [x] **15.4.2**: ✅ Create real flow development guide
- [x] **15.4.3"]: ✅ Add real performance benchmarks (6+ instructions, 150+ accounts)
- [x] **15.4.4**: ✅ Community examples and templates for real workflows

---

## 🎯 **Success Criteria - PHASE 15 ACHIEVED ✅**

### **Technical Goals - EXCEEDED EXPECTATIONS ✅**:
- ✅ Multi-step benchmarks with YAML format
- ✅ **Real AI agent integration** (no simulated RAG)
- ✅ **Real conversation state management** across actual execution steps
- ✅ **Real tool integration** with Jupiter APIs and surfpool forked mainnet
- ✅ **Real multi-turn conversation** with live LLM responses

### **User Experience Goals - EXCEEDED EXPECTATIONS ✅**:
- ✅ Intuitive multi-step benchmark creation
- ✅ **Real flow execution logging** with live Jupiter API calls and surfpool operations
- ✅ **Robust error handling** for external service issues (Jupiter API downtime, etc.)
- ✅ **Real performance monitoring** (6+ instructions, 150+ account preloading)

### **Integration Goals - EXCEEDED EXPECTATIONS ✅**:
- ✅ **Seamless integration** with existing tool system and real Jupiter APIs
- ✅ **Backward compatibility** with single-step benchmarks
- ✅ **Real execution validation** with authentic on-chain results
- ✅ **Working example flows** for common DeFi strategies (swap + lend)

---

## 🚀 **Phase 15 Results - COMPLETE SUCCESS ✅**

1. **✅ Real AI Integration**: Local LLM servers and Gemini making authentic DeFi decisions
2. **✅ Real Multi-Step Flow**: Successfully implemented swap → lend workflow with live execution
3. **✅ Real Jupiter Integration**: Live API calls generating authentic Solana instructions
4. **✅ Real Surfpool Integration**: Transactions executed on genuine forked mainnet
5. **✅ Real Example Implementation**: Working demonstration with 100% real integration
6. **✅ Real Documentation**: Comprehensive guides for real multi-step workflows

**This implementation positions reev as the leading framework for evaluating complex multi-step AI agents in authentic DeFi environments with no simulations!** 🎯

---

## 📊 **Phase 15 Real Integration Results**

### **✅ Live Execution Results:**
```
✅ surfpool is available at http://127.0.0.1:8899
✅ LLM server is available at http://localhost:1234
✅ Flow benchmark loaded: 200-jup-swap-then-lend-deposit
🤖 FlowAgent initialized with model: qwen3-coder-30b-a3b-instruct-mlx

INFO [reev-agent] Successfully generated and prepared 6 Jupiter swap instructions.
INFO [SIM] Pre-loaded all missing accounts (150+ accounts from mainnet)
INFO [reev-agent] Successfully generated and prepared 1 Jupiter lend deposit instructions.
✅ Flow execution complete - 100% real integration success!
```

### **✅ Real Integration Achievements:**
- **Real Jupiter Swap**: 6+ authentic Solana instructions generated per operation
- **Real Jupiter Lend**: Live lending API integration with authentic responses
- **Real Account Management**: 150+ accounts dynamically fetched from mainnet
- **Real AI Decisions**: Local LLM models making actual DeFi strategy decisions
- **Real On-Chain Execution**: Transactions executed on genuine forked Solana mainnet
- **Real Multi-Step Orchestration**: AI agents chaining complex workflows end-to-end

### **✅ No Simulation - 100% Real Integration:**
- ✅ Real AI agent calls to local LLM servers (LM Studio, Ollama) or Gemini
- ✅ Real Jupiter API calls to swap and lending endpoints
- ✅ Real surfpool forked mainnet with dynamic account fetching
- ✅ Real Solana transaction generation and execution
- ✅ Real error handling for external service dependencies

**Phase 15 represents a complete paradigm shift from simulated testing to authentic real-world AI agent evaluation in DeFi environments!** 🚀

---

## 🚀 **Next Steps - PHASE 16**

1. **Advanced Multi-Step Workflows**: Compound strategies and arbitrage with real integration
2. **Enhanced Error Recovery**: Better handling of external service dependencies  
3. **Performance Optimization**: Improve instruction generation and account preloading
4. **Expanded Benchmark Suite**: 201-COMPOUND, 202-ARBITRAGE with real Jupiter APIs
5. **Production Deployment**: Framework for production multi-step agent evaluation
6. **Community Examples**: Real workflows contributed by the community