//! Planner for Phase 1 LLM Integration
//!
//! This module implements the Phase 1 LLM integration for structured YML generation
//! from user prompts. It handles language refinement, intent analysis, and creates
//! structured YML flows with wallet context and steps.

use crate::context::ContextResolver;
use crate::llm::glm_client::init_glm_client;
use crate::refiner::LanguageRefiner;
use crate::yml_generator::YmlGenerator;
use anyhow::{anyhow, Result};
use reev_types::flow::WalletContext;

use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Planner for Phase 1 LLM integration
pub struct Planner {
    /// Context resolver for wallet information
    context_resolver: ContextResolver,
    /// Language refiner for Phase 1 prompt refinement
    language_refiner: LanguageRefiner,
    /// YML generator for Phase 1 structured YML generation
    yml_generator: YmlGenerator,
    /// LLM client for legacy flow generation (deprecated)
    llm_client: Option<Box<dyn LlmClient>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(context_resolver: ContextResolver) -> Self {
        Self {
            context_resolver,
            language_refiner: LanguageRefiner::new(),
            yml_generator: YmlGenerator::new(),
            llm_client: None,
        }
    }

    /// Create a new planner with GLM client initialized
    pub fn new_with_glm(context_resolver: ContextResolver) -> Result<Self> {
        let llm_client = init_glm_client()?;
        Ok(Self {
            context_resolver,
            language_refiner: LanguageRefiner::new(),
            yml_generator: YmlGenerator::new(),
            llm_client: Some(llm_client),
        })
    }

    /// Set the LLM client
    pub fn with_llm_client(mut self, client: Box<dyn LlmClient>) -> Self {
        self.llm_client = Some(client);
        self
    }

