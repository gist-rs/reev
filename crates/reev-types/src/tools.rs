//! Tool definitions with type-safe enums
//!
//! This module provides type-safe tool handling using enums with strum derive macros
//! to eliminate string-based tool name errors throughout the codebase.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

/// Available tool names with type safety
#[derive(
    Debug, Clone, Display, EnumString, IntoStaticStr, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub enum ToolName {
    /// Account balance checking tool
    #[strum(serialize = "account_balance")]
    AccountBalance,

    /// Jupiter swap tool for token exchanges
    #[strum(serialize = "jupiter_swap")]
    JupiterSwap,

    /// Jupiter lend/deposit tool
    #[strum(serialize = "jupiter_lend")]
    JupiterLend,

    /// Jupiter withdraw tool
    #[strum(serialize = "jupiter_withdraw")]
    JupiterWithdraw,

    /// Jupiter positions checking tool
    #[strum(serialize = "jupiter_positions")]
    JupiterPositions,

    /// Jupiter earn tool (restricted to benchmarks)
    #[strum(serialize = "jupiter_earn")]
    JupiterEarn,

    /// Generic transaction execution tool
    #[strum(serialize = "execute_transaction")]
    ExecuteTransaction,

    /// SOL transfer tool
    #[strum(serialize = "sol_transfer")]
    SolTransfer,

    /// Jupiter swap flow tool
    #[strum(serialize = "jupiter_swap_flow")]
    JupiterSwapFlow,

    /// Lend earn tokens tool
    #[strum(serialize = "lend_earn_tokens")]
    LendEarnTokens,

    /// Position info tool
    #[strum(serialize = "get_position_info")]
    GetPositionInfo,

    /// Jupiter lend earn deposit tool
    #[strum(serialize = "jupiter_lend_earn_deposit")]
    JupiterLendEarnDeposit,

    /// Jupiter lend earn mint tool
    #[strum(serialize = "jupiter_lend_earn_mint")]
    JupiterLendEarnMint,

    /// Jupiter lend earn redeem tool
    #[strum(serialize = "jupiter_lend_earn_redeem")]
    JupiterLendEarnRedeem,
}

impl ToolName {
    /// Check if tool requires wallet context
    pub fn requires_wallet(&self) -> bool {
        matches!(
            self,
            ToolName::AccountBalance
                | ToolName::JupiterPositions
                | ToolName::JupiterSwap
                | ToolName::JupiterLend
                | ToolName::JupiterWithdraw
                | ToolName::JupiterEarn
                | ToolName::SolTransfer
                | ToolName::JupiterSwapFlow
                | ToolName::LendEarnTokens
                | ToolName::GetPositionInfo
                | ToolName::JupiterLendEarnDeposit
                | ToolName::JupiterLendEarnMint
                | ToolName::JupiterLendEarnRedeem
        )
    }

    /// Get estimated execution time in milliseconds
    pub fn estimated_time_ms(&self) -> u64 {
        match self {
            ToolName::AccountBalance => 5000,
            ToolName::JupiterSwap => 30000,
            ToolName::JupiterLend => 45000,
            ToolName::JupiterWithdraw => 25000,
            ToolName::JupiterPositions => 10000,
            ToolName::JupiterEarn => 40000,
            ToolName::ExecuteTransaction => 20000,
            ToolName::SolTransfer => 15000,
            ToolName::JupiterSwapFlow => 35000,
            ToolName::LendEarnTokens => 12000,
            ToolName::GetPositionInfo => 8000,
            ToolName::JupiterLendEarnDeposit => 50000,
            ToolName::JupiterLendEarnMint => 30000,
            ToolName::JupiterLendEarnRedeem => 35000,
        }
    }

    /// Check if tool is Jupiter-related
    pub fn is_jupiter_tool(&self) -> bool {
        matches!(
            self,
            ToolName::JupiterSwap
                | ToolName::JupiterLend
                | ToolName::JupiterWithdraw
                | ToolName::JupiterPositions
                | ToolName::JupiterEarn
                | ToolName::JupiterSwapFlow
                | ToolName::JupiterLendEarnDeposit
                | ToolName::JupiterLendEarnMint
                | ToolName::JupiterLendEarnRedeem
        )
    }

    /// Check if tool is restricted to benchmarks only
    pub fn is_benchmark_restricted(&self) -> bool {
        matches!(self, ToolName::JupiterEarn)
    }

