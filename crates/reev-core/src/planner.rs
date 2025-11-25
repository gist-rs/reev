//! Planner for Phase 1 LLM Integration
//!
//! This module implements the Phase 1 LLM integration for structured YML generation
//! from user prompts. It handles language refinement, intent analysis, and creates
//! structured YML flows with wallet context and steps.

use crate::context::ContextResolver;
use crate::llm::glm_client::init_glm_client;

use crate::yml_schema::{
    YmlAssertion, YmlFlow, YmlGroundTruth, YmlStep, YmlToolCall, YmlWalletInfo,
};
use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;
use reev_types::tools::ToolName;
use serde_json;
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

    /// Create a new planner with GLM client initialized
    pub fn new_with_glm(context_resolver: ContextResolver) -> Result<Self> {
        let llm_client = init_glm_client()?;

        Ok(Self {
            context_resolver,
            llm_client: Some(llm_client),
        })
    }

    /// Set the LLM client
    pub fn with_llm_client(mut self, client: Box<dyn LlmClient>) -> Self {
        self.llm_client = Some(client);

        self
    }

    /// Create a planner for testing with mock LLM client
    #[cfg(test)]
    pub fn new_for_test(context_resolver: ContextResolver) -> Self {
        // Mock LLM implementation for testing
        #[cfg(test)]
        struct MockLLMClient;

        #[cfg(test)]
        #[async_trait::async_trait]
        impl LlmClient for MockLLMClient {
            async fn generate_flow(&self, prompt: &str) -> anyhow::Result<String> {
                // Generate a simple mock YML flow based on the prompt
                let flow_id = uuid::Uuid::new_v4();
                let yml_response = format!(
                    r#"flow_id: {flow_id}
user_prompt: "{prompt}"
subject_wallet_info:
  pubkey: "USER_WALLET_PUBKEY"
  lamports: 4000000000
  tokens: []
  total_value_usd: 100.0
steps:
  - step_id: "1"
    prompt: "{prompt}"
    context: "Test context"
    critical: true
    estimated_time_seconds: 5
ground_truth:
  final_state_assertions: []
  expected_tool_calls: []"#
                );
                Ok(yml_response)
            }
        }

        let llm_client: Option<Box<dyn LlmClient>> = Some(Box::new(MockLLMClient));

        Self {
            context_resolver,
            llm_client,
        }
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

        // Check if we should force rule-based generation (for testing)
        let use_rule_based = std::env::var("REEV_USE_RULE_BASED").is_ok();

        // Generate structured YML flow using LLM or rule-based fallback
        let yml_flow = if use_rule_based || self.llm_client.is_none() {
            if use_rule_based {
                info!("Using rule-based flow generation (forced by REEV_USE_RULE_BASED)");
            } else {
                warn!("No LLM client configured, using rule-based fallback");
            }
            self.generate_flow_rule_based(prompt, &wallet_context, &mappings)
                .await?
        } else {
            info!("Using LLM client for flow generation");
            self.generate_flow_with_llm(
                prompt,
                &wallet_context,
                self.llm_client.as_ref().unwrap().as_ref(),
            )
            .await?
        };

        debug!("Generated YML flow: {}", yml_flow.flow_id);
        Ok(yml_flow)
    }

    /// Generate flow using LLM for intent extraction
    pub async fn generate_flow_with_llm(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        llm_client: &dyn LlmClient,
    ) -> Result<YmlFlow> {
        debug!("Calling LLM for intent extraction");

        // Call LLM client to get response with intent and parameters
        let llm_response = llm_client.generate_flow(prompt).await?;

        // Parse LLM response as JSON with intent and parameters
        let json: serde_json::Value = serde_json::from_str(&llm_response)
            .map_err(|e| anyhow!("Failed to parse LLM response as JSON: {e}"))?;

        // Extract intent from the response
        let intent_str = json
            .get("intent")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Extract parameters
        let params = json
            .get("parameters")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let from_token = params
            .get("from_token")
            .and_then(|v| v.as_str())
            .unwrap_or("SOL")
            .to_uppercase();

        let to_token = params
            .get("to_token")
            .and_then(|v| v.as_str())
            .unwrap_or("USDC")
            .to_uppercase();

        // Handle percentage parameter when amount is null or "all"
        let amount = if let Some("null") = params.get("amount").and_then(|v| v.as_str()) {
            // Amount is null, check for percentage or use all SOL
            if let Some(percentage_str) = params.get("percentage").and_then(|v| v.as_str()) {
                // Convert percentage to amount based on wallet context
                let percentage = percentage_str
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .unwrap_or(100.0)
                    / 100.0;
                wallet_context.sol_balance as f64 * percentage
            } else {
                // No percentage specified, use all SOL
                wallet_context.sol_balance as f64
            }
        } else if let Some("all") = params.get("amount").and_then(|v| v.as_str()) {
            // Explicit "all" specified
            wallet_context.sol_balance as f64
        } else if let Some(percentage_str) = params.get("percentage").and_then(|v| v.as_str()) {
            // Percentage specified directly
            let percentage = percentage_str
                .trim_end_matches('%')
                .parse::<f64>()
                .unwrap_or(100.0)
                / 100.0;
            wallet_context.sol_balance as f64 * percentage
        } else {
            // Parse amount directly
            let amount_str = params
                .get("amount")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0");
            amount_str.parse::<f64>().unwrap_or(1.0)
        };

        // Check if the prompt contains "all" and we're dealing with SOL
        let is_sell_all = prompt.to_lowercase().contains("all")
            && prompt.to_lowercase().contains("sol")
            && (amount == 0.0 || from_token == "SOL");

        // If we're in a "sell all SOL" scenario but amount is 0, use wallet balance
        let effective_amount = if is_sell_all && amount == 0.0 {
            wallet_context.sol_balance as f64
        } else {
            amount
        };

        let percentage_str = params.get("percentage").and_then(|v| v.as_str());

        let _percentage: Option<f64> = percentage_str.and_then(|s| s.parse().ok());

        // Convert amount from lamports to SOL for display
        let amount_sol = if from_token == "SOL" {
            // Account for gas reserve when calculating display amount
            let gas_reserve_lamports = 50_000_000u64; // 0.05 SOL
            let amount_u64 = effective_amount as u64;

            // Ensure amount is greater than 0
            if amount_u64 == 0 {
                // Default to 1 SOL if amount is 0 (gas reserve will be handled later)
                1.0
            } else {
                let display_amount = if amount_u64 > gas_reserve_lamports {
                    (amount_u64 - gas_reserve_lamports) as f64
                } else {
                    effective_amount / 2.0
                };
                display_amount / 1_000_000_000.0
            }
        } else {
            effective_amount
        };

        // Generate a proper UUID for the flow
        let flow_id = uuid::Uuid::now_v7().to_string();

        // Create wallet info programmatically
        let mut wallet_info = crate::yml_schema::YmlWalletInfo::new(
            wallet_context.owner.clone(),
            wallet_context.sol_balance,
        )
        .with_total_value(wallet_context.total_value_usd);

        // Add each token balance to the wallet info
        for token in wallet_context.token_balances.values() {
            wallet_info = wallet_info.with_token(token.clone());
        }

        // Create a appropriate flow based on intent
        let yml_flow = match intent_str.as_str() {
            "swap" => {
                // Create a swap flow
                let _from_mint = self.token_to_mint(&from_token);
                let _to_mint = self.token_to_mint(&to_token);

                let step = crate::yml_schema::YmlStep::new(
                    "swap".to_string(),
                    format!("swap {amount_sol:.1} {from_token} to {to_token}"),
                    format!("Exchange {amount_sol:.1} {from_token} for {to_token}"),
                )
                .with_tool_call(crate::yml_schema::YmlToolCall::new(
                    reev_types::tools::ToolName::JupiterSwap,
                    true,
                ));

                let ground_truth = crate::yml_schema::YmlGroundTruth::new()
                    .with_assertion(
                        crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
                            .with_pubkey(wallet_context.owner.clone())
                            .with_expected_change_gte(
                                -(amount_sol * 1_000_000_000.0 + 50_000_000.0 + 10_000_000.0),
                            ),
                    ) // Account for swap amount + gas reserve + transaction fees
                    .with_tool_call(crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::JupiterSwap,
                        true,
                    ));

                crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
                    .with_step(step)
                    .with_ground_truth(ground_truth)
            }
            "lend" => {
                // Create a lend flow
                let _mint = self.token_to_mint(&from_token);

                let step = crate::yml_schema::YmlStep::new(
                    "lend".to_string(),
                    format!("lend {amount} {from_token} to jupiter"),
                    format!("Lend {amount} {from_token} for yield"),
                )
                .with_tool_call(crate::yml_schema::YmlToolCall::new(
                    reev_types::tools::ToolName::JupiterLendEarnDeposit,
                    true,
                ));

                let ground_truth = crate::yml_schema::YmlGroundTruth::new().with_tool_call(
                    crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::JupiterLendEarnDeposit,
                        true,
                    ),
                );

                crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
                    .with_step(step)
                    .with_ground_truth(ground_truth)
            }
            "swap_then_lend" => {
                // Create a swap then lend flow
                let _from_mint = self.token_to_mint(&from_token);
                let _to_mint = self.token_to_mint(&to_token);

                let step1 = crate::yml_schema::YmlStep::new(
                    "swap".to_string(),
                    format!("swap {amount} {from_token} to {to_token}"),
                    format!("Exchange {amount} {from_token} for {to_token}"),
                )
                .with_tool_call(crate::yml_schema::YmlToolCall::new(
                    reev_types::tools::ToolName::JupiterSwap,
                    true,
                ));

                let step2 = crate::yml_schema::YmlStep::new(
                    "lend".to_string(),
                    format!("lend SWAPPED_AMOUNT {to_token} to jupiter"),
                    format!("Lend swapped {to_token} for yield"),
                )
                .with_tool_call(crate::yml_schema::YmlToolCall::new(
                    reev_types::tools::ToolName::JupiterLendEarnDeposit,
                    true,
                ));

                let ground_truth = crate::yml_schema::YmlGroundTruth::new()
                    .with_assertion(
                        crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
                            .with_pubkey(wallet_context.owner.clone())
                            .with_expected_change_gte(-(amount + 0.1) * 1_000_000_000.0),
                    ) // Account for fees
                    .with_tool_call(crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::JupiterSwap,
                        true,
                    ))
                    .with_tool_call(crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::JupiterLendEarnDeposit,
                        true,
                    ));

                crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
                    .with_step(step1)
                    .with_step(step2)
                    .with_ground_truth(ground_truth)
            }
            "transfer" => {
                // Create a transfer flow
                let _mint = self.token_to_mint(&from_token);

                // Extract recipient from parameters
                let recipient = params
                    .get("recipient")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let step = crate::yml_schema::YmlStep::new(
                    "transfer".to_string(),
                    format!("transfer {amount} {from_token} to {recipient}"),
                    format!("Transfer {amount} {from_token} to specified recipient {recipient}"),
                )
                .with_tool_call(crate::yml_schema::YmlToolCall::new(
                    reev_types::tools::ToolName::SolTransfer,
                    true,
                ));

                let ground_truth = crate::yml_schema::YmlGroundTruth::new()
                    .with_assertion(
                        crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
                            .with_pubkey(wallet_context.owner.clone())
                            .with_expected_change_lte(-(amount + 0.1) * 1_000_000_000.0), // Account for fees
                    )
                    .with_tool_call(crate::yml_schema::YmlToolCall::new(
                        reev_types::tools::ToolName::SolTransfer,
                        true,
                    ));

                crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
                    .with_step(step)
                    .with_ground_truth(ground_truth)
            }
            _ => {
                // Default to a simple flow for unknown intent
                let step = crate::yml_schema::YmlStep::new(
                    "unknown".to_string(),
                    format!("Process user request: {prompt}"),
                    "Processing user request with appropriate tools".to_string(),
                );

                crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
                    .with_step(step)
            }
        };

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
            UserIntent::Transfer {
                mint,
                to,
                amount,
                percentage,
            } => {
                self.create_transfer_flow(prompt, wallet_context, &mint, &to, amount, percentage)
                    .await
            }
            UserIntent::Unknown => Err(anyhow!("Unable to determine intent from prompt: {prompt}")),
        }
    }

    /// Build structured prompt for LLM
    #[allow(dead_code)]
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
    pub fn parse_intent(&self, prompt: &str) -> Result<UserIntent> {
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
        // Handle transfer/send intents
        else if prompt_lower.contains("send") || prompt_lower.contains("transfer") {
            let (mint, to, amount) = self.extract_transfer_params(&prompt_lower)?;
            Ok(UserIntent::Transfer {
                mint,
                to,
                amount,
                percentage,
            })
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
        for token in ["SOL", "USDC", "USDT"] {
            if prompt.contains(&format!("{} ", token.to_lowercase()))
                || prompt.contains(&format!(" {}", token.to_lowercase()))
            {
                from = token.to_string();
                break;
            }
        }

        // Try to extract "to" token
        for token in ["SOL", "USDC", "USDT"] {
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
        for token in ["SOL", "USDC", "USDT"] {
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

    /// Extract transfer parameters from prompt
    fn extract_transfer_params(&self, prompt: &str) -> Result<(String, String, f64)> {
        // Default values
        let mut mint = "SOL".to_string();
        let mut to = "unknown".to_string();
        let mut amount = 1.0;

        // Try to extract token type
        for token in ["SOL", "USDC", "USDT"] {
            if prompt.contains(&format!("{} ", token.to_lowercase()))
                || prompt.contains(&format!(" {}", token.to_lowercase()))
            {
                mint = token.to_string();
                break;
            }
        }

        // Try to extract recipient address
        let pubkey_regex = regex::Regex::new(r"([A-HJ-NP-Za-km-z1-9]{32,44})").unwrap();
        if let Some(captures) = pubkey_regex.captures(prompt) {
            to = captures[1].to_string();
        }

        // Try to extract amount
        if let Some(percentage) = self.extract_percentage(prompt) {
            // Percentage detected
            // We'll use percentage in the flow creation
            return Ok((mint, to, percentage));
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

        Ok((mint, to, amount))
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

    /// Create a transfer flow
    async fn create_transfer_flow(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        mint: &str,
        to: &str,
        amount: f64,
        _percentage: Option<f64>,
    ) -> Result<YmlFlow> {
        let flow_id = Uuid::now_v7().to_string();

        // Convert token to mint address
        let _mint = self.token_to_mint(mint)?;

        // Create wallet info
        let wallet_info =
            YmlWalletInfo::new(wallet_context.owner.clone(), wallet_context.sol_balance)
                .with_total_value(wallet_context.total_value_usd);

        // Create transfer step
        let transfer_step = YmlStep::new(
            "transfer".to_string(),
            format!("transfer {amount} {mint} to {to}"),
            format!("Transfer {amount} {mint} to recipient {to}"),
        )
        .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
        .with_critical(true);

        // Create ground truth
        let ground_truth = YmlGroundTruth::new()
            .with_assertion(
                YmlAssertion::new("SolBalanceChange".to_string())
                    .with_pubkey(wallet_context.owner.clone())
                    .with_expected_change_lte(-((amount * 1_000_000_000.0) + 5_000_000.0)), // Include fees
            )
            .with_tool_call(YmlToolCall::new(ToolName::SolTransfer, true))
            .with_error_tolerance(0.01);

        // Create flow
        let flow = YmlFlow::new(flow_id, prompt.to_string(), wallet_info)
            .with_step(transfer_step)
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
    pub fn token_to_mint(&self, token: &str) -> Result<String> {
        // Simple mapping for common tokens
        match token.to_uppercase().as_str() {
            "SOL" => Ok("So11111111111111111111111111111111111111112".to_string()),
            "USDC" => Ok("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            "USDT" => Ok("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
            _ => Err(anyhow!("Unknown token: {token}")),
        }
    }

    /// Extract percentage from prompt
    pub fn extract_percentage(&self, prompt: &str) -> Option<f64> {
        let regex = regex::Regex::new(r"(\d+\.?\d*)%").unwrap();
        regex
            .captures(prompt)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    }
}

/// User intent extracted from prompt
#[derive(Debug, Clone)]
/// User intent extracted from prompt
pub enum UserIntent {
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
    Transfer {
        mint: String,
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
