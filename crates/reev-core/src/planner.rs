//! Planner for Phase 1 LLM Integration
//!
//! This module implements the Phase 1 LLM integration for structured YML generation
//! from user prompts. It handles language refinement, intent analysis, and creates
//! structured YML flows with wallet context and steps.

use crate::context::ContextResolver;
use crate::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Planner for Phase 1 LLM integration
pub struct Planner {
    /// Context resolver for wallet information
    context_resolver: ContextResolver,
    /// LLM client for generating flows
    llm_client: Option<Box<dyn LlmClient>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(context_resolver: ContextResolver) -> Self {
        Self {
            context_resolver,
            llm_client: None,
        }
    }

    /// Set the LLM client
    pub fn with_llm_client(mut self, client: Box<dyn LlmClient>) -> Self {
        self.llm_client = Some(client);
        self
    }

    /// Refine and plan: Phase 1 LLM integration for structured YML generation
    #[instrument(skip(self))]
    pub async fn refine_and_plan(&self, prompt: &str, wallet_pubkey: &str) -> Result<YmlFlow> {
        info!("Starting Phase 1: Refine and Plan for prompt: {}", prompt);

        // Resolve wallet context
        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(wallet_pubkey)
            .await?;
        debug!("Resolved wallet context for {}", wallet_pubkey);

        // Get placeholder mappings
        let mappings = self
            .context_resolver
            .get_placeholder_mappings(&wallet_context)
            .await;

        // Build YML wallet info
        let _wallet_info =
            YmlWalletInfo::new(wallet_pubkey.to_string(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Generate structured YML flow using LLM or rule-based fallback
        let yml_flow = match &self.llm_client {
            Some(client) => {
                self.generate_flow_with_llm(prompt, &wallet_context, client.as_ref())
                    .await?
            }
            None => {
                self.generate_flow_rule_based(prompt, &wallet_context, &mappings)
                    .await?
            }
        };

        debug!("Generated YML flow: {}", yml_flow.flow_id);
        Ok(yml_flow)
    }

    /// Generate flow using LLM
    pub async fn generate_flow_with_llm(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        llm_client: &dyn LlmClient,
    ) -> Result<YmlFlow> {
        // Build structured prompt for LLM
        let structured_prompt = self.build_structured_prompt(prompt, wallet_context)?;

        debug!("Calling LLM with structured prompt");
        let llm_response = llm_client.generate_flow(&structured_prompt).await?;
        debug!("Received LLM response");

        // Parse LLM response into YML flow
        let yml_flow: YmlFlow = serde_yaml::from_str(&llm_response)
            .map_err(|e| anyhow!("Failed to parse LLM response as YML: {e}"))?;

        // Validate the generated flow
        yml_flow
            .validate()
            .map_err(|e| anyhow!("Invalid YML flow generated: {e}"))?;

        Ok(yml_flow)
    }

    /// Generate flow using rule-based approach as fallback
    async fn generate_flow_rule_based(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        _mappings: &HashMap<String, String>,
    ) -> Result<YmlFlow> {
        warn!("Using rule-based fallback for prompt: {}", prompt);

        // Parse the prompt for intent
        let intent = self.parse_intent(prompt)?;

        // Generate flow based on intent
        match intent {
            UserIntent::Swap {
                from,
                to,
                amount,
                percentage,
            } => {
                self.create_swap_flow(prompt, wallet_context, &from, &to, amount, percentage)
                    .await
            }
            UserIntent::Lend {
                mint,
                amount,
                percentage,
            } => {
                self.create_lend_flow(prompt, wallet_context, &mint, amount, percentage)
                    .await
            }
            UserIntent::SwapThenLend {
                from,
                to,
                amount,
                percentage,
            } => {
                self.create_swap_then_lend_flow(
                    prompt,
                    wallet_context,
                    &from,
                    &to,
                    amount,
                    percentage,
                )
                .await
            }
            UserIntent::Unknown => Err(anyhow!("Unable to determine intent from prompt: {prompt}")),
        }
    }

    /// Build structured prompt for LLM
    fn build_structured_prompt(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
    ) -> Result<String> {
        let flow_id = Uuid::now_v7().to_string();

        // Extract wallet information for the prompt
        let wallet_info = json!({
            "pubkey": wallet_context.owner,
            "sol_balance": wallet_context.sol_balance_sol(),
            "total_value_usd": wallet_context.total_value_usd
        });

        // Extract token information for the prompt
        let mut tokens = Vec::new();
        for (mint, balance) in &wallet_context.token_balances {
            let token_info = json!({
                "mint": mint,
                "balance": balance.balance,
                "decimals": balance.decimals.unwrap_or(0),
                "symbol": balance.symbol.as_ref().unwrap_or(&"UNKNOWN".to_string())
            });
            tokens.push(token_info);
        }

        let structured_prompt = format!(
            r#"You are a DeFi flow planner. Generate a structured YAML flow from the user's prompt.

User Prompt: {}

Wallet Info:
{}

Available Tokens:
{}

Instructions:
1. Generate a valid YAML flow following this structure:
   flow_id: {}
   user_prompt: {}
   subject_wallet_info:
     pubkey: <wallet_pubkey>
     lamports: <lamports>
     total_value_usd: <total_value>
     tokens:
       - mint: <token_mint>
         balance: <balance>
         decimals: <decimals>
         symbol: <symbol>
   steps:
     - step_id: <step_id>
       prompt: "<step_prompt>"
       context: "<step_context>"
       critical: true
   ground_truth:
     final_state_assertions:
       - assertion_type: "<assertion_type>"
         pubkey: <pubkey>
         expected_change_gte: <expected_change>
     error_tolerance: 0.01

2. Only include necessary steps.
3. Ensure prompts are clear and specific.
4. Include appropriate guardrails in ground_truth.

Generate the YAML flow:"#,
            prompt,
            serde_json::to_string_pretty(&wallet_info)?,
            serde_json::to_string_pretty(&tokens)?,
            flow_id,
            prompt
        );

        Ok(structured_prompt)
    }

    /// Parse user intent from prompt
    fn parse_intent(&self, prompt: &str) -> Result<UserIntent> {
        let prompt_lower = prompt.to_lowercase();

        // Extract percentage if present
        let percentage = self.extract_percentage(prompt);

        // Handle swap intents
        if prompt_lower.contains("swap") || prompt_lower.contains("exchange") {
            if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
                // Swap then lend
                let (from, to, amount) = self.extract_swap_params(&prompt_lower)?;
                Ok(UserIntent::SwapThenLend {
                    from,
                    to,
                    amount,
                    percentage,
                })
            } else {
                // Just swap
                let (from, to, amount) = self.extract_swap_params(&prompt_lower)?;
                Ok(UserIntent::Swap {
                    from,
                    to,
                    amount,
                    percentage,
                })
            }
        }
        // Handle lend intents
        else if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
            let (mint, amount) = self.extract_lend_params(&prompt_lower)?;
            Ok(UserIntent::Lend {
                mint,
                amount,
                percentage,
            })
        }
        // Unknown intent
        else {
            Ok(UserIntent::Unknown)
        }
    }

