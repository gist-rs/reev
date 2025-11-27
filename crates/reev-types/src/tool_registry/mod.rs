//! Type-safe tool registry for centralized tool management
//!
//! This module provides a centralized way to manage tool names using Rig's
//! type-safe constants instead of hardcoded strings throughout the codebase.

use crate::tools::{ToolCategory, ToolName};

/// Tool registry for centralized tool management
pub struct ToolRegistry;

impl ToolRegistry {
    /// Get ALL tool names using enum-based generation
    pub fn all_tools() -> Vec<String> {
        use crate::tools::ToolName;

        vec![
            // Discovery tools
            ToolName::GetAccountBalance.to_string(),
            ToolName::GetJupiterLendEarnTokens.to_string(),
            // Transaction tools
            ToolName::SolTransfer.to_string(),
            ToolName::SplTransfer.to_string(),
            ToolName::JupiterSwap.to_string(),
            ToolName::JupiterSwapFlow.to_string(),
            // Jupiter lending tools
            ToolName::JupiterLendEarnDeposit.to_string(),
            ToolName::JupiterLendEarnWithdraw.to_string(),
            ToolName::JupiterLendEarnMint.to_string(),
            ToolName::JupiterLendEarnRedeem.to_string(),
            // Position tools
            ToolName::GetJupiterLendEarnPosition.to_string(),
        ]
    }

    /// Get tool category using enum
    pub fn category(tool_name: &str) -> Option<ToolCategory> {
        ToolName::from_str_safe(tool_name)?.category().into()
    }

    /// Validate tool name against Rig constants
    pub fn is_valid_tool(tool_name: &str) -> bool {
        Self::all_tools().iter().any(|tool| tool == tool_name)
    }

    /// Get tools by category
    pub fn tools_by_category(category: ToolCategory) -> Vec<String> {
        Self::all_tools()
            .into_iter()
            .filter(|tool| {
                if let Some(tool_name) = ToolName::from_str_safe(tool) {
                    tool_name.category() == category
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get Jupiter tools (restricted to benchmarks)
    pub fn jupiter_tools() -> Vec<String> {
        Self::all_tools()
            .into_iter()
            .filter(|tool| {
                if let Some(tool_name) = ToolName::from_str_safe(tool) {
                    tool_name.is_jupiter_tool()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get benchmark-restricted tools
    pub fn benchmark_restricted_tools() -> Vec<String> {
        Self::all_tools()
            .into_iter()
            .filter(|tool| {
                if let Some(tool_name) = ToolName::from_str_safe(tool) {
                    tool_name.is_benchmark_restricted()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get discovery tools
    pub fn discovery_tools() -> Vec<String> {
        Self::tools_by_category(ToolCategory::Discovery)
    }

    /// Get swap tools
    pub fn swap_tools() -> Vec<String> {
        Self::tools_by_category(ToolCategory::Swap)
    }

    /// Get lending tools
    pub fn lending_tools() -> Vec<String> {
        Self::tools_by_category(ToolCategory::Lending)
    }

    /// Get position tools
    pub fn position_tools() -> Vec<String> {
        Self::tools_by_category(ToolCategory::Positions)
    }
}
// Tests moved to tests/tool_registry_tests.rs
