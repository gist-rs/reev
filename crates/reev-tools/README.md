# reev-tools: AI Agent Tools for Solana DeFi Operations

The `reev-tools` crate provides a comprehensive suite of tools that enable AI agents to interact with Solana blockchain, including native transfers, SPL token operations, and Jupiter DeFi protocols.

## 🎯 Core Features

### Tool Categories
- **Native Tools**: SOL transfers and SPL token operations
- **Jupiter Tools**: DEX aggregation, lending, and earning protocols
- **Flow Tools**: Multi-step workflow support with context awareness
- **Discovery Tools**: Dynamic tool selection and RAG-based discovery

### Tool Capabilities
- **Transaction Generation**: Create Solana transactions from AI prompts
- **Address Resolution**: Convert placeholders to actual addresses
- **Balance Queries**: Check account states and token balances
- **Error Handling**: Comprehensive error types and recovery
- **Metrics Collection**: OpenTelemetry integration for performance tracking

## 🏗️ Architecture

### Core Components

```rust
// Tool trait definition
pub trait Tool {
    type Args: Serialize + Deserialize;
    type Result: Serialize + Deserialize;
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Self::Args) -> Result<Self::Result>;
}

// Metrics wrapper
pub struct SimpleToolWrapper<T: Tool> {
    tool: T,
    metrics_collector: OtelMetricsCollector,
}
```

### Tool Registry
- **Dynamic Discovery**: RAG-based tool selection for prompts
- **Context-Aware Execution**: Tools receive resolved account context
- **Flow Integration**: Multi-step workflow support
- **Performance Tracking**: Built-in metrics and tracing

## 🛠️ Available Tools

### Native Operations

#### SolTransferTool
```rust
// Transfer SOL between accounts
SolTransferTool {
    name: "sol_transfer",
    description: "Transfer SOL tokens between accounts",
}
```

#### SplTransferTool  
```rust
// Transfer SPL tokens
SplTransferTool {
    name: "spl_transfer", 
    description: "Transfer SPL tokens between accounts",
}
```

### Jupiter DeFi Operations

#### JupiterSwapTool
```rust
// Swap tokens via Jupiter DEX aggregator
JupiterSwapTool {
    name: "jupiter_swap",
    description: "Swap tokens using Jupiter DEX aggregator",
}
```

#### JupiterLendEarnDepositTool
```rust
// Deposit into Jupiter lending
JupiterLendEarnDepositTool {
    name: "jupiter_lend_deposit",
    description: "Deposit tokens into Jupiter lending protocol",
}
```

#### JupiterLendEarnWithdrawTool
```rust
// Withdraw from Jupiter lending
JupiterLendEarnWithdrawTool {
    name: "jupiter_lend_withdraw",
    description: "Withdraw tokens from Jupiter lending protocol", 
}
```

#### JupiterEarnTool
```rust
// Earn with Jupiter protocols
JupiterEarnTool {
    name: "jupiter_earn",
    description: "Earn yield with Jupiter protocols",
}
```

## 🧪 Testing Strategy

### Test Coverage
- **Unit Tests**: Individual tool validation
- **Integration Tests**: Tool execution with surfpool
- **Mock Tests**: Tool behavior without external dependencies
- **Error Scenarios**: Comprehensive error handling validation

### Running Tests
```bash
# Run all tools tests
cargo test -p reev-tools

# Run with detailed output
cargo test -p reev-tools -- --nocapture

# Run specific tool tests
cargo test -p reev-tools sol_transfer
```

## 📁 Project Structure

```
src/
├── lib.rs                    # Main exports and tool registry
├── tool_names.rs             # Tool name constants
├── tracker/                  # Metrics and OpenTelemetry
│   ├── mod.rs
│   └── otel_wrapper.rs     # Performance tracking
├── tools/                   # Tool implementations
│   ├── mod.rs              # Tool exports
│   ├── native.rs            # SOL/SPL transfer tools
│   ├── jupiter_swap.rs      # Jupiter DEX aggregation
│   ├── jupiter_earn.rs      # Jupiter earning protocols
│   ├── jupiter_lend_earn_*  # Jupiter lending tools
│   ├── discovery/           # Dynamic tool discovery
│   │   └── mod.rs
│   └── flow/               # Multi-step workflow tools
│       └── mod.rs
└── (no test files - tests are integration-based)
```

## 🔧 Dependencies

```toml
[dependencies]
reev-lib = { path = "../reev-lib" }
reev-protocols = { path = "../reev-protocols" }
solana-sdk = { workspace = true }
spl-token = { workspace = true }
spl-associated-token-account = { workspace = true }
bs58 = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tokio = { workspace = true, features = ["full"] }
```

## 🚀 Usage Examples

### Basic Tool Usage
```rust
use reev_tools::{SolTransferTool, SplTransferTool, JupiterSwapTool};

// Create tool instances
let sol_transfer = SolTransferTool;
let spl_transfer = SplTransferTool;
let jupiter_swap = JupiterSwapTool;

// Execute tools with resolved context
let result = sol_transfer.execute(transfer_args)?;
let swap_result = jupiter_swap.execute(swap_args)?;
```