    /// Extract swap parameters from prompt
    fn extract_swap_params(&self, prompt: &str) -> Result<(String, String, f64)> {
        // Default values
        let mut from = "SOL".to_string();
        let mut to = "USDC".to_string();
        let mut amount = 1.0;

        // Try to extract "from" token
        for token in ["SOL", "USDC", "USDT", "ETH", "BTC"] {
            if prompt.contains(&format!("{} ", token.to_lowercase()))
                || prompt.contains(&format!(" {}", token.to_lowercase()))
            {
                from = token.to_string();
                break;
            }
        }

        // Try to extract "to" token
        for token in ["SOL", "USDC", "USDT", "ETH", "BTC"] {
            if token != from
                && (prompt.contains(&format!(" {}", token.to_lowercase()))
                    || prompt.contains(&format!(" to {}", token.to_lowercase())))
            {
                to = token.to_string();
                break;
            }
        }

        // Try to extract amount
        if let Some(percentage) = self.extract_percentage(prompt) {
            // Percentage detected
            // We'll use percentage in the flow creation
            return Ok((from, to, percentage));
        } else {
            // Look for specific amount
            let amount_regex =
                regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
            if let Some(captures) = amount_regex.captures(prompt) {
                if let Ok(val) = captures[1].parse::<f64>() {
                    amount = val;
                }
            }
        }

        Ok((from, to, amount))
    }

    /// Extract lend parameters from prompt
    fn extract_lend_params(&self, prompt: &str) -> Result<(String, f64)> {
        // Default values
        let mut mint = "USDC".to_string();
        let mut amount = 100.0;

        // Try to extract token
        for token in ["SOL", "USDC", "USDT", "ETH", "BTC"] {
            if prompt.contains(&token.to_lowercase()) {
                mint = token.to_string();
                break;
            }
        }

        // Try to extract amount
        if let Some(percentage) = self.extract_percentage(prompt) {
            return Ok((mint, percentage));
        } else {
            let amount_regex =
                regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
            if let Some(captures) = amount_regex.captures(prompt) {
                if let Ok(val) = captures[1].parse::<f64>() {
                    amount = val;
                }
            }
        }

        Ok((mint, amount))
    }

