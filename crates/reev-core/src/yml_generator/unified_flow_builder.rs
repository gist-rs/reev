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
    /// Following V3 plan, this creates steps for multi-step operations
    /// when detected, otherwise creates a single step
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

        // Check if prompt contains multiple operations
        let prompt_lower = refined_prompt.refined.to_lowercase();
        let is_multi_step = prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ")
            || (prompt_lower.contains("swap") && prompt_lower.contains("lend"))
            || (prompt_lower.contains("swap") && prompt_lower.contains("transfer"))
            || (prompt_lower.contains("lend") && prompt_lower.contains("transfer"));

        info!("DEBUG: is_multi_step = {}", is_multi_step);

        let flow_id = uuid::Uuid::new_v4().to_string();
        let _flow_id = uuid::Uuid::new_v4().to_string();
        // Add assert to verify multi-step detection
        assert!(
            is_multi_step,
            "Prompt '{}' should be detected as multi-step operation",
            refined_prompt.refined
        );

        let steps = if is_multi_step {
            // For multi-step prompts, parse individual operations
            self.parse_multi_step_operations(&refined_prompt.refined)?
        } else {
            // For single-step prompts, create a single step
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
            vec![step]
        };

        // Get the number of steps before creating the flow
        let steps_count = steps.len();

        // Create the flow
        let flow = YmlFlow {
            flow_id,
            user_prompt: refined_prompt.original.clone(),
            refined_prompt: refined_prompt.refined.clone(),
            created_at: chrono::Utc::now(),
            subject_wallet_info: crate::yml_schema::YmlWalletInfo::new("test".to_string(), 0),
            steps,
            ground_truth: None,
            metadata: crate::yml_schema::FlowMetadata::new(),
        };

        info!(
            "Built flow with {} steps, ID: {}",
            steps_count, flow.flow_id
        );
        Ok(flow)
    }

    /// Parse multi-step operations from a refined prompt
    /// For now, this is a simple implementation that splits on "then"
    fn parse_multi_step_operations(
        &self,
        refined_prompt: &str,
    ) -> Result<Vec<crate::yml_schema::YmlStep>> {
        let mut steps = Vec::new();

        // Split on "then" for now - this is a simple implementation
        let parts: Vec<&str> = refined_prompt.split(" then ").collect();

        for (i, part) in parts.iter().enumerate() {
            let step_id = format!("step_{}_{}", i + 1, uuid::Uuid::new_v4());
            let step = crate::yml_schema::YmlStep {
                step_id: step_id.clone(),
                prompt: part.to_string(),
                refined_prompt: part.to_string(),
                context: format!("Step {}: {}", i + 1, part),
                expected_tool_calls: None, // Let RigAgent determine tools
                expected_tools: None,      // Will be determined by RigAgent
                critical: Some(true),
                estimated_time_seconds: Some(30),
            };
            steps.push(step);
        }

        Ok(steps)
    }
}