    /// Get tool category for grouping and analytics
    pub fn category(&self) -> ToolCategory {
        match self {
            ToolName::AccountBalance | ToolName::GetPositionInfo => ToolCategory::Discovery,
            ToolName::JupiterSwap
            | ToolName::JupiterSwapFlow
            | ToolName::ExecuteTransaction
            | ToolName::SolTransfer => ToolCategory::Swap,
            ToolName::JupiterLend
            | ToolName::JupiterWithdraw
            | ToolName::JupiterLendEarnDeposit
            | ToolName::JupiterLendEarnMint
            | ToolName::JupiterLendEarnRedeem
            | ToolName::LendEarnTokens => ToolCategory::Lending,
            ToolName::JupiterPositions | ToolName::JupiterEarn => ToolCategory::Positions,
        }
    }
}

/// Tool categories for organization and analytics
#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    /// Discovery and information tools
    Discovery,
    /// Token swap and exchange tools
    Swap,
    /// Lending and borrowing tools
    Lending,
    /// Position management tools
    Positions,
}

impl ToolCategory {
    /// Get all tools in this category
    pub fn tools(&self) -> Vec<ToolName> {
        match self {
            ToolCategory::Discovery => vec![ToolName::AccountBalance, ToolName::GetPositionInfo],
            ToolCategory::Swap => vec![
                ToolName::JupiterSwap,
                ToolName::JupiterSwapFlow,
                ToolName::ExecuteTransaction,
                ToolName::SolTransfer,
            ],
            ToolCategory::Lending => vec![
                ToolName::JupiterLend,
                ToolName::JupiterWithdraw,
                ToolName::JupiterLendEarnDeposit,
                ToolName::JupiterLendEarnMint,
                ToolName::JupiterLendEarnRedeem,
                ToolName::LendEarnTokens,
            ],
            ToolCategory::Positions => vec![ToolName::JupiterPositions, ToolName::JupiterEarn],
        }
    }
}

/// Tool execution context requirements
#[derive(Debug, Clone)]
pub struct ToolRequirements {
    /// Whether tool requires wallet context
    pub requires_wallet: bool,
    /// Whether tool requires network access
    pub requires_network: bool,
    /// Whether tool is restricted to benchmarks
    pub benchmark_only: bool,
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,
}

impl ToolRequirements {
    /// Get requirements for a specific tool
    pub fn for_tool(tool: &ToolName) -> Self {
        Self {
            requires_wallet: tool.requires_wallet(),
            requires_network: true, // All tools currently need network
            benchmark_only: tool.is_benchmark_restricted(),
            estimated_time_ms: tool.estimated_time_ms(),
        }
    }
}

/// Conversion utilities for backward compatibility
impl ToolName {
    /// Convert from string (with validation)
    pub fn from_str_safe(s: &str) -> Option<Self> {
        s.parse::<Self>().ok()
    }

    /// Get string representation for serialization
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    /// Check if tool name matches a pattern (for compatibility)
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        let tool_str = self.as_str();
        tool_str.contains(pattern) || pattern.contains(tool_str)
    }

    /// Convert to string vector for compatibility with existing code
    pub fn vec_to_string(tools: &[ToolName]) -> Vec<String> {
        tools.iter().map(|tool| tool.to_string()).collect()
    }

    /// Convert from string vector for compatibility with existing code
    pub fn vec_from_string(strings: &[String]) -> Vec<ToolName> {
        strings
            .iter()
            .filter_map(|s| s.parse::<ToolName>().ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name_serialization() {
        let tool = ToolName::JupiterSwap;
        assert_eq!(tool.to_string(), "jupiter_swap");
        assert_eq!(tool.as_str(), "jupiter_swap");
    }

    #[test]
    fn test_tool_name_deserialization() {
        let parsed: ToolName = "jupiter_swap".parse().unwrap();
        assert_eq!(parsed, ToolName::JupiterSwap);
    }

    #[test]
    fn test_tool_requirements() {
        let tool = ToolName::JupiterSwap;
        assert!(tool.requires_wallet());
        assert!(tool.is_jupiter_tool());
        assert!(!tool.is_benchmark_restricted());
        assert_eq!(tool.estimated_time_ms(), 30000);
    }

    #[test]
    fn test_tool_categories() {
        let swap_tools = ToolCategory::Swap.tools();
        assert!(swap_tools.contains(&ToolName::JupiterSwap));
        assert!(swap_tools.contains(&ToolName::JupiterSwapFlow));
        assert!(!swap_tools.contains(&ToolName::AccountBalance));
    }

    #[test]
    fn test_benchmark_restricted() {
        assert!(ToolName::JupiterEarn.is_benchmark_restricted());
        assert!(!ToolName::JupiterSwap.is_benchmark_restricted());
    }

    #[test]
    fn test_pattern_matching() {
        assert!(ToolName::JupiterSwap.matches_pattern("jupiter"));
        assert!(ToolName::JupiterSwap.matches_pattern("swap"));
        assert!(!ToolName::JupiterSwap.matches_pattern("lend"));
    }
}
