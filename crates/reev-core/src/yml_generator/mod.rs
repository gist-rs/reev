//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the V3 plan YML generation component in Phase 1.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows
//! with appropriate expected_tools hints for the rig agent. This implementation
//! follows the V3 plan's dynamic operation parsing and composable step builders.

mod flow_templates;
pub mod operation_parser;
mod step_builders;
mod unified_flow_builder;

pub use operation_parser::{FlowTemplate, Operation};
pub use unified_flow_builder::UnifiedFlowBuilder;

use anyhow::Result;
use reev_types::flow::WalletContext;
use tracing::{info, instrument};

use crate::refiner::RefinedPrompt;
use crate::yml_schema::YmlFlow;

/// YML generator for creating structured flows from refined prompts
///
/// This implementation follows the V3 plan by using the UnifiedFlowBuilder
/// which supports dynamic operation parsing and composable step builders.
pub struct YmlGenerator {
    /// Unified flow builder for dynamic operation sequences
    unified_flow_builder: UnifiedFlowBuilder,
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self {
            unified_flow_builder: UnifiedFlowBuilder::new(),
        }
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

        // Use the unified flow builder to handle any sequence of operations
        // This replaces the fixed operation type matching from the previous implementation
        let flow = self
            .unified_flow_builder
            .build_flow_from_operations(refined_prompt, wallet_context)
            .await?;

        info!("Generated YML flow with ID: {}", flow.flow_id);
        Ok(flow)
    }
}
