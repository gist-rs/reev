//! Unified Flow Builder for Simple YML Generation
//!
//! This module implements a simplified flow builder that creates YML structures
//! with refined prompts without pre-determining operations. Following V3 plan,
//! RigAgent should handle tool selection based on refined prompts, not a rule-based parser.

use crate::refiner::RefinedPrompt;
// OperationParser removed - replaced by RigAgent in V3 architecture
use crate::yml_schema::YmlFlow;
use anyhow::Result;
use reev_types::flow::WalletContext;
use tracing::{info, instrument};

/// Simplified flow builder for YML generation without operation pre-determination
pub struct UnifiedFlowBuilder {
    // No fields needed - builder stateless in V3 architecture
}

impl Default for UnifiedFlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedFlowBuilder {
    /// Create a new unified flow builder
    pub fn new() -> Self {
        UnifiedFlowBuilder {}
    }

    /// Build a flow from a refined prompt and wallet context
    /// Following V3 plan, this creates a single step with the refined prompt.
    /// RigAgent will handle all operations within the refined prompt.
    #[instrument(skip(self, refined_prompt))]
    pub async fn build_flow_from_operations(
        &self,
        refined_prompt: &RefinedPrompt,
        _wallet_context: &WalletContext,
    ) -> Result<YmlFlow> {
        info!(
            "Building flow from refined prompt: {}",
            refined_prompt.refined
        );

        // Following V3 plan: create a single step with the refined prompt
        // Let RigAgent handle all operations within the refined prompt
        let flow_id = uuid::Uuid::new_v4().to_string();

        // Create a single step with the refined prompt
        let step_id = format!("step_{}", uuid::Uuid::new_v4());
        let step = crate::yml_schema::YmlStep {
            step_id: step_id.clone(),
            prompt: refined_prompt.original.clone(),
            refined_prompt: refined_prompt.refined.clone(),
            context: format!("User request: {}", refined_prompt.original),
            expected_tool_calls: None, // Let RigAgent determine tools
            expected_tools: None,      // Will be determined by RigAgent
            critical: Some(true),
            estimated_time_seconds: Some(30),
        };

        // Create the flow
        let flow = YmlFlow {
            flow_id,
            user_prompt: refined_prompt.original.clone(),
            refined_prompt: refined_prompt.refined.clone(),
            created_at: chrono::Utc::now(),
            subject_wallet_info: crate::yml_schema::YmlWalletInfo::new("test".to_string(), 0),
            steps: vec![step],
            ground_truth: None,
            metadata: crate::yml_schema::FlowMetadata::new(),
        };

        info!("Built flow with 1 step, ID: {}", flow.flow_id);
        Ok(flow)
    }
}
