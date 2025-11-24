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

        // Generate structured YML flow using LLM or rule-based fallback
        let yml_flow = match &self.llm_client {
            Some(client) => {
                info!("Using LLM client for flow generation");
                self.generate_flow_with_llm(prompt, &wallet_context, client.as_ref())
                    .await?
            }
            None => {
                warn!("No LLM client configured, using rule-based fallback");
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
        _wallet_context: &WalletContext,
        llm_client: &dyn LlmClient,
    ) -> Result<YmlFlow> {
        debug!("Calling LLM for flow generation");

        // Call LLM client to get response
        let llm_response = llm_client.generate_flow(prompt).await?;

        // Parse LLM response (could be JSON or YAML)
        let yml_flow: YmlFlow = if llm_response.starts_with('{') {
            // Try parsing as JSON first
            let json: serde_json::Value = serde_json::from_str(&llm_response)
                .map_err(|e| anyhow!("Failed to parse LLM response as JSON: {e}"))?;

            // Convert JSON to YmlFlow
            // This is a simplified conversion - in a real implementation,
            // we'd need a proper JSON-to-YML conversion
            let flow_id = json
                .get("flow_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let user_prompt = json
                .get("user_prompt")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let subject_wallet_info_json = json
                .get("subject_wallet_info")
                .unwrap_or(&serde_json::Value::Object(Default::default()))
                .clone();

            // Convert JSON values to YmlWalletInfo
            let subject_wallet_info = if subject_wallet_info_json.is_object() {
                let obj = subject_wallet_info_json.as_object().unwrap();
                let pubkey = obj
                    .get("pubkey")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let lamports = obj
                    .get("lamports")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default();
                let total_value_usd = obj
                    .get("total_value_usd")
                    .and_then(|v| v.as_f64())
                    .unwrap_or_default();

                let tokens =
                    if let Some(tokens_array) = obj.get("tokens").and_then(|v| v.as_array()) {
                        tokens_array
                            .iter()
                            .filter_map(|token| {
                                if let Some(token_obj) = token.as_object() {
                                    let mint = token_obj
                                        .get("mint")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string();
                                    let balance = token_obj
                                        .get("balance")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or_default();
                                    Some(reev_types::benchmark::TokenBalance {
                                        mint,
                                        balance,
                                        decimals: Some(0), // Default value
                                        formatted_amount: Some(String::new()), // Default value
                                        owner: Some(String::new()), // Default value
                                        symbol: Some(String::new()), // Default value
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                YmlWalletInfo {
                    pubkey,
                    lamports,
                    tokens,
                    total_value_usd: Some(total_value_usd),
                }
            } else {
                // Fallback for invalid JSON structure
                YmlWalletInfo::new("USER_WALLET_PUBKEY".to_string(), 0)
            };

            let empty_steps: Vec<serde_json::Value> = Vec::new();
            let steps_json = json
                .get("steps")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_steps);

            let steps = steps_json
                .iter()
                .filter_map(|step| {
                    if let Some(step_obj) = step.as_object() {
                        let step_id = step_obj
                            .get("step_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let prompt = step_obj
                            .get("prompt")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let context = step_obj
                            .get("context")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let critical = step_obj
                            .get("critical")
                            .and_then(|v| v.as_bool())
                            .unwrap_or_default();
                        let estimated_time_seconds = step_obj
                            .get("estimated_time_seconds")
                            .and_then(|v| v.as_u64())
                            .unwrap_or_default();
                        let expected_tool_calls = if let Some(calls_array) = step_obj
                            .get("expected_tool_calls")
                            .and_then(|v| v.as_array())
                        {
                            calls_array
                                .iter()
                                .filter_map(|call| {
                                    if let Some(call_obj) = call.as_object() {
                                        let tool_name_str = call_obj
                                            .get("tool_name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or_default();

                                        // Convert string to enum
                                        let tool_name = match tool_name_str {
                                            "JupiterSwap" => {
                                                reev_types::tools::ToolName::JupiterSwap
                                            }
                                            "JupiterLendEarnDeposit" => {
                                                reev_types::tools::ToolName::JupiterLendEarnDeposit
                                            }
                                            "GetAccountBalance" => {
                                                reev_types::tools::ToolName::GetAccountBalance
                                            }
                                            _ => reev_types::tools::ToolName::GetAccountBalance, // Fallback
                                        };

                                        let critical = call_obj
                                            .get("critical")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or_default();

                                        Some(crate::yml_schema::YmlToolCall {
                                            tool_name,
                                            critical,
                                            expected_parameters: None,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        } else {
                            vec![]
                        };

                        Some(crate::yml_schema::YmlStep {
                            step_id,
                            prompt,
                            context,
                            critical: Some(critical),
                            estimated_time_seconds: Some(estimated_time_seconds),
                            expected_tool_calls: Some(expected_tool_calls),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            let empty_truth: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let ground_truth_json = json
                .get("ground_truth")
                .and_then(|v| v.as_object())
                .unwrap_or(&empty_truth);

            let ground_truth = if let Some(assertions_array) = ground_truth_json
                .get("final_state_assertions")
                .and_then(|v| v.as_array())
            {
                let assertions = assertions_array
                    .iter()
                    .filter_map(|assertion| {
                        if let Some(assertion_obj) = assertion.as_object() {
                            let assertion_type = assertion_obj
                                .get("assertion_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let pubkey = assertion_obj
                                .get("pubkey")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let expected_change_gte = assertion_obj
                                .get("expected_change_gte")
                                .and_then(|v| v.as_i64())
                                .unwrap_or_default();
                            let _error_tolerance = assertion_obj
                                .get("error_tolerance")
                                .and_then(|v| v.as_f64())
                                .unwrap_or_default();

                            Some(crate::yml_schema::YmlAssertion {
                                assertion_type,
                                pubkey: Some(pubkey),
                                expected_change_gte: Some(expected_change_gte as f64),
                                expected_change_lte: None,
                                parameters: None,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                let expected_tool_calls = if let Some(calls_array) = ground_truth_json
                    .get("expected_tool_calls")
                    .and_then(|v| v.as_array())
                {
                    calls_array
                        .iter()
                        .filter_map(|call| {
                            if let Some(call_obj) = call.as_object() {
                                let tool_name_str = call_obj
                                    .get("tool_name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();

                                // Convert string to enum
                                let tool_name = match tool_name_str {
                                    "JupiterSwap" => reev_types::tools::ToolName::JupiterSwap,
                                    "JupiterLendEarnDeposit" => {
                                        reev_types::tools::ToolName::JupiterLendEarnDeposit
                                    }
                                    "GetAccountBalance" => {
                                        reev_types::tools::ToolName::GetAccountBalance
                                    }
                                    _ => reev_types::tools::ToolName::GetAccountBalance, // Fallback
                                };

                                let critical = call_obj
                                    .get("critical")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or_default();

                                Some(crate::yml_schema::YmlToolCall {
                                    tool_name,
                                    critical,
                                    expected_parameters: None,
                                })
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    vec![]
                };

                Some(YmlGroundTruth {
                    final_state_assertions: assertions,
                    expected_tool_calls: Some(expected_tool_calls),
                    error_tolerance: Some(0.01),
                })
            } else {
                None
            };

            // Convert JSON to metadata
            let metadata = json.get("metadata").and_then(|v| v.as_object()).map(|obj| {
                let version = obj
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let _created_at = obj
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let category = obj
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let complexity_score = obj
                    .get("complexity_score")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default() as u8;
                let tags = if let Some(tags_array) = obj.get("tags").and_then(|v| v.as_array()) {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str())
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![]
                };

                crate::yml_schema::FlowMetadata {
                    version,
                    category,
                    complexity_score,
                    tags,
                }
            });

            YmlFlow {
                flow_id,
                user_prompt,
                subject_wallet_info,
                steps,
                ground_truth,
                metadata: metadata.unwrap_or_default(),
                created_at: chrono::Utc::now(),
            }
        } else {
            // Try parsing as YAML
            serde_yaml::from_str(&llm_response)
                .map_err(|e| anyhow!("Failed to parse LLM response as YAML: {e}"))?
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
    Unknown,
}

/// LLM client trait for generating flows
/// Trait for LLM client abstraction
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_flow(&self, prompt: &str) -> Result<String>;
}