    /// Create a swap flow
    async fn create_swap_flow(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        from: &str,
        to: &str,
        amount: f64,
        _percentage: Option<f64>,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Convert tokens to mint addresses
        let _from_mint = self.token_to_mint(from)?;
        let _to_mint = self.token_to_mint(to)?;

        // Calculate amount in lamports if it's a percentage
        let swap_amount = if from == "SOL" {
            wallet_context.sol_balance_sol() * amount / 100.0
        } else {
            // For tokens, use a default amount for now
            amount
        };

        // Create wallet info
        let wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Create swap step
        let swap_step = YmlStep::new(
            "swap".to_string(),
            format!("swap {swap_amount} {from} to {to}"),
            format!("Exchange {swap_amount} {from} for {to}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_critical(true);

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_gte(-((swap_amount * 1_000_000_000.0) + 10_000_000.0)), // Include fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
            .with_error_tolerance(0.01);

        // Create flow
        let flow = YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
            .with_step(swap_step)
            .with_ground_truth(ground_truth);

        Ok(flow)
    }

    /// Create a lend flow
    async fn create_lend_flow(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        mint: &str,
        amount: f64,
        _percentage: Option<f64>,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Convert token to mint address
        let _token_mint = self.token_to_mint(mint)?;

        // Create wallet info
        let wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Create lend step
        let lend_step = YmlStep::new(
            "lend".to_string(),
            format!("lend {amount} {mint} to jupiter"),
            format!("Deposit {amount} {mint} in Jupiter earn for yield"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_critical(true);

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
            .with_error_tolerance(0.01);

        // Create flow
        let flow = YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
            .with_step(lend_step)
            .with_ground_truth(ground_truth);

        Ok(flow)
    }

    /// Create a swap then lend flow
    async fn create_swap_then_lend_flow(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        from: &str,
        to: &str,
        amount: f64,
        _percentage: Option<f64>,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Convert tokens to mint addresses
        let _from_mint = self.token_to_mint(from)?;
        let _to_mint = self.token_to_mint(to)?;

        // Calculate amount in lamports if it's a percentage
        let swap_amount = if from == "SOL" {
            wallet_context.sol_balance_sol() * amount / 100.0
        } else {
            // For tokens, use a default amount for now
            amount
        };

        // Create wallet info
        let wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Create swap step
        let swap_step = YmlStep::new(
            "swap".to_string(),
            format!("swap {swap_amount} {from} to {to}"),
            format!("Exchange {swap_amount} {from} for {to}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
        .with_critical(true);

        // Create lend step
        let lend_step = YmlStep::new(
            "lend".to_string(),
            format!("lend {{SWAPPED_AMOUNT}} {to} to jupiter"),
            format!("Lend swapped {to} for yield"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
        .with_critical(true);

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_gte(-((swap_amount * 1_000_000_000.0) + 10_000_000.0)), // Include fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::JupiterSwap, true))
            .with_tool_call(YmlToolCall::new(ToolName::JupiterLendEarnDeposit, true))
            .with_error_tolerance(0.01);

        // Create flow
        let flow = YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
            .with_step(swap_step)
            .with_step(lend_step)
            .with_ground_truth(ground_truth);

        Ok(flow)
    }

    /// Convert token symbol to mint address
    fn token_to_mint(&self, token: &str) -> Result<String> {
        // Simple mapping for common tokens
        match token.to_uppercase().as_str() {
            "SOL" => Ok("So11111111111111111111111111111111111111112".to_string()),
            "USDC" => Ok("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            "USDT" => Ok("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
            _ => Err(anyhow!("Unknown token: {token}")),
        }
    }

    /// Extract percentage from prompt
    fn extract_percentage(&self, prompt: &str) -> Option<f64> {
        let regex = regex::Regex::new(r"(\d+\.?\d*)%").unwrap();
        regex
            .captures(prompt)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    }
}

/// User intent extracted from prompt
#[derive(Debug, Clone)]
enum UserIntent {
    Swap {
        from: String,
        to: String,
        amount: f64,
        percentage: Option<f64>,
    },
    Lend {
        mint: String,
        amount: f64,
        percentage: Option<f64>,
    },
    SwapThenLend {
        from: String,
        to: String,
        amount: f64,
        percentage: Option<f64>,
    },
    Unknown,
}

/// LLM client trait for generating flows
/// Trait for LLM client abstraction
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_flow(&self, prompt: &str) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLlmClient;

    #[async_trait::async_trait]
    impl LlmClient for MockLlmClient {
        async fn generate_flow(&self, _prompt: &str) -> Result<String> {
            Ok(r#"
flow_id: "test-flow-id"
user_prompt: "swap 1 SOL to USDC"
subject_wallet_info:
  pubkey: "11111111111111111111111111111112"
  lamports: 1000000000
  total_value_usd: 150.0
  tokens:
    - mint: "So11111111111111111111111111111111111111112"
      balance: 1000000000
      decimals: 9
      symbol: "SOL"
steps:
  - step_id: "swap"
    prompt: "swap 1 SOL to USDC"
    context: "Exchange 1 SOL for USDC"
    critical: true
ground_truth:
  final_state_assertions:
    - assertion_type: "SolBalanceChange"
      pubkey: "11111111111111111111111111111112"
      expected_change_gte: -1010000000
  error_tolerance: 0.01
metadata:
  category: "swap"
  complexity_score: 1
  tags: []
  version: "1.0"
created_at: "2023-01-01T00:00:00Z"
"#
            .to_string())
        }
    }

    #[tokio::test]
    async fn test_refine_and_plan_with_llm() {
        let context_resolver = ContextResolver::default();
        let llm_client = Box::new(MockLlmClient);
        let planner = Planner::new(context_resolver).with_llm_client(llm_client);

        // Directly test the LLM client integration without wallet resolution
        // This avoids blocking RPC call and SURFPOOL dependency
        let mut wallet_context = reev_types::flow::WalletContext::new("test-pubkey".to_string());
        wallet_context.sol_balance = 1_000_000_000; // 1 SOL
        wallet_context.total_value_usd = 150.0;

        let mock_client = MockLlmClient;
        let flow = planner
            .generate_flow_with_llm("swap 1 SOL to USDC", &wallet_context, &mock_client)
            .await
            .unwrap();

        assert_eq!(flow.flow_id, "test-flow-id");
        assert_eq!(flow.user_prompt, "swap 1 SOL to USDC");
        assert_eq!(
            flow.subject_wallet_info.pubkey,
            "11111111111111111111111111111112"
        );
        assert_eq!(flow.steps.len(), 1);
        assert_eq!(flow.steps[0].step_id, "swap");
        assert!(flow.ground_truth.is_some());
    }

    #[tokio::test]
    async fn test_parse_swap_intent() {
        let context_resolver = ContextResolver::default();
        let planner = Planner::new(context_resolver);

        let intent = planner.parse_intent("swap 1 SOL to USDC").unwrap();

        match intent {
            UserIntent::Swap {
                from,
                to,
                amount,
                percentage,
            } => {
                assert_eq!(from, "SOL");
                assert_eq!(to, "USDC");
                assert_eq!(amount, 1.0);
                assert!(percentage.is_none());
            }
            _ => panic!("Expected Swap intent"),
        }
    }

    #[tokio::test]
    async fn test_parse_lend_intent() {
        let context_resolver = ContextResolver::default();
        let planner = Planner::new(context_resolver);

        let intent = planner.parse_intent("lend 100 USDC to jupiter").unwrap();

        match intent {
            UserIntent::Lend {
                mint,
                amount,
                percentage,
            } => {
                assert_eq!(mint, "USDC");
                assert_eq!(amount, 100.0);
                assert!(percentage.is_none());
            }
            _ => panic!("Expected Lend intent"),
        }
    }

    #[tokio::test]
    async fn test_parse_swap_then_lend_intent() {
        let context_resolver = ContextResolver::default();
        let planner = Planner::new(context_resolver);

        let intent = planner
            .parse_intent("swap 50% SOL to USDC and lend 50%")
            .unwrap();

        match intent {
            UserIntent::SwapThenLend {
                from,
                to,
                amount,
                percentage,
            } => {
                assert_eq!(from, "SOL");
                assert_eq!(to, "USDC");
                assert_eq!(amount, 50.0);
                assert!(percentage.is_some());
            }
            _ => panic!("Expected SwapThenLend intent"),
        }
    }

    #[tokio::test]
    async fn test_extract_percentage() {
        let context_resolver = ContextResolver::default();
        let planner = Planner::new(context_resolver);

        assert_eq!(
            planner.extract_percentage("swap 50% SOL to USDC"),
            Some(50.0)
        );
        assert_eq!(
            planner.extract_percentage("swap 25.5% SOL to USDC"),
            Some(25.5)
        );
        assert_eq!(planner.extract_percentage("swap 1 SOL to USDC"), None);
    }

    #[tokio::test]
    async fn test_token_to_mint() {
        let context_resolver = ContextResolver::default();
        let planner = Planner::new(context_resolver);

        assert_eq!(
            planner.token_to_mint("SOL").unwrap(),
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(
            planner.token_to_mint("USDC").unwrap(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
        assert_eq!(
            planner.token_to_mint("USDT").unwrap(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
        );

        assert!(planner.token_to_mint("UNKNOWN").is_err());
    }
}
