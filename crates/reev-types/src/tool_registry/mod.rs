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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_exist() {
        let tools = ToolRegistry::all_tools();
        assert!(!tools.is_empty());

        // Verify all tools are valid
        for tool in &tools {
            assert!(ToolRegistry::is_valid_tool(tool.as_str()));
        }
    }

    #[test]
    fn test_tool_categories() {
        let jupiter_tools = ToolRegistry::jupiter_tools();
        assert!(!jupiter_tools.is_empty());

        // All Jupiter tools should be valid
        for tool in &jupiter_tools {
            assert!(ToolRegistry::is_valid_tool(tool.as_str()));
        }
    }

    #[test]
    fn test_category_separation() {
        let discovery_tools = ToolRegistry::discovery_tools();
        let swap_tools = ToolRegistry::swap_tools();
        let lending_tools = ToolRegistry::lending_tools();
        let position_tools = ToolRegistry::position_tools();

        // Verify categories are non-overlapping
        let all_discovery: std::collections::HashSet<_> =
            discovery_tools.iter().map(|s| s.as_str()).collect();
        let all_swap: std::collections::HashSet<_> =
            swap_tools.iter().map(|s| s.as_str()).collect();
        let all_lending: std::collections::HashSet<_> =
            lending_tools.iter().map(|s| s.as_str()).collect();
        let all_position: std::collections::HashSet<_> =
            position_tools.iter().map(|s| s.as_str()).collect();

        // Verify we have expected number of tools
        assert_eq!(all_discovery.len(), 2);
        assert_eq!(all_swap.len(), 4);
        assert_eq!(all_lending.len(), 4);
        assert_eq!(all_position.len(), 1);

        // Tools should only appear in one category
        assert!(all_discovery
            .intersection(&all_swap)
            .collect::<Vec<_>>()
            .is_empty());
        assert!(all_discovery
            .intersection(&all_lending)
            .collect::<Vec<_>>()
            .is_empty());
        assert!(all_discovery
            .intersection(&all_position)
            .collect::<Vec<_>>()
            .is_empty());
    }
}
