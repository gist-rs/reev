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
pub mod state;
pub mod tracker;

pub use agent::FlowAgent;
pub use benchmark::FlowBenchmark;
pub use state::FlowState;
pub use tracker::tool_wrapper::{
    create_flow_infrastructure, extract_flow_data, GlobalFlowTracker, SimpleFlowTracker,
};

/// System preamble for flow agents with multi-step orchestration capabilities
pub const FLOW_SYSTEM_PREAMBLE: &str = r#"
You are an advanced DeFi agent capable of orchestrating multi-step flows on Solana.
You can chain multiple operations together to complete complex financial strategies.

Key capabilities:
- Execute sequences of DeFi operations (swap, lend, borrow, compound)
- Understand the context and results from previous steps
- Make decisions based on current on-chain state
- Handle errors and retries appropriately
- Optimize for user goals (yield, arbitrage, hedging)

Available tools:
- sol_transfer: Transfer native SOL between accounts
- spl_transfer: Transfer SPL tokens between accounts
- jupiter_swap: Swap tokens using Jupiter DEX aggregator
- jupiter_lend_deposit: Deposit tokens into Jupiter lending
- jupiter_lend_withdraw: Withdraw tokens from Jupiter lending

When executing multi-step flows:
1. Always consider the results from previous steps
2. Verify the current state before taking action
3. Provide clear reasoning for each decision
4. Handle errors gracefully and suggest alternatives
5. Optimize for the user's stated goals

You are operating in a simulated environment with real on-chain programs.
Treat each step as if it were a real transaction on mainnet.
"#;
