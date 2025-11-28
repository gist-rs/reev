//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the V3 plan YML generation component in Phase 1.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows
//! with appropriate expected_tools hints for the rig agent. This implementation
//! follows the V3 plan where RigAgent handles tool selection based on refined prompts.

use reev_types::tools::ToolName;

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

        // Create a flow with potentially multiple steps based on the refined prompt
        // Following V3 plan, each operation should be a separate step
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

        // Parse the refined prompt to extract individual operations
        let operations = extract_operations_from_prompt(&refined_prompt.refined);

        // If no operations found, create a single step with the refined prompt
        let steps = if operations.is_empty() {
            let expected_tools =
                determine_expected_tools(&refined_prompt.refined).unwrap_or_default();

            vec![crate::yml_schema::YmlStep::new(
                uuid::Uuid::new_v4().to_string(),
                refined_prompt.refined.clone(),
                format!("Executing: {}", refined_prompt.original),
            )
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_expected_tools(expected_tools)]
        } else {
            // Create a separate step for each operation
            operations
                .into_iter()
                .enumerate()
                .map(|(i, operation)| {
                    let step_id = uuid::Uuid::new_v4().to_string();
                    let expected_tools = determine_expected_tools(&operation).unwrap_or_default();

                    crate::yml_schema::YmlStep::new(
                        step_id,
                        operation.clone(),
                        format!("Step {}: {}", i + 1, operation),
                    )
                    .with_refined_prompt(operation)
                    .with_expected_tools(expected_tools)
                })
                .collect()
        };

        // Create the flow
        let mut flow = crate::yml_schema::YmlFlow::new(
            flow_id,
            refined_prompt.original.clone(),
            final_wallet_info,
        )
        .with_refined_prompt(refined_prompt.refined.clone());

        // Add all steps to the flow
        for step in steps {
            flow = flow.with_step(step);
        }

        info!(
            "Generated YML flow with {} steps, ID: {}",
            flow.steps.len(),
            flow.flow_id
        );
        Ok(flow)
    }
}

/// Determine expected tools based on the refined prompt
fn determine_expected_tools(refined_prompt: &str) -> Option<Vec<ToolName>> {
    let prompt_lower = refined_prompt.to_lowercase();

    // Check for swap operations
    if prompt_lower.contains("swap") {
        return Some(vec![ToolName::JupiterSwap]);
    }

    // Check for lend operations
    if prompt_lower.contains("lend") || prompt_lower.contains("deposit") {
        return Some(vec![ToolName::JupiterLendEarnDeposit]);
    }

    // Check for transfer/send operations
    if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        return Some(vec![ToolName::SolTransfer]);
    }

    // Check for balance operations
    if prompt_lower.contains("balance") || prompt_lower.contains("get") {
        return Some(vec![ToolName::GetAccountBalance]);
    }

    // Default to no expected tools if no pattern matches
    None
}

/// Extract individual operations from a multi-step prompt
fn extract_operations_from_prompt(refined_prompt: &str) -> Vec<String> {
    let prompt_lower = refined_prompt.to_lowercase();
    let mut operations = Vec::new();

    // If this is a multi-step prompt with "then" or "and"
    if prompt_lower.contains(" then ") || prompt_lower.contains(" and ") {
        // Split by "then" first (preferred over "and")
        if prompt_lower.contains(" then ") {
            let parts: Vec<&str> = refined_prompt.split(" then ").collect();

            for part in parts {
                if !part.trim().is_empty() {
                    // Clean up the operation string
                    let mut operation = part.trim().to_string();

                    // Remove any trailing quotes that might have been added
                    if operation.ends_with('"') {
                        operation.pop();
                    }

                    // Check if this is a valid operation (contains action words)
                    if operation.to_lowercase().contains("swap")
                        || operation.to_lowercase().contains("lend")
                        || operation.to_lowercase().contains("transfer")
                        || operation.to_lowercase().contains("send")
                    {
                        operations.push(operation);
                    }
                }
            }
        }
        // Try splitting by "and" if "then" wasn't found
        else if prompt_lower.contains(" and ") {
            let parts: Vec<&str> = refined_prompt.split(" and ").collect();

            for part in parts {
                if !part.trim().is_empty() {
                    // Clean up the operation string
                    let mut operation = part.trim().to_string();

                    // Remove any trailing quotes that might have been added
                    if operation.ends_with('"') {
                        operation.pop();
                    }

                    // Check if this is a valid operation (contains action words)
                    if operation.to_lowercase().contains("swap")
                        || operation.to_lowercase().contains("lend")
                        || operation.to_lowercase().contains("transfer")
                        || operation.to_lowercase().contains("send")
                    {
                        operations.push(operation);
                    }
                }
            }
        }
    }

    // Return the operations if we found multiple valid ones
    if operations.len() > 1 {
        return operations;
    }

    // If no multi-step pattern found, return empty to use single step approach
    Vec::new()
}
