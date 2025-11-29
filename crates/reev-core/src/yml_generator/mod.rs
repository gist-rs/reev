//! YML Generator for Phase 1 of V3 Plan
//!
//! This module implements the V3 plan YML generation component in Phase 1.
//! It uses refined prompts from the LanguageRefiner to generate structured YML flows
//! with appropriate expected_tools hints for the rig agent. This implementation
//! follows the V3 plan where RigAgent handles tool selection based on refined prompts.

use reev_types::tools::ToolName;
use std::sync::Arc;
use tracing::{debug, info};

mod flow_templates;
// operation_parser module has been completely removed in V3 architecture
mod step_builders;
mod unified_flow_builder;

// operation_parser exports removed - module has been completely deleted
pub use unified_flow_builder::UnifiedFlowBuilder;

use anyhow::Result;
use reev_types::flow::WalletContext;
use tracing::instrument;

use crate::refiner::RefinedPrompt;
use crate::yml_schema::YmlFlow;

/// YML generator for creating structured flows from refined prompts
///
/// This implementation follows the V3 plan with LLM-based operation parsing
/// and tool determination for proper multi-step flow generation.
pub struct YmlGenerator {
    /// LLM client for operation extraction and tool determination
    llm_client: Option<Arc<dyn crate::planner::LlmClient>>,
}

impl Default for YmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl YmlGenerator {
    /// Create a new YML generator
    pub fn new() -> Self {
        Self { llm_client: None }
    }

    /// Create a new YML generator with LLM client
    pub fn with_llm_client(llm_client: Arc<dyn crate::planner::LlmClient>) -> Self {
        Self {
            llm_client: Some(llm_client),
        }
    }