    /// Refine and plan: Phase 1 LLM integration for structured YML generation
    #[instrument(skip(self))]
    pub async fn refine_and_plan(
        &self,
        prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<crate::yml_schema::YmlFlow> {
        info!("Starting Phase 1: Refine and Plan for prompt: {}", prompt);

        // Always use V3 implementation as per PLAN_CORE_V3.md
        return self.refine_and_plan_v3(prompt, wallet_pubkey).await;
    }

    /// V3 implementation of refine_and_plan using LanguageRefiner and YmlGenerator
    async fn refine_and_plan_v3(
        &self,
        prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<crate::yml_schema::YmlFlow> {
        info!(
            "Starting Phase 1 V3: Refine and Plan for prompt: {}",
            prompt
        );

        // Resolve wallet context
        let wallet_context = self
            .context_resolver
            .resolve_wallet_context(wallet_pubkey)
            .await?;
        debug!("Resolved wallet context for {}", wallet_pubkey);

        // Step 1: Language refinement using LLM
        info!("Step 1: Refining language with LLM");
        let refined_prompt = self.language_refiner.refine_prompt(prompt).await?;
        debug!("Refined prompt: {}", refined_prompt.refined);

        if refined_prompt.changes_detected {
            info!("Language refinement applied changes");
        }

        // Step 2: Generate YML structure using rule-based templates
        info!("Step 2: Generating YML structure with rule-based templates");
        let yml_flow = self
            .yml_generator
            .generate_flow(&refined_prompt, &wallet_context)
            .await?;
        debug!("Generated YML flow: {}", yml_flow.flow_id);

        // Phase 1 V3 completed successfully
        Ok(yml_flow)
    }

    /// Generate flow using LLM for intent extraction
    pub async fn generate_flow_with_llm(
        &self,
        prompt: &str,
        wallet_context: &WalletContext,
        llm_client: &dyn LlmClient,
    ) -> Result<crate::yml_schema::YmlFlow> {
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

        // Parse amount from parameters
        let amount_str = params
            .get("amount")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0");

        // Check if amount is "null" or "all"
        let amount = if amount_str == "null" || amount_str == "all" {
            wallet_context.sol_balance as f64
        } else {
            amount_str.parse::<f64>().unwrap_or(1.0)
        };

        // Generate a proper UUID for the flow
        let flow_id = Uuid::now_v7().to_string();

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
            "swap" => create_swap_flow(
                flow_id,
                prompt,
                &wallet_info,
                wallet_context,
                &from_token,
                &to_token,
                amount,
            ),
            "lend" => create_lend_flow(
                flow_id,
                prompt,
                &wallet_info,
                wallet_context,
                &from_token,
                amount,
            ),
            "transfer" => {
                // Extract recipient from parameters
                let recipient = params
                    .get("recipient")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                create_transfer_flow(
                    flow_id,
                    prompt,
                    &wallet_info,
                    wallet_context,
                    &from_token,
                    recipient,
                    amount,
                )
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
}

/// Create a swap flow
fn create_swap_flow(
    flow_id: String,
    prompt: &str,
    wallet_info: &crate::yml_schema::YmlWalletInfo,
    wallet_context: &WalletContext,
    from: &str,
    to: &str,
    amount: f64,
) -> crate::yml_schema::YmlFlow {
    // Convert amount from SOL to display value if needed
    let amount_sol = if from == "SOL" {
        // Account for gas reserve when calculating display amount
        let gas_reserve_lamports = 50_000_000u64; // 0.05 SOL
        let amount_in_lamports = amount * 1_000_000_000.0;
        let display_amount = if amount_in_lamports > gas_reserve_lamports as f64 {
            amount_in_lamports - gas_reserve_lamports as f64
        } else {
            amount_in_lamports / 2.0
        };
        display_amount / 1_000_000_000.0
    } else {
        amount
    };

    // Create swap step
    let step = crate::yml_schema::YmlStep::new(
        "swap".to_string(),
        format!("swap {amount_sol:.1} {from} to {to}"),
        format!("Exchange {amount_sol:.1} {from} for {to}"),
    )
    .with_tool_call(crate::yml_schema::YmlToolCall::new(
        reev_types::tools::ToolName::JupiterSwap,
        true,
    ));

    // Create ground truth
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

    crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info.clone())
        .with_step(step)
        .with_ground_truth(ground_truth)
}

/// Create a lend flow
fn create_lend_flow(
    flow_id: String,
    prompt: &str,
    wallet_info: &crate::yml_schema::YmlWalletInfo,
    _wallet_context: &WalletContext,
    mint: &str,
    amount: f64,
) -> crate::yml_schema::YmlFlow {
    // Create lend step
    let step = crate::yml_schema::YmlStep::new(
        "lend".to_string(),
        format!("lend {amount} {mint} to jupiter"),
        format!("Lend {amount} {mint} for yield"),
    )
    .with_tool_call(crate::yml_schema::YmlToolCall::new(
        reev_types::tools::ToolName::JupiterLendEarnDeposit,
        true,
    ));

    // Create ground truth
    let ground_truth = crate::yml_schema::YmlGroundTruth::new().with_tool_call(
        crate::yml_schema::YmlToolCall::new(
            reev_types::tools::ToolName::JupiterLendEarnDeposit,
            true,
        ),
    );

    crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info.clone())
        .with_step(step)
        .with_ground_truth(ground_truth)
}

/// Create a transfer flow
fn create_transfer_flow(
    flow_id: String,
    prompt: &str,
    wallet_info: &crate::yml_schema::YmlWalletInfo,
    _wallet_context: &WalletContext,
    mint: &str,
    to: &str,
    amount: f64,
) -> crate::yml_schema::YmlFlow {
    // Create transfer step
    let step = crate::yml_schema::YmlStep::new(
        "transfer".to_string(),
        format!("transfer {amount} {mint} to {to}"),
        format!("Transfer {amount} {mint} to recipient {to}"),
    )
    .with_tool_call(crate::yml_schema::YmlToolCall::new(
        reev_types::tools::ToolName::SolTransfer,
        true,
    ));

    // Create ground truth
    let ground_truth = crate::yml_schema::YmlGroundTruth::new()
        .with_assertion(
            crate::yml_schema::YmlAssertion::new("SolBalanceChange".to_string())
                .with_expected_change_lte(-(amount + 0.1) * 1_000_000_000.0), // Account for fees
        )
        .with_tool_call(crate::yml_schema::YmlToolCall::new(
            reev_types::tools::ToolName::SolTransfer,
            true,
        ));

    crate::yml_schema::YmlFlow::new(flow_id, prompt.to_string(), wallet_info.clone())
        .with_step(step)
        .with_ground_truth(ground_truth)
}

/// LLM client trait for generating flows
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_flow(&self, prompt: &str) -> Result<String>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refine_and_plan() {
        // Mock LLM implementation for testing
        // Create a mock LLM client for testing
        #[allow(dead_code)]
        struct MockLLMClient;

        #[async_trait::async_trait]
        impl LlmClient for MockLLMClient {
            async fn generate_flow(&self, prompt: &str) -> Result<String> {
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

        // Create a mock wallet context instead of trying to resolve
        let mut wallet_context = WalletContext::new("test_wallet".to_string());
        wallet_context.sol_balance = 5_000_000_000; // 5 SOL

        let refined_prompt = crate::refiner::RefinedPrompt::new_for_test(
            "test prompt".to_string(),
            "test prompt".to_string(),
            false,
        );

        let yml_generator = crate::yml_generator::YmlGenerator::new();
        let result = yml_generator
            .generate_flow(&refined_prompt, &wallet_context)
            .await;

        assert!(result.is_ok());
    }
}
