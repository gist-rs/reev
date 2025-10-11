//! 🔍 Tool Selector Module
//!
//! Simple, direct tool selection without RAG complexity.

use rig::tool::ToolDyn;
use std::collections::HashMap;
use tracing::info;

/// Simple tool selector that matches keywords to available tools
pub struct ToolSelector {
    tools: HashMap<String, Box<dyn ToolDyn>>,
}

impl ToolSelector {
    /// Create a new tool selector with available tools
    pub fn new(tools: HashMap<String, Box<dyn ToolDyn>>) -> Self {
        Self { tools }
    }

    /// Find relevant tools based on simple keyword matching
    pub async fn find_relevant_tools(&self, prompt: &str) -> Vec<String> {
        let mut relevant_tools = Vec::new();
        let prompt_lower = prompt.to_lowercase();

        // Simple keyword-based selection
        if (prompt_lower.contains("swap") || prompt_lower.contains("exchange"))
            && self.tools.contains_key("jupiter_swap")
        {
            relevant_tools.push("jupiter_swap".to_string());
        }

        if (prompt_lower.contains("mint") || prompt_lower.contains("deposit"))
            && self.tools.contains_key("jupiter_lend_earn_mint")
        {
            relevant_tools.push("jupiter_lend_earn_mint".to_string());
        }

        if (prompt_lower.contains("redeem") || prompt_lower.contains("withdraw"))
            && self.tools.contains_key("jupiter_lend_earn_redeem")
        {
            relevant_tools.push("jupiter_lend_earn_redeem".to_string());
        }

        if prompt_lower.contains("lend") && self.tools.contains_key("jupiter_lend_earn_deposit") {
            relevant_tools.push("jupiter_lend_earn_deposit".to_string());
        }

        if prompt_lower.contains("withdraw")
            && self.tools.contains_key("jupiter_lend_earn_withdraw")
        {
            relevant_tools.push("jupiter_lend_earn_withdraw".to_string());
        }

        // Check for Jupiter positions/earnings
        if prompt_lower.contains("position") && self.tools.contains_key("jupiter_earn") {
            relevant_tools.push("jupiter_earn".to_string());
        }

        info!(
            "[ToolSelector] Found {} relevant tools for prompt",
            relevant_tools.len()
        );
        relevant_tools
    }
}
