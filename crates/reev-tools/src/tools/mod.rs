pub mod discovery;
pub mod flow;
pub mod jupiter_earn;
pub mod jupiter_lend_earn_deposit;
pub mod jupiter_lend_earn_mint_redeem;
pub mod jupiter_lend_earn_withdraw;
pub mod jupiter_swap;
pub mod native;

pub use discovery::*;
pub use flow::*;
pub use jupiter_earn::JupiterEarnTool;
pub use jupiter_lend_earn_deposit::JupiterLendEarnDepositTool;
pub use jupiter_lend_earn_mint_redeem::JupiterLendEarnMintTool;
pub use jupiter_lend_earn_mint_redeem::JupiterLendEarnRedeemTool;
pub use jupiter_lend_earn_withdraw::JupiterLendEarnWithdrawTool;
pub use jupiter_swap::JupiterSwapTool;
pub use native::SolTransferTool;
pub use native::SplTransferTool;

// Re-export OpenTelemetry tool wrapper for proper tracing
pub use crate::tracker::otel_wrapper::{
    init_otel_tool_tracing, OtelMetricsCollector, OtelToolWrapper, ToolExecutionMetrics,
};
