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
    /// Following V3 plan, this creates a simple flow with the refined prompt
    /// without trying to parse operations - letting RigAgent handle tool selection
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

        // Create a simple flow with a single step containing the refined prompt
        // According to V3 plan, RigAgent should determine the tools and parameters
        let flow_id = uuid::Uuid::new_v4().to_string();
        let step_id = format!("step_{}", uuid::Uuid::new_v4());

        // Create a single step with the refined prompt
        let step = crate::yml_schema::YmlStep {
            step_id: step_id.clone(),
            prompt: refined_prompt.original.clone(),
            refined_prompt: refined_prompt.refined.clone(), // Set's refined prompt
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

        info!("Built flow with ID: {}", flow.flow_id);
        Ok(flow)
    }
}

#[cfg(test)]
mod tests {
    use crate::{refiner::RefinedPrompt, yml_generator::UnifiedFlowBuilder};

    #[tokio::test]
    async fn test_build_flow_preserves_operation_type() {
        // Create a unified flow builder
        let builder = UnifiedFlowBuilder::new();

        // Create a mock wallet context
        let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

        // Create a mock refined prompt for a swap operation
        let refined_prompt = RefinedPrompt::new_for_test(
            "swap 0.1 SOL for USDC".to_string(),
            "swap 0.1 SOL for USDC".to_string(),
            false,
        );

        // Build the flow
        let flow = builder
            .build_flow_from_operations(&refined_prompt, &wallet_context)
            .await
            .unwrap();

        // Verify the flow preserves the operation type
        assert_eq!(flow.user_prompt, "swap 0.1 SOL for USDC");
        assert_eq!(flow.refined_prompt, "swap 0.1 SOL for USDC");
        assert_eq!(flow.steps.len(), 1);

        // Verify step has refined prompt
        let step = &flow.steps[0];
        assert_eq!(step.refined_prompt, "swap 0.1 SOL for USDC");

        // Verify no pre-determined tools (RigAgent should determine these)
        assert_eq!(step.expected_tools, None);
        assert_eq!(step.expected_tool_calls, None);
    }

    #[tokio::test]
    async fn test_build_flow_preserves_transfer_operation() {
        // Create a unified flow builder
        let builder = UnifiedFlowBuilder::new();

        // Create a mock wallet context
        let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

        // Create a mock refined prompt for a transfer operation
        let refined_prompt = RefinedPrompt::new_for_test(
            "send 1 SOL to address123".to_string(),
            "transfer 1 SOL to address123".to_string(),
            true,
        );

        // Build the flow
        let flow = builder
            .build_flow_from_operations(&refined_prompt, &wallet_context)
            .await
            .unwrap();

        // Verify the flow preserves the operation type
        assert_eq!(flow.user_prompt, "send 1 SOL to address123");
        assert_eq!(flow.refined_prompt, "transfer 1 SOL to address123");
        assert_eq!(flow.steps.len(), 1);

        // Verify step has refined prompt
        let step = &flow.steps[0];
        assert_eq!(step.refined_prompt, "transfer 1 SOL to address123");

        // Verify no pre-determined tools (RigAgent should determine these)
        assert_eq!(step.expected_tools, None);
        assert_eq!(step.expected_tool_calls, None);
    }

    #[tokio::test]
    async fn test_build_flow_preserves_lend_operation() {
        // Create a unified flow builder
        let builder = UnifiedFlowBuilder::new();

        // Create a mock wallet context
        let wallet_context = reev_types::flow::WalletContext::new("test_wallet".to_string());

        // Create a mock refined prompt for a lend operation
        let refined_prompt = RefinedPrompt::new_for_test(
            "lend 100 USDC".to_string(),
            "lend 100 USDC".to_string(),
            false,
        );

        // Build the flow
        let flow = builder
            .build_flow_from_operations(&refined_prompt, &wallet_context)
            .await
            .unwrap();

        // Verify the flow preserves the operation type
        assert_eq!(flow.user_prompt, "lend 100 USDC");
        assert_eq!(flow.refined_prompt, "lend 100 USDC");
        assert_eq!(flow.steps.len(), 1);

        // Verify step has refined prompt
        let step = &flow.steps[0];
        assert_eq!(step.refined_prompt, "lend 100 USDC");

        // Verify no pre-determined tools (RigAgent should determine these)
        assert_eq!(step.expected_tools, None);
        assert_eq!(step.expected_tool_calls, None);
    }
}