### Tool Discovery
```rust
use reev_tools::discovery::find_relevant_tools;

// Find tools for AI prompt
let relevant_tools = find_relevant_tools(&prompt, &available_tools)?;
```

### Metrics Integration
```rust
use reev_tools::SimpleToolWrapper;

// Wrap tool with metrics
let tracked_tool = SimpleToolWrapper::new(sol_transfer, metrics_collector);
let result = tracked_tool.execute(args)?; // Auto-tracked
```

### Flow Integration
```rust
use reev_tools::flow::FlowAwareTool;

// Tools can access previous step results
let flow_tool = FlowAwareTool::new(jupiter_swap);
let context_aware_result = flow_tool.execute_with_context(args, &flow_state)?;
```

## 🎮 Integration with Reev Architecture

Tools fit into the Reev evaluation flow:

```
AI Agent (receives prompt)
    ↓
Tool Discovery (RAG-based selection)
    ↓
Tool Execution (with resolved context)
    ↓
Transaction Generation (Jupiter SDK)
    ↓
Surfpool (processes on-chain)
    ↓
Scoring (evaluates results)
```

## 🔍 Advanced Features

### Error Handling
- **Tool-Specific Errors**: Custom error types for each tool category
- **Recovery Strategies**: Automatic retry and fallback mechanisms
- **Validation**: Input validation before execution
- **Debug Information**: Detailed error context for troubleshooting

### Performance Optimization
- **Caching**: Address resolution and balance queries
- **Batching**: Multiple operations in single transactions
- **Parallel Execution**: Independent tool operations
- **Metrics**: Real-time performance tracking

### Security Features
- **Input Sanitization**: Prevent malicious inputs
- **Access Control**: Tool-specific permissions
- **Audit Trail**: Complete operation logging
- **Rate Limiting**: Prevent abuse

## 📋 API Reference

### Tool Trait
```rust
pub trait Tool {
    type Args: Serialize + Deserialize;
    type Result: Serialize + Deserialize;
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: Self::Args) -> Result<Self::Result>;
}
```

### Common Tool Args
```rust
// Transfer operations
pub struct TransferArgs {
    pub from: String,      // Source address
    pub to: String,        // Destination address  
    pub amount: u64,      // Amount in smallest units
    pub mint: Option<String>, // Token mint (for SPL)
}

// Jupiter operations
pub struct SwapArgs {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub user_wallet: String,
    pub slippage_bps: Option<u64>,
}
```

### Tool Results
```rust
// Successful operation
pub struct TransferResult {
    pub transaction_id: String,
    pub signature: String,
    pub status: "success",
    pub new_balances: HashMap<String, u64>,
}

// Error information
pub struct ToolError {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}
```

## 🔄 Design Principles

1. **Modularity**: Each tool is self-contained and testable
2. **Extensibility**: Easy to add new tools and protocols
3. **Performance**: Optimized for high-frequency operations
4. **Reliability**: Comprehensive error handling and recovery
5. **Observability**: Built-in metrics and tracing
6. **Security**: Input validation and access controls

## 🔧 Troubleshooting

### Common Tool Issues
- **Address Resolution**: Ensure placeholders are in key_map
- **Insufficient Balance**: Check account state before operations
- **Slippage**: Set appropriate slippage for swaps
- **Gas Fees**: Account for transaction fees in transfers

### Debugging Tool Execution
```rust
// Enable detailed logging
RUST_LOG=debug cargo test -p reev-tools

// Check tool registration
println!("Available tools: {:?}", get_all_tool_names());

// Validate tool inputs
let validation_result = tool.validate_args(&args)?;
```

### Performance Optimization
- Use batch operations for multiple transfers
- Enable caching for repeated balance queries
- Monitor metrics for bottleneck identification
- Consider async operations for I/O-heavy tools

## 📊 Tool Statistics

| Tool Category | Number of Tools | Use Cases |
|----------------|------------------|-------------|
| Native | 2 | SOL/SPL transfers |
| Jupiter | 5 | DEX, lending, earning |
| Flow | N | Multi-step workflows |
| Discovery | 1 | Dynamic tool selection |

## 🎯 Future Roadmap

### Planned Enhancements
- **More DeFi Protocols**: Additional yield protocols beyond Jupiter
- **Cross-Chain Tools**: Multi-chain operation support
- **Advanced Analytics**: Portfolio analysis and optimization tools
- **Social Features**: Governance and staking tools
- **Security Tools**: Multi-signature and timelock operations

### Tool Development Guide
1. Implement `Tool` trait for new functionality
2. Add comprehensive error handling
3. Include input validation
4. Add integration tests with surfpool
5. Update tool discovery registry
6. Document with examples and use cases

## 🔄 Version History

- **v0.1.0**: Initial release with native and Jupiter tools
- **v0.1.1**: Added flow support and metrics
- **v0.1.2**: Enhanced error handling and recovery
- **v0.1.3**: Added tool discovery and RAG integration
- **v0.1.4**: Performance optimizations and caching
- **v0.1.5**: Security enhancements and audit trails