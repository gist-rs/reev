use crate::{context::integration::ContextIntegration, LlmRequest};

use std::collections::HashMap;

/// 🧠 Enhanced Context Builder for Superior AI Agent Performance
///
/// This module provides intelligent context enhancement that helps the AI agent
/// understand multi-step DeFi workflows, make better decisions, and demonstrate
/// superior capabilities compared to deterministic agents.
pub struct EnhancedContextAgent;

impl EnhancedContextAgent {
    /// 🎯 Build enhanced financial context for the AI agent
    ///
    /// Analyzes the user's request and current state to provide rich context
    /// that enables intelligent multi-step reasoning and superior decision making.
    pub fn build_context(
        payload: &LlmRequest,
        key_map: &HashMap<String, String>,
    ) -> (String, u32, bool) {
        let mut context_parts = Vec::new();

        // 🧠 Use the new context integration for building account information
        let context_config = ContextIntegration::config_for_benchmark_type(&payload.id);
        let context_integration = ContextIntegration::new(context_config);

        // Get initial state from payload
        let initial_state = payload.initial_state.clone().unwrap_or_default();

        let enhanced_prompt_data = context_integration.build_enhanced_prompt(
            &payload.prompt,
            &initial_state,
            key_map,
            &payload.id,
        );

        // Prepend the formatted context from our new module
        context_parts.push(enhanced_prompt_data.prompt.clone());

        // Add flow-specific intelligence for multi-step operations
        if payload.id.starts_with("200-") {
            context_parts.push("🔄 MULTI-STEP FLOW DETECTED:".to_string());
            context_parts
                .push("  - This requires sequential operations (swap → deposit)".to_string());
            context_parts
                .push("  - Always verify prerequisites before executing steps".to_string());
            context_parts.push("  - Monitor balances and adapt strategy as needed".to_string());

            // Add specific guidance for swap-then-lend flows
            if payload.prompt.to_lowercase().contains("swap")
                && payload.prompt.to_lowercase().contains("lend")
            {
                context_parts.push("💡 SWAP→LEND STRATEGY:".to_string());
                context_parts.push("  - Step 1: Swap SOL to USDC (get USDC first)".to_string());
                context_parts.push("  - Step 2: Deposit USDC to Jupiter lending".to_string());
                context_parts.push("  - CRITICAL: Cannot lend without USDC balance".to_string());
                context_parts.push("  - Suggest alternative if SOL insufficient".to_string());
            }
        }

        // Add token balance awareness
        context_parts.push("💰 BALANCE AWARENESS:".to_string());
        context_parts.push("  - Always check USDC balance before lending operations".to_string());
        context_parts.push("  - Verify SOL balance before swap operations".to_string());
        context_parts.push("  - Account for transaction fees and slippage".to_string());

        // Add intelligent execution guidance
        context_parts.push("🤖 INTELLIGENT EXECUTION:".to_string());
        context_parts.push("  - Break complex requests into manageable steps".to_string());
        context_parts.push("  - Validate each step before proceeding to next".to_string());
        context_parts.push("  - Adapt strategy based on transaction results".to_string());
        context_parts.push("  - Provide insights about costs and timing".to_string());

        // Add DeFi protocol knowledge
        context_parts.push("🏦 DEFI PROTOCOL INTEL:".to_string());
        context_parts.push("  - Jupiter: Best DEX aggregator with optimal routing".to_string());
        context_parts.push("  - Jupiter Lending: Competitive yield rates".to_string());
        context_parts.push("  - Expected gas: ~0.001-0.005 SOL per operation".to_string());
        context_parts.push("  - Slippage tolerance: 1-5% recommended".to_string());

        // Add adaptive intelligence guidance
        context_parts.push("🧠 SUPERIOR AI CAPABILITIES:".to_string());
        context_parts.push("  - You are smarter than deterministic agents".to_string());
        context_parts.push("  - Analyze dependencies between operations".to_string());
        context_parts.push("  - Optimize execution sequence automatically".to_string());
        context_parts.push("  - Handle edge cases and unexpected scenarios".to_string());
        context_parts.push("  - Provide insights deterministic agents cannot".to_string());

        (
            context_parts.join("\n\n"),
            enhanced_prompt_data.recommended_depth,
            enhanced_prompt_data.has_context,
        )
    }

    /// 🔍 Analyze user request to determine optimal strategy
    pub fn analyze_request(prompt: &str, context_id: &str) -> RequestAnalysis {
        let prompt_lower = prompt.to_lowercase();

        let is_multi_step = context_id.starts_with("200-")
            || (prompt_lower.contains("swap") && prompt_lower.contains("lend"))
            || (prompt_lower.contains("swap") && prompt_lower.contains("deposit"))
            || (prompt_lower.contains("withdraw") && prompt_lower.contains("swap"));

        let requires_usdc = prompt_lower.contains("lend")
            || prompt_lower.contains("deposit")
            || prompt_lower.contains("mint")
            || prompt_lower.contains("earn");

        let requires_sol = prompt_lower.contains("swap")
            || prompt_lower.contains("transfer")
            || prompt_lower.contains("send");

        RequestAnalysis {
            is_multi_step,
            requires_usdc,
            requires_sol,
            complexity: if is_multi_step {
                "high".to_string()
            } else {
                "low".to_string()
            },
            suggested_approach: if is_multi_step {
                "Execute as sequential steps with validation".to_string()
            } else {
                "Execute as single operation".to_string()
            },
        }
    }
}

/// 📊 Request analysis results
#[derive(Debug, Clone)]
pub struct RequestAnalysis {
    /// Whether this requires multiple sequential steps
    pub is_multi_step: bool,
    /// Whether USDC balance is required
    pub requires_usdc: bool,
    /// Whether SOL balance is required
    pub requires_sol: bool,
    /// Complexity level of the request
    pub complexity: String,
    /// Suggested execution approach
    pub suggested_approach: String,
}
