//! Orchestrator Gateway
//!
//! This module handles user prompt refinement and flow planning,
//! acting as the entry point for dynamic flow generation.

use crate::context_resolver::ContextResolver;
use crate::generators::YmlGenerator;
use crate::Result;
use reev_types::flow::{DynamicFlowPlan, WalletContext};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};

/// Orchestrator Gateway for processing user prompts and generating flows
#[derive(Debug)]
pub struct OrchestratorGateway {
    /// Context resolver for wallet and price information
    context_resolver: Arc<ContextResolver>,
    /// YML generator for creating benchmark files
    yml_generator: Arc<YmlGenerator>,
    /// Generated files tracker for cleanup
    generated_files: Arc<RwLock<Vec<NamedTempFile>>>,
}

impl OrchestratorGateway {
    /// Create a new orchestrator gateway
    pub fn new() -> Self {
        Self {
            context_resolver: Arc::new(ContextResolver::new()),
            yml_generator: Arc::new(YmlGenerator::new()),
            generated_files: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Process user request and generate dynamic flow
    #[instrument(skip(self))]
    pub async fn process_user_request(
        &self,
        user_prompt: &str,
        wallet_pubkey: &str,
    ) -> Result<(DynamicFlowPlan, String)> {
        info!("Processing user request: {}", user_prompt);

        // 1. Resolve wallet context
        let context = self
            .context_resolver
            .resolve_wallet_context(wallet_pubkey)
            .await?;
        debug!(
            "Resolved wallet context: {} SOL, ${:.2} total",
            context.sol_balance_sol(),
            context.total_value_usd
        );

        // 2. Refine user prompt with context
        let refined_prompt = self.refine_prompt(user_prompt, &context);
        debug!("Refined prompt: {}", refined_prompt);

        // 3. Generate flow plan (use original prompt, not refined)
        let flow_plan = self.generate_flow_plan(user_prompt, &context)?;
        debug!("Generated flow plan with {} steps", flow_plan.steps.len());

        // 4. Generate YML for bridge mode
        let yml_path = self.yml_generator.generate_yml(&flow_plan).await?;
        info!("Generated YML: {}", yml_path);

        Ok((flow_plan, yml_path))
    }

    /// Refine user prompt with wallet context
    pub fn refine_prompt(&self, prompt: &str, _context: &WalletContext) -> String {
        // Return original prompt unchanged for now - refinement will be done in step generation
        prompt.to_string()
    }

    /// Generate flow plan from refined prompt
    pub fn generate_flow_plan(
        &self,
        prompt: &str,
        context: &WalletContext,
    ) -> Result<DynamicFlowPlan> {
        let flow_id = format!(
            "dynamic-{}-{}",
            chrono::Utc::now().timestamp(),
            uuid::Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        let mut flow = DynamicFlowPlan::new(flow_id.clone(), prompt.to_string(), context.clone());

        // Parse intent and generate steps - more flexible matching
        let prompt_lower = prompt.to_lowercase();
        let has_swap = prompt_lower.contains("swap") || prompt_lower.contains("sol");
        let has_lend = prompt_lower.contains("lend");
        let has_multiply = prompt_lower.contains("multiply") || prompt_lower.contains("1.5x");
        let has_sol_percentage = prompt_lower.contains("%") && prompt_lower.contains("sol");

        // Check for complex flows first
        if has_swap && (has_lend || has_multiply) {
            // Swap then lend flow
            flow = flow
                .with_step(create_swap_step(context, prompt)?)
                .with_step(create_lend_step(context)?);
        } else if has_swap {
            // Single swap flow
            flow = flow.with_step(create_swap_step(context, prompt)?);
        } else if has_lend {
            // Single lend flow
            flow = flow.with_step(create_lend_step(context)?);
        } else if has_sol_percentage {
            // Percentage-based flow - assume swap
            flow = flow.with_step(create_swap_step(context, prompt)?);
        } else {
            return Err(anyhow::anyhow!("Unsupported flow type: {prompt}"));
        }

        Ok(flow)
    }

    /// Clean up generated temporary files
    #[instrument(skip(self))]
    pub async fn cleanup(&self) -> Result<()> {
        let mut files = self.generated_files.write().await;
        for file in files.drain(..) {
            if let Err(e) = file.close() {
                error!("Failed to cleanup temp file: {}", e);
            }
        }
        Ok(())
    }
}

impl Default for OrchestratorGateway {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a swap step based on context
pub fn create_swap_step(
    context: &WalletContext,
    prompt: &str,
) -> Result<reev_types::flow::DynamicStep> {
    let sol_balance = context.sol_balance_sol();
    // Extract amount from prompt or use 50% default
    let prompt_lower = prompt.to_lowercase();
    let swap_amount = if prompt_lower.contains("1 sol") {
        "1".to_string()
    } else if prompt_lower.contains("0.5 sol") {
        "0.5".to_string()
    } else if prompt_lower.contains("25%") {
        (sol_balance * 0.25).to_string()
    } else if prompt_lower.contains("50%") {
        (sol_balance * 0.5).to_string()
    } else if prompt_lower.contains("100%") {
        sol_balance.to_string()
    } else {
        (sol_balance * 0.5).to_string() // Default 50%
    };

    let prompt_template = format!(
        "Swap {} SOL from wallet {} to USDC using Jupiter DEX. \
         Current SOL price: ${:.2}, USDC available for lending at 5-8% APY.",
        swap_amount,
        context.owner,
        context
            .get_token_price("So11111111111111111111111111111111111111112")
            .unwrap_or(150.0)
    );

    Ok(reev_types::flow::DynamicStep::new(
        "swap_1".to_string(),
        prompt_template,
        "Swap SOL to USDC using Jupiter".to_string(),
    )
    .with_tool("sol_tool")
    .with_estimated_time(30))
}

/// Create a lend step based on context
pub fn create_lend_step(_context: &WalletContext) -> Result<reev_types::flow::DynamicStep> {
    let prompt_template =
        "Depositing USDC from the previous swap into Jupiter lending to earn yield. \
         Using the maximum available USDC balance for optimal returns."
            .to_string();

    Ok(reev_types::flow::DynamicStep::new(
        "lend_1".to_string(),
        prompt_template,
        "Deposit USDC into Jupiter lending".to_string(),
    )
    .with_tool("jupiter_earn_tool")
    .with_estimated_time(45))
}
