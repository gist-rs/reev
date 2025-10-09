//! Context integration module for enhanced agents
//!
//! This module provides integration points for adding account context
//! to LLM prompts, enabling smarter decision-making and reducing unnecessary tool calls.

use crate::context::{AccountContext, ContextBuilder};
use reev_lib::benchmark::InitialStateItem;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Context integration configuration
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Whether to provide account context
    pub enable_context: bool,
    /// Maximum conversation depth for context scenarios
    pub context_depth: u32,
    /// Maximum conversation depth for discovery scenarios
    pub discovery_depth: u32,
    /// Force discovery mode (for testing)
    pub force_discovery: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enable_context: true,
            context_depth: 3,
            discovery_depth: 7,
            force_discovery: false,
        }
    }
}

/// Context integration service for enhanced agents
pub struct ContextIntegration {
    pub builder: ContextBuilder,
    pub config: ContextConfig,
}

impl ContextIntegration {
    /// Create new context integration service
    pub fn new(config: ContextConfig) -> Self {
        Self {
            builder: ContextBuilder::new(),
            config,
        }
    }

    /// Build enhanced prompt with account context
    pub fn build_enhanced_prompt(
        &self,
        base_prompt: &str,
        initial_state: &[InitialStateItem],
        key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> EnhancedPrompt {
        info!(
            "[ContextIntegration] Building enhanced prompt for benchmark: {}",
            benchmark_id
        );

        let (context, should_use_context, depth) = if self.config.force_discovery {
            (
                self.builder.build_discovery_context(key_map, benchmark_id),
                false,
                self.config.discovery_depth,
            )
        } else if self.config.enable_context
            && self
                .builder
                .should_provide_context(benchmark_id, initial_state)
        {
            match self
                .builder
                .build_from_benchmark(initial_state, key_map, benchmark_id)
            {
                Ok(context) => {
                    // Validate context completeness
                    if let Err(validation_error) = self.builder.validate_context(&context) {
                        warn!(
                            "[ContextIntegration] Context validation failed: {}, falling back to discovery",
                            validation_error
                        );
                        (
                            self.builder.build_discovery_context(key_map, benchmark_id),
                            false,
                            self.config.discovery_depth,
                        )
                    } else {
                        (context, true, self.config.context_depth)
                    }
                }
                Err(e) => {
                    warn!(
                        "[ContextIntegration] Failed to build context: {}, using discovery mode",
                        e
                    );
                    (
                        self.builder.build_discovery_context(key_map, benchmark_id),
                        false,
                        self.config.discovery_depth,
                    )
                }
            }
        } else {
            (
                self.builder.build_minimal_context(key_map),
                false,
                self.config.discovery_depth,
            )
        };

        let enhanced_prompt =
            self.format_enhanced_prompt(base_prompt, &context, should_use_context);

        debug!(
            "[ContextIntegration] Enhanced prompt built: {} chars, context: {}, depth: {}",
            enhanced_prompt.len(),
            should_use_context,
            depth
        );

        EnhancedPrompt {
            prompt: enhanced_prompt,
            has_context: should_use_context,
            recommended_depth: depth,
            context_summary: self.summarize_context(&context),
        }
    }

    /// Format enhanced prompt with context integration
    fn format_enhanced_prompt(
        &self,
        base_prompt: &str,
        context: &AccountContext,
        has_context: bool,
    ) -> String {
        let mut enhanced = String::new();

        // Add context header
        enhanced.push_str("=== ACCOUNT CONTEXT ===\n");
        enhanced.push_str(&context.formatted_context);
        enhanced.push('\n');

        if has_context {
            enhanced.push_str("=== INSTRUCTION ===\n");
            enhanced.push_str(
                "Use the account information above to respond directly to the user request.\n",
            );
            enhanced.push_str("Avoid unnecessary balance or position checks since the information is already provided.\n");
            enhanced.push_str("Focus on executing the requested action efficiently.\n");
        } else {
            enhanced.push_str("=== INSTRUCTION ===\n");
            enhanced.push_str("Account information is limited. Use tools to discover current state before acting.\n");
            enhanced
                .push_str("You have extended conversation depth for exploration and discovery.\n");
        }

        enhanced.push_str("=== JUPITER LENDING TOOL SELECTION ===\n");
        enhanced.push_str("For Jupiter lending operations:\n");
        enhanced.push_str(
            "- Use 'jupiter_lend_earn_deposit' for token amounts (e.g., '0.1 SOL', '50 USDC')\n",
        );
        enhanced.push_str("- Use 'jupiter_lend_earn_mint' only for share quantities (rare)\n");
        enhanced.push_str("- Use 'jupiter_lend_earn_withdraw' to withdraw token amounts\n");
        enhanced
            .push_str("- Use 'jupiter_lend_earn_redeem' only to redeem share quantities (rare)\n");
        enhanced
            .push_str("MOST requests should use deposit/withdraw tools, not mint/redeem tools.\n");
        enhanced.push_str("IMPORTANT: Execute the correct tool and STOP. Do not call additional tools after successful execution.\n\n");

        enhanced.push_str("=== USER REQUEST ===\n");
        enhanced.push_str(base_prompt);
        enhanced.push('\n');

        enhanced
    }

    /// Summarize context for logging and debugging
    fn summarize_context(&self, context: &AccountContext) -> String {
        let mut summary = String::new();

        if context.sol_balance.is_some() {
            summary.push_str("SOL, ");
        }

        if !context.token_balances.is_empty() {
            summary.push_str(&format!("{} tokens, ", context.token_balances.len()));
        }

        if !context.lending_positions.is_empty() {
            summary.push_str(&format!("{} positions", context.lending_positions.len()));
        }

        if summary.is_empty() {
            summary = "minimal".to_string();
        } else if summary.ends_with(", ") {
            summary.truncate(summary.len() - 2);
        }

        summary
    }

    /// Determine optimal conversation depth based on context availability
    pub fn determine_optimal_depth(
        &self,
        initial_state: &[InitialStateItem],
        _key_map: &HashMap<String, String>,
        benchmark_id: &str,
    ) -> u32 {
        if self.config.force_discovery {
            return self.config.discovery_depth;
        }

        if self.config.enable_context
            && self
                .builder
                .should_provide_context(benchmark_id, initial_state)
        {
            // Check if we have rich account data
            let has_token_accounts = initial_state.iter().any(|item| item.data.is_some());
            let has_lending_positions = initial_state.iter().any(|item| {
                item.data
                    .as_ref()
                    .is_some_and(|data| self.builder.is_lending_token(&data.mint))
            });

            if has_token_accounts && has_lending_positions {
                self.config.context_depth
            } else if has_token_accounts {
                self.config.context_depth + 1 // Extra turn for verification
            } else {
                self.config.discovery_depth
            }
        } else {
            self.config.discovery_depth
        }
    }

    /// Create context configuration for specific benchmark types
    pub fn config_for_benchmark_type(benchmark_id: &str) -> ContextConfig {
        match benchmark_id {
            // Jupiter benchmarks benefit most from context
            id if id.contains("jup")
                && (id.contains("lend") || id.contains("earn") || id.contains("swap")) =>
            {
                ContextConfig {
                    enable_context: true,
                    context_depth: 3,
                    discovery_depth: 5,
                    force_discovery: false,
                }
            }
            // Complex multi-step benchmarks need more depth
            id if id.contains("200-") || id.contains("complex") => ContextConfig {
                enable_context: true,
                context_depth: 5,
                discovery_depth: 7,
                force_discovery: false,
            },
            // Simple benchmarks: SOL transfers use minimal context, SPL transfers need balance info
            id if id.contains("001-") => ContextConfig {
                enable_context: false,
                context_depth: 3,
                discovery_depth: 7,
                force_discovery: false,
            },
            // SPL transfers have token account data and should provide context
            id if id.contains("002-") => ContextConfig {
                enable_context: true,
                context_depth: 3,
                discovery_depth: 7,
                force_discovery: false,
            },
            // Default configuration
            _ => ContextConfig::default(),
        }
    }

    /// Analyze context effectiveness after LLM interaction
    pub fn analyze_context_effectiveness(
        &self,
        benchmark_id: &str,
        tool_calls_made: u32,
        depth_used: u32,
        success: bool,
    ) -> ContextAnalysis {
        let expected_depth = if self.config.enable_context {
            self.config.context_depth
        } else {
            self.config.discovery_depth
        };

        let efficiency_score = if tool_calls_made == 0 {
            0.0
        } else {
            expected_depth as f64 / tool_calls_made as f64
        };

        let depth_efficiency = depth_used as f64 / expected_depth as f64;

        ContextAnalysis {
            benchmark_id: benchmark_id.to_string(),
            tool_calls_made,
            depth_used,
            expected_depth,
            success,
            efficiency_score,
            depth_efficiency,
            recommendation: self.generate_recommendation(
                efficiency_score,
                depth_efficiency,
                success,
            ),
        }
    }

    /// Generate optimization recommendations
    fn generate_recommendation(
        &self,
        efficiency: f64,
        _depth_efficiency: f64,
        success: bool,
    ) -> String {
        if !success {
            return "Increase conversation depth or improve tool descriptions".to_string();
        }

        if efficiency > 1.5 {
            "Context is very effective - consider reducing depth further".to_string()
        } else if efficiency > 1.0 {
            "Context is effective - current configuration is good".to_string()
        } else if efficiency > 0.7 {
            "Moderate efficiency - consider improving context quality".to_string()
        } else {
            "Low efficiency - consider using discovery mode or enhancing context".to_string()
        }
    }
}

/// Enhanced prompt with integrated context
#[derive(Debug, Clone)]
pub struct EnhancedPrompt {
    /// The enhanced prompt text
    pub prompt: String,
    /// Whether account context was provided
    pub has_context: bool,
    /// Recommended conversation depth
    pub recommended_depth: u32,
    /// Summary of provided context
    pub context_summary: String,
}

/// Analysis of context effectiveness
#[derive(Debug, Clone)]
pub struct ContextAnalysis {
    pub benchmark_id: String,
    pub tool_calls_made: u32,
    pub depth_used: u32,
    pub expected_depth: u32,
    pub success: bool,
    pub efficiency_score: f64,
    pub depth_efficiency: f64,
    pub recommendation: String,
}
