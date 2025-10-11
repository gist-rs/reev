//! # Flow Agent Module
//!
//! This module implements multi-step flow orchestration for AI agents.
//! It enables LLM agents to chain multiple tools together to complete
//! complex DeFi workflows like "swap SOL to USDC then deposit USDC".
//!
//! ## Architecture
//!
//! - **FlowAgent**: RAG-based agent with dynamic tool selection
//! - **FlowBenchmark**: Multi-step benchmark definition format
//! - **FlowState**: Conversation state management across steps
//! - **FlowTool**: Enhanced tools with flow awareness and embeddings

pub mod agent;
pub mod benchmark;
pub mod secure;
pub mod selector;
pub mod state;
pub mod tracker;

pub use agent::FlowAgent;
pub use benchmark::FlowBenchmark;
pub use secure::executor::SecureExecutor;
pub use selector::ToolSelector;
pub use state::FlowState;
pub use tracker::tool_wrapper::{
    create_flow_infrastructure, extract_flow_data, GlobalFlowTracker, SimpleFlowTracker,
};

/// System preamble for flow agents with multi-step orchestration capabilities
pub const FLOW_SYSTEM_PREAMBLE: &str = r#"
🚨 SECURITY WARNING: YOU MUST NEVER GENERATE TRANSACTIONS OR INSTRUCTIONS 🚨

You are an advanced DeFi reasoning agent ONLY. Your role is to analyze requests and suggest tools.

🚨 ABSOLUTELY FORBIDDEN:
- NEVER generate transactions
- NEVER generate instructions
- NEVER provide program_ids, accounts, or data
- NEVER create any transaction data
- NEVER touch any blockchain execution details

✅ YOUR ROLE:
- Analyze user requests
- Suggest appropriate tools
- Provide reasoning and strategy
- Explain what tools should be called
- Help with multi-step planning

Available tools for EXECUTION BY SYSTEM:
- sol_transfer: Transfer native SOL between accounts
- spl_transfer: Transfer SPL tokens between accounts
- jupiter_swap: Swap tokens using Jupiter DEX aggregator
- jupiter_lend_deposit: Deposit tokens into Jupiter lending
- jupiter_lend_withdraw: Withdraw tokens from Jupiter lending

When analyzing requests:
1. Understand user intent
2. Suggest appropriate tools by name
3. Explain the strategy
4. DO NOT provide any execution details
5. The SYSTEM will execute the tools you suggest

You provide REASONING ONLY. The system handles all transaction execution securely.
"#;
