//! Rig Agent Integration for Phase 2 Tool Selection
//!
//! This module implements the RigAgent component that wraps rig framework
//! for LLM-driven tool selection and parameter extraction in Phase 2 of
//! Reev Core Architecture.

use anyhow::{anyhow, Result};
use reev_agent::enhanced::common::AgentTools;
use reev_types::flow::{StepResult, WalletContext};
use rig::tool::ToolSet;
use serde_json::json;
use std::collections::HashMap;

use std::string::String;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::yml_schema::YmlStep;

// Import modules
mod context;
mod prompting;
mod tools;
mod types;

// Re-export types and traits
pub use context::ContextProvider;
pub use prompting::{HttpProvider, MultiStepHandler, PromptProvider};
pub use tools::{AgentProvider, AgentToolHelper, ToolExecutor};
pub use types::*;

/// RigAgent for LLM-driven tool selection and parameter extraction
pub struct RigAgent {
    /// Model name for logging
    model_name: String,
    /// API key for the LLM service
    api_key: String,
    /// HTTP client for direct API calls
    http_client: reqwest::Client,
    /// Agent tools for executing blockchain operations
    agent_tools: Option<Arc<AgentTools>>,
}

impl RigAgent {
    /// Create a new RigAgent with the given model and tools
    pub async fn new(api_key: Option<String>, model_name: Option<String>) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: None,
        })
    }

    /// Create a new RigAgent with the given model and tools
    pub async fn new_with_tools(
        api_key: Option<String>,
        model_name: Option<String>,
        agent_tools: Arc<AgentTools>,
    ) -> Result<Self> {
        let model_name = model_name.unwrap_or_else(|| "gpt-4".to_string());
        let api_key = api_key.ok_or_else(|| anyhow!("API key is required for RigAgent"))?;

        // Initialize tool set with Reev tools
        let _tool_set = Self::initialize_tool_set().await?; // Prefix with _ to suppress warning

        Ok(Self {
            model_name,
            api_key,
            http_client: reqwest::Client::new(),
            agent_tools: Some(agent_tools),
        })
    }

    /// Execute a step using the rig agent for tool selection
    #[instrument(skip(self, step, wallet_context))]
    pub async fn execute_step_with_rig(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
    ) -> Result<StepResult> {
        self.execute_step_with_rig_and_history(step, wallet_context, &[])
            .await
    }

    /// Execute a step with rig agent and previous step history
    pub async fn execute_step_with_rig_and_history(
        &self,
        step: &YmlStep,
        wallet_context: &WalletContext,
        previous_results: &[StepResult],
    ) -> Result<StepResult> {
        info!("Executing step {} with rig agent", step.step_id);

        // Debug log to verify the current context before creating the prompt
        debug!(
            "USDC balance in context: {:?}",
            wallet_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Use structured prompt data if available, otherwise use refined prompt or original prompt
        let (prompt, action, parameters) = if let Some(structured_prompt) = &step.structured_prompt
        {
            (
                structured_prompt.refined_prompt.clone(),
                Some(structured_prompt.action.clone()),
                Some(&structured_prompt.parameters),
            )
        } else if !step.refined_prompt.is_empty() {
            (step.refined_prompt.clone(), None, None)
        } else {
            (step.prompt.clone(), None, None)
        };

        // Create YML context and convert to prompt
        let yml_context = self.create_yml_context(step, wallet_context, previous_results)?;
        let context_prompt = self.yml_context_to_prompt(&yml_context, &prompt)?;

        // Log the YML context for debugging
        debug!(
            "Generated YML context for step {}: {:?}",
            step.step_id, yml_context
        );

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            debug!("Using expected tools to guide agent: {:?}", tools);
            self.prompt_with_expected_tools(&context_prompt, &tools)
                .await?
        } else {
            debug!("No expected tools provided, using general agent prompt");
            self.prompt_agent(&context_prompt).await?
        };

        debug!("Got response from agent: {}", response);

        debug!("Parsing tool calls from LLM response");

        // If we have structured data with action and parameters, use them directly
        // Otherwise, extract tool calls from the LLM response
        let tool_calls = if let (Some(action), Some(parameters)) = (action, parameters) {
            debug!(
                "Using structured action and parameters: action={:?}, parameters={:?}",
                action, parameters
            );
            self.create_tool_calls_from_structured_data(&action, parameters)?
        } else {
            debug!("Extracting tool calls from LLM response");
            self.extract_tool_calls(&response)?
        };

        // Check if this is a multi-step prompt and we have multiple operations
        let prompt_lower = prompt.to_lowercase();
        let is_multi_step = prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ");

        debug!("is_multi_step = {}", is_multi_step);
        debug!("Initial tool_calls count = {}", tool_calls.len());

        // For multi-step prompts, we need to ensure we extract all operations
        let tool_calls = if is_multi_step && tool_calls.len() < 2 {
            // Try to extract additional operations if we only got one tool call
            debug!("Multi-step prompt detected but only one tool call extracted, attempting to extract additional operations");
            let additional_calls = self.extract_multi_step_tool_calls(&response, &tool_calls)?;
            debug!("Additional tool_calls = {:?}", additional_calls);
            additional_calls
        } else {
            debug!("Using initial tool_calls as-is");
            tool_calls
        };

        // Execute selected tools
        debug!("Tool calls extracted: {:?}", tool_calls);
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await?;
        debug!("Tool execution results: {:?}", tool_results);

        // Create list of tool names that were executed
        let executed_tool_names: Vec<String> = tool_calls.keys().cloned().collect();

        // Create the step result
        let step_result = StepResult {
            step_id: step.step_id.clone(),
            success: true,
            error_message: None,
            tool_calls: executed_tool_names,
            output: json!({ "tool_results": tool_results }),
            execution_time_ms: 100, // This would be calculated in a real implementation
        };

        Ok(step_result)
    }

    /// Initialize the tool set with Reev tools
    async fn initialize_tool_set() -> Result<ToolSet> {
        // Create a tool set with all Reev tools
        // For now, we'll create a minimal tool set as a placeholder
        // In a full implementation, we would add all Reev tools (SolTransfer, JupiterSwap, etc.)

        // Use the agent builder to create tools directly
        let tool_set = ToolSet::default();

        Ok(tool_set)
    }

    /// Create tool calls directly from structured prompt data
    /// This method implements Phase 4 of the structured LLM response system
    fn create_tool_calls_from_structured_data(
        &self,
        action: &crate::prompt_processor::PromptAction,
        parameters: &crate::prompt_processor::PromptParameters,
    ) -> Result<HashMap<String, serde_json::Value>> {
        debug!(
            "Creating tool calls from structured data: action={:?}",
            action
        );

        let mut tool_calls = HashMap::new();

        match action {
            crate::prompt_processor::PromptAction::Transfer => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    // Select tool based on token type
                    let tool_name =
                        if input_mint == "So11111111111111111111111111111111111111111112" {
                            "sol_transfer".to_string()
                        } else {
                            "spl_transfer".to_string()
                        };

                    tool_calls.insert(
                        tool_name,
                        json!({
                            "amount": amount,
                            "mint": input_mint,
                            "recipient": parameters.additional.get("recipient")
                                .or_else(|| parameters.additional.get("target_pubkey"))
                                .cloned()
                                .unwrap_or(serde_json::Value::Null)
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Swap => {
                if let (Some(amount), Some(input_mint), Some(output_mint)) = (
                    &parameters.amount,
                    &parameters.input_mint,
                    &parameters.output_mint,
                ) {
                    tool_calls.insert(
                        "jupiter_swap".to_string(),
                        json!({
                            "input_amount": amount,
                            "input_mint": input_mint,
                            "output_mint": output_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Lend => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    tool_calls.insert(
                        "jupiter_lend".to_string(),
                        json!({
                            "amount": amount,
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Earn => {
                if let Some(input_mint) = &parameters.input_mint {
                    tool_calls.insert(
                        "jupiter_earn".to_string(),
                        json!({
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Borrow => {
                if let (Some(amount), Some(input_mint)) =
                    (&parameters.amount, &parameters.input_mint)
                {
                    tool_calls.insert(
                        "jupiter_borrow".to_string(),
                        json!({
                            "amount": amount,
                            "mint": input_mint,
                        }),
                    );
                }
            }
            crate::prompt_processor::PromptAction::Unknown => {
                // For unknown actions, we can't create specific tool calls
                debug!("Unknown action type, cannot create tool calls");
            }
        }

        debug!("Created tool calls from structured data: {:?}", tool_calls);
        Ok(tool_calls)
    }
}

// Implement required traits for RigAgent
impl HttpProvider for RigAgent {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

impl AgentProvider for RigAgent {
    fn agent_tools(&self) -> Option<Arc<AgentTools>> {
        self.agent_tools.clone()
    }
}