    /// Get a reference to the LLM client
    pub fn llm_client(&self) -> &Option<Arc<dyn crate::planner::LlmClient>> {
        &self.llm_client
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

        // Parse the refined prompt to extract individual operations using LLM
        debug!(
            "DEBUG: generate_flow called, LLM client available: {}",
            self.llm_client.is_some()
        );
        debug!(
            "DEBUG: generate_flow called, LLM client available: {}",
            self.llm_client.is_some()
        );
        debug!(
            "DEBUG: Refined prompt before extraction: {}",
            refined_prompt.refined
        );

        let operations =
            extract_operations_from_prompt(&refined_prompt.refined, &self.llm_client).await;
        debug!("DEBUG: Operations extracted: {:?}", operations);
        debug!("DEBUG: Number of operations: {}", operations.len());

        // If no operations found, create a single step with the refined prompt
        let steps = if operations.is_empty() {
            info!(
                "DEBUG: Determining tools for single step with prompt: {}",
                refined_prompt.refined
            );
            let expected_tools = {
                determine_expected_tools(&refined_prompt.refined, &self.llm_client)
                    .await
                    .unwrap_or_default()
            };
            info!("DEBUG: Determined tools: {:?}", expected_tools);

            vec![crate::yml_schema::YmlStep::new(
                uuid::Uuid::new_v4().to_string(),
                refined_prompt.refined.clone(),
                format!("Executing: {}", refined_prompt.original),
            )
            .with_refined_prompt(refined_prompt.refined.clone())
            .with_expected_tools(expected_tools)]
        } else {
            // Create a separate step for each operation using async block
            let mut steps = Vec::new();
            for (i, operation) in operations.into_iter().enumerate() {
                let step_id = uuid::Uuid::new_v4().to_string();
                let expected_tools = {
                    determine_expected_tools(&operation, &self.llm_client)
                        .await
                        .unwrap_or_default()
                };

                let step = crate::yml_schema::YmlStep::new(
                    step_id,
                    operation.clone(),
                    format!("Step {}: {}", i + 1, operation),
                )
                .with_refined_prompt(operation)
                .with_expected_tools(expected_tools);

                steps.push(step);
            }
            steps
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

/// Determine expected tools using pattern matching instead of LLM
/// This is a fallback implementation since GLM client is not designed for tool determination
pub async fn determine_expected_tools(
    refined_prompt: &str,
    _llm_client: &Option<Arc<dyn crate::planner::LlmClient>>,
) -> Option<Vec<ToolName>> {
    debug!(
        "DEBUG: determine_expected_tools called with: {}",
        refined_prompt
    );

    // Convert to lowercase for easier pattern matching
    let prompt_lower = refined_prompt.to_lowercase();
    let mut tools = Vec::new();

    // Check for swap operation
    if prompt_lower.contains("swap") {
        tools.push(ToolName::JupiterSwap);
    }

    // Check for lend operation
    if prompt_lower.contains("lend") {
        tools.push(ToolName::GetJupiterLendEarnPosition);
    }

    // Check for transfer operation (SOL)
    if prompt_lower.contains("transfer") || prompt_lower.contains("send") {
        if prompt_lower.contains("sol") {
            tools.push(ToolName::SolTransfer);
        } else {
            tools.push(ToolName::SplTransfer);
        }
    }

    debug!("DEBUG: Determined tools: {:?}", tools);
    if tools.is_empty() {
        None
    } else {
        Some(tools)
    }
}

/// Extract individual operations from a multi-step prompt using pattern matching
/// This is a fallback implementation since the GLM client is not designed for operation extraction
pub async fn extract_operations_from_prompt(
    refined_prompt: &str,
    _llm_client: &Option<Arc<dyn crate::planner::LlmClient>>,
) -> Vec<String> {
    info!(
        "DEBUG: Using pattern-based operation extraction for prompt: {}",
        refined_prompt
    );

    // Check if refined_prompt is JSON format and extract it if needed
    let (extracted_prompt, is_json) = if refined_prompt.trim().starts_with('{') {
        // Try to parse as JSON to extract refined_prompt field
        match serde_json::from_str::<serde_json::Value>(refined_prompt) {
            Ok(json) => {
                if let Some(prompt) = json.get("refined_prompt").and_then(|v| v.as_str()) {
                    (prompt.to_string(), true)
                } else {
                    (refined_prompt.to_string(), false)
                }
            }
            Err(_) => {
                // Not valid JSON, use as-is
                (refined_prompt.to_string(), false)
            }
        }
    } else {
        // Not JSON, use as-is
        (refined_prompt.to_string(), false)
    };

    // Use extracted prompt for operation extraction
    let prompt_for_extraction = if is_json {
        info!(
            "DEBUG: Refined prompt is JSON, extracted: {}",
            extracted_prompt
        );
        extracted_prompt.clone()
    } else {
        info!("DEBUG: Refined prompt is plain text: {}", extracted_prompt);
        extracted_prompt.to_string()
    };

    // Convert to lowercase for easier pattern matching
    let prompt_lower = prompt_for_extraction.to_lowercase();

    // Check for multi-step patterns
    let mut operations = Vec::new();

    // Pattern 1: "swap X to Y then lend Z"
    if prompt_lower.contains("swap")
        && prompt_lower.contains("then")
        && prompt_lower.contains("lend")
        && !prompt_lower.contains("and then")
    // Avoid matching Pattern 2
    {
        // Extract swap operation
        if let Some(swap_start) = prompt_lower.find("swap") {
            if let Some(_then_pos) = prompt_lower.find("then") {
                // For "then", we need to find the actual position in the original prompt
                let original_then_pos = refined_prompt.to_lowercase().find("then").unwrap();

                let swap_text = refined_prompt[swap_start..original_then_pos].trim();
                operations.push(swap_text.to_string());

                // Extract lend operation and clean it up
                let mut lend_text = refined_prompt[original_then_pos + 4..].trim().to_string();

                // Remove trailing sentences about multi-step processes
                if let Some(dot_pos) = lend_text.find('.') {
                    lend_text = lend_text[..dot_pos].to_string();
                }
                operations.push(lend_text.trim().to_string());
            }
        }
    }
    // Pattern 2: "swap X to Y and then lend Z"
    else if prompt_lower.contains("swap")
        && prompt_lower.contains("and then")
        && prompt_lower.contains("lend")
    {
        // Extract swap operation
        if let Some(swap_start) = prompt_lower.find("swap") {
            if let Some(_and_then_pos) = prompt_lower.find("and then") {
                // For "and then", we need to find the actual position in the original prompt
                let original_and_then_pos = refined_prompt.to_lowercase().find("and then").unwrap();

                let swap_text = refined_prompt[swap_start..original_and_then_pos].trim();
                operations.push(swap_text.to_string());

                // Extract lend operation and clean it up
                let mut lend_text = refined_prompt[original_and_then_pos + 8..]
                    .trim()
                    .to_string();

                // Remove trailing sentences about multi-step processes
                if let Some(dot_pos) = lend_text.find('.') {
                    lend_text = lend_text[..dot_pos].to_string();
                }
                operations.push(lend_text.trim().to_string());
            }
        }
    }
    // Pattern 3: "swap X to Y and lend Z"
    else if prompt_lower.contains("swap")
        && prompt_lower.contains("and")
        && prompt_lower.contains("lend")
        && !prompt_lower.contains("and then")
    // Avoid matching Pattern 2
    {
        // Extract swap operation
        if let Some(swap_start) = prompt_lower.find("swap") {
            if let Some(_and_pos) = prompt_lower.find(" and ") {
                // For "and", we need to find the actual position in the original prompt
                let original_and_pos = refined_prompt.to_lowercase().find(" and ").unwrap();

                let swap_text = refined_prompt[swap_start..original_and_pos].trim();
                operations.push(swap_text.to_string());

                // Extract lend operation and clean it up
                let mut lend_text = refined_prompt[original_and_pos + 5..].trim().to_string();

                // Remove trailing sentences about multi-step processes
                if let Some(dot_pos) = lend_text.find('.') {
                    lend_text = lend_text[..dot_pos].to_string();
                }
                operations.push(lend_text.trim().to_string());
            }
        }
    }

    // If no multi-step pattern detected, return single operation
    if operations.is_empty() {
        operations.push(prompt_for_extraction);
    }

    info!(
        "DEBUG: Extracted {} operations: {:?}",
        operations.len(),
        operations
    );
    operations
}
