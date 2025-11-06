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
    #[strum(serialize = "get_account_balance")]
    GetAccountBalance,

    /// Get Jupiter position info tool
    #[strum(serialize = "get_jupiter_position_info")]
    GetPositionInfo,

    /// Get Jupiter lend earn tokens tool
    #[strum(serialize = "get_jupiter_lend_earn_tokens")]
    GetLendEarnTokens,

    /// SOL transfer tool
    #[strum(serialize = "sol_transfer")]
    SolTransfer,

    /// SPL transfer tool
    #[strum(serialize = "spl_transfer")]
    SplTransfer,

    /// Jupiter swap tool for token exchanges
    #[strum(serialize = "jupiter_swap")]
    JupiterSwap,

    /// Jupiter swap flow tool
    #[strum(serialize = "jupiter_swap_flow")]
    JupiterSwapFlow,

    /// Jupiter earn tool (restricted to benchmarks)
    #[strum(serialize = "jupiter_earn")]
    JupiterEarn,

    /// Jupiter lend earn deposit tool
    #[strum(serialize = "jupiter_lend_earn_deposit")]
    JupiterLendEarnDeposit,

    /// Jupiter lend earn withdraw tool
    #[strum(serialize = "jupiter_lend_earn_withdraw")]
    JupiterLendEarnWithdraw,

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
            ToolName::GetAccountBalance
                | ToolName::GetPositionInfo
                | ToolName::GetLendEarnTokens
                | ToolName::SolTransfer
                | ToolName::SplTransfer
                | ToolName::JupiterSwap
                | ToolName::JupiterSwapFlow
                | ToolName::JupiterEarn
                | ToolName::JupiterLendEarnDeposit
                | ToolName::JupiterLendEarnWithdraw
                | ToolName::JupiterLendEarnMint
                | ToolName::JupiterLendEarnRedeem
        )
    }

    /// Get estimated execution time in milliseconds
    pub fn estimated_time_ms(&self) -> u64 {
        match self {
            ToolName::GetAccountBalance => 5000,
            ToolName::GetPositionInfo => 8000,
            ToolName::GetLendEarnTokens => 12000,
            ToolName::SolTransfer => 15000,
            ToolName::SplTransfer => 20000,
            ToolName::JupiterSwap => 30000,
            ToolName::JupiterSwapFlow => 35000,
            ToolName::JupiterEarn => 40000,
            ToolName::JupiterLendEarnDeposit => 50000,
            ToolName::JupiterLendEarnWithdraw => 25000,
            ToolName::JupiterLendEarnMint => 30000,
            ToolName::JupiterLendEarnRedeem => 35000,
        }
    }

    /// Check if tool is Jupiter-related
    pub fn is_jupiter_tool(&self) -> bool {
        matches!(
            self,
            ToolName::JupiterSwap
                | ToolName::JupiterSwapFlow
                | ToolName::JupiterEarn
                | ToolName::JupiterLendEarnDeposit
                | ToolName::JupiterLendEarnWithdraw
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
            ToolName::GetAccountBalance
            | ToolName::GetPositionInfo
            | ToolName::GetLendEarnTokens => ToolCategory::Discovery,
            ToolName::SolTransfer
            | ToolName::SplTransfer
            | ToolName::JupiterSwap
            | ToolName::JupiterSwapFlow => ToolCategory::Swap,
            ToolName::JupiterLendEarnDeposit
            | ToolName::JupiterLendEarnWithdraw
            | ToolName::JupiterLendEarnMint
            | ToolName::JupiterLendEarnRedeem => ToolCategory::Lending,
            ToolName::JupiterEarn => ToolCategory::Positions,
        }
    }

    /// Type-safe validation against actual tool names
    pub fn validate_against_registry(&self) -> bool {
        crate::tool_registry::ToolRegistry::is_valid_tool(self.as_str())
    }

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
            ToolCategory::Discovery => vec![
                ToolName::GetAccountBalance,
                ToolName::GetPositionInfo,
                ToolName::GetLendEarnTokens,
            ],
            ToolCategory::Swap => vec![
                ToolName::SolTransfer,
                ToolName::SplTransfer,
                ToolName::JupiterSwap,
                ToolName::JupiterSwapFlow,
            ],
            ToolCategory::Lending => vec![
                ToolName::JupiterLendEarnDeposit,
                ToolName::JupiterLendEarnWithdraw,
                ToolName::JupiterLendEarnMint,
                ToolName::JupiterLendEarnRedeem,
            ],
            ToolCategory::Positions => vec![ToolName::JupiterEarn],
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

        // Test all actual tool names
        assert!("get_account_balance".parse::<ToolName>().is_ok());
        assert!("get_jupiter_position_info".parse::<ToolName>().is_ok());
        assert!("get_jupiter_lend_earn_tokens".parse::<ToolName>().is_ok());
        assert!("sol_transfer".parse::<ToolName>().is_ok());
        assert!("spl_transfer".parse::<ToolName>().is_ok());
        assert!("jupiter_swap".parse::<ToolName>().is_ok());
        assert!("jupiter_swap_flow".parse::<ToolName>().is_ok());
        assert!("jupiter_earn".parse::<ToolName>().is_ok());
        assert!("jupiter_lend_earn_deposit".parse::<ToolName>().is_ok());
        assert!("jupiter_lend_earn_withdraw".parse::<ToolName>().is_ok());
        assert!("jupiter_lend_earn_mint".parse::<ToolName>().is_ok());
        assert!("jupiter_lend_earn_redeem".parse::<ToolName>().is_ok());
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
    fn test_spl_transfer_included() {
        let tool = ToolName::SplTransfer;
        assert_eq!(tool.to_string(), "spl_transfer");
        assert!(tool.requires_wallet());
        assert_eq!(tool.category(), ToolCategory::Swap);
    }

    #[test]
    fn test_correct_serializations() {
        // Test the fixes mentioned in issue #37
        let account_balance = ToolName::GetAccountBalance;
        assert_eq!(account_balance.to_string(), "get_account_balance");

        let lend_earn_tokens = ToolName::GetLendEarnTokens;
        assert_eq!(lend_earn_tokens.to_string(), "get_jupiter_lend_earn_tokens");

        let jupiter_withdraw = ToolName::JupiterLendEarnWithdraw;
        assert_eq!(jupiter_withdraw.to_string(), "jupiter_lend_earn_withdraw");
    }

    #[test]
    fn test_tool_categories() {
        let swap_tools = ToolCategory::Swap.tools();
        assert!(swap_tools.contains(&ToolName::JupiterSwap));
        assert!(swap_tools.contains(&ToolName::SplTransfer));
        assert!(!swap_tools.contains(&ToolName::GetAccountBalance));

        let discovery_tools = ToolCategory::Discovery.tools();
        assert!(discovery_tools.contains(&ToolName::GetAccountBalance));
        assert!(discovery_tools.contains(&ToolName::GetLendEarnTokens));
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
