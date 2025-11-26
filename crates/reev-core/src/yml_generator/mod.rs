//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the V3 plan YML generation component in Phase 1.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows
//! with appropriate expected_tools hints for the rig agent. This implementation
//! follows the V3 plan where RigAgent handles tool selection based on refined prompts.

mod flow_templates;
// operation_parser module has been completely removed in V3 architecture
mod step_builders;
mod unified_flow_builder;

// operation_parser exports removed - module has been completely deleted
pub use unified_flow_builder::UnifiedFlowBuilder;

use anyhow::Result;
use reev_types::flow::WalletContext;
use tracing::{info, instrument};

use crate::refiner::RefinedPrompt;
use crate::yml_schema::YmlFlow;

/// YML generator for creating structured flows from refined prompts
///
/// This implementation follows the V3 plan with a simplified YmlGenerator
/// that generates flows directly without operation parsing.
pub struct YmlGenerator {
    // Stateless in V3 architecture - no fields needed
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self {} // Stateless in V3 architecture
    }

    /// Create a YML generator with custom error tolerance
    pub fn with_error_tolerance(_error_tolerance: f64) -> Self {
        // In V3, error tolerance is handled at the ground truth level
        // This method is kept for backward compatibility
        Self::new()
    }

    /// Generate a YML flow from a refined prompt and wallet context
    #[instrument(skip(self, refined_prompt, wallet_context))]
    pub async fn generate_flow(
        &self,
        refined_prompt: &RefinedPrompt,
        wallet_context: &WalletContext,
    ) -> Result<YmlFlow> {
        info!(
            "Generating YML flow from refined prompt: {}",
            refined_prompt.refined
        );

        // Create a simple flow with a single step containing the refined prompt
        // According to V3 plan, RigAgent should determine tools and parameters
        let flow_id = uuid::Uuid::new_v4().to_string();

        // Create wallet info from context
        let wallet_info = crate::yml_schema::YmlWalletInfo::new(
            wallet_context.owner.clone(),
            wallet_context.sol_balance,
        )
        .with_total_value(wallet_context.total_value_usd);

        // Add tokens to wallet info
        let mut final_wallet_info = wallet_info;
        for token in wallet_context.token_balances.values() {
            final_wallet_info = final_wallet_info.with_token(token.clone());
        }

        // Create a simple step with the refined prompt
        let step = crate::yml_schema::YmlStep::new(
            uuid::Uuid::new_v4().to_string(),
            refined_prompt.refined.clone(),
            format!("Executing: {}", refined_prompt.original),
        )
        .with_refined_prompt(refined_prompt.refined.clone());

        // Create the flow
        let flow = crate::yml_schema::YmlFlow::new(
            flow_id,
            refined_prompt.original.clone(),
            final_wallet_info,
        )
        .with_step(step)
        .with_refined_prompt(refined_prompt.refined.clone());

        info!("Generated YML flow with ID: {}", flow.flow_id);
        Ok(flow)
    }
}
