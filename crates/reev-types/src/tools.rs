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

    /// Get Jupiter lend earn position info tool
    #[strum(serialize = "get_jupiter_lend_earn_position")]
    GetJupiterLendEarnPosition,

    /// Get Jupiter lend earn tokens tool
    #[strum(serialize = "get_jupiter_lend_earn_tokens")]
    GetJupiterLendEarnTokens,

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

    /// Generic transaction execution tool
    #[strum(serialize = "execute_transaction")]
    ExecuteTransaction,
}

impl ToolName {
    /// Check if tool requires wallet context
    pub fn requires_wallet(&self) -> bool {
        matches!(
            self,
            ToolName::GetAccountBalance
                | ToolName::GetJupiterLendEarnPosition
                | ToolName::GetJupiterLendEarnTokens
                | ToolName::SolTransfer
                | ToolName::SplTransfer
                | ToolName::JupiterSwap
                | ToolName::JupiterSwapFlow
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
            ToolName::GetJupiterLendEarnPosition => 8000,
            ToolName::GetJupiterLendEarnTokens => 12000,
            ToolName::SolTransfer => 15000,
            ToolName::SplTransfer => 20000,
            ToolName::JupiterSwap => 30000,
            ToolName::JupiterSwapFlow => 35000,
            ToolName::JupiterLendEarnDeposit => 50000,
            ToolName::JupiterLendEarnWithdraw => 25000,
            ToolName::JupiterLendEarnMint => 30000,
            ToolName::JupiterLendEarnRedeem => 35000,
            ToolName::ExecuteTransaction => 20000,
        }
    }

    /// Check if tool is Jupiter-related
    pub fn is_jupiter_tool(&self) -> bool {
        matches!(
            self,
            ToolName::JupiterSwap
                | ToolName::JupiterSwapFlow
                | ToolName::GetJupiterLendEarnPosition
                | ToolName::JupiterLendEarnDeposit
                | ToolName::JupiterLendEarnWithdraw
                | ToolName::JupiterLendEarnMint
                | ToolName::JupiterLendEarnRedeem
        )
    }

    /// Check if tool is restricted to benchmarks only
    pub fn is_benchmark_restricted(&self) -> bool {
        matches!(self, ToolName::GetJupiterLendEarnPosition)
    }

    /// Check if tool is a transfer tool
    pub fn is_transfer_tool(&self) -> bool {
        matches!(self, ToolName::SolTransfer | ToolName::SplTransfer)
    }

    /// Get tool category for grouping and analytics
    pub fn category(&self) -> ToolCategory {
        match self {
            ToolName::GetAccountBalance | ToolName::GetJupiterLendEarnTokens => {
                ToolCategory::Discovery
            }
            ToolName::SolTransfer
            | ToolName::SplTransfer
            | ToolName::JupiterSwap
            | ToolName::JupiterSwapFlow
            | ToolName::ExecuteTransaction => ToolCategory::Swap,
            ToolName::JupiterLendEarnDeposit
            | ToolName::JupiterLendEarnWithdraw
            | ToolName::JupiterLendEarnMint
            | ToolName::JupiterLendEarnRedeem => ToolCategory::Lending,
            ToolName::GetJupiterLendEarnPosition => ToolCategory::Positions,
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
                ToolName::GetJupiterLendEarnTokens,
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
            ToolCategory::Positions => vec![ToolName::GetJupiterLendEarnPosition],
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
