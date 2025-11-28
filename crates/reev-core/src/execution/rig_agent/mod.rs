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

use std::string::String;
use std::sync::Arc;
use tracing::{info, instrument};

use crate::yml_schema::YmlStep;

// Import modules
mod context;
mod prompting;
mod tool_execution;
mod types;

// Re-export types and traits
pub use context::ContextProvider;
pub use prompting::{HttpProvider, MultiStepHandler, PromptProvider};
pub use tool_execution::{AgentProvider, AgentToolHelper, ToolExecutor};
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
        info!(
            "DEBUG: execute_step_with_rig_and_history - USDC balance in context: {:?}",
            wallet_context
                .token_balances
                .get("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
                .map(|t| t.balance)
        );

        // Use the refined prompt if available, otherwise use the original prompt
        let prompt = if !step.refined_prompt.is_empty() {
            step.refined_prompt.clone()
        } else {
            step.prompt.clone()
        };

        // Create YML context and convert to prompt
        let yml_context = self.create_yml_context(step, wallet_context, previous_results)?;
        let context_prompt = self.yml_context_to_prompt(&yml_context, &prompt)?;

        // Log the YML context for debugging
        info!(
            "Generated YML context for step {}: {:?}",
            step.step_id, yml_context
        );

        // Get expected tools hints from the step
        let expected_tools = step.expected_tools.clone();

        // If we have expected tools, use them to guide the agent
        let response = if let Some(tools) = expected_tools {
            info!("Using expected tools to guide agent: {:?}", tools);
            self.prompt_with_expected_tools(&context_prompt, &tools)
                .await?
        } else {
            info!("No expected tools provided, using general agent prompt");
            self.prompt_agent(&context_prompt).await?
        };

        info!("Got response from agent: {}", response);

        info!("DEBUG: Parsing tool calls from LLM response: {}", response);

        // Extract tool calls from the response
        let tool_calls = self.extract_tool_calls(&response)?;

        // Check if this is a multi-step prompt and we have multiple operations
        let prompt_lower = prompt.to_lowercase();
        let is_multi_step = prompt_lower.contains(" then ")
            || prompt_lower.contains(" and ")
            || prompt_lower.contains(" followed by ");

        info!("DEBUG: is_multi_step = {}", is_multi_step);
        info!("DEBUG: Initial tool_calls count = {}", tool_calls.len());
        info!("DEBUG: Initial tool_calls = {:?}", tool_calls);
        info!("DEBUG: Response = {}", response);

        // For multi-step prompts, we need to ensure we extract all operations
        let tool_calls = if is_multi_step && tool_calls.len() < 2 {
            // Try to extract additional operations if we only got one tool call
            info!("Multi-step prompt detected but only one tool call extracted, attempting to extract additional operations");
            let additional_calls = self.extract_multi_step_tool_calls(&response, &tool_calls)?;
            info!("DEBUG: Additional tool_calls = {:?}", additional_calls);
            additional_calls
        } else {
            info!("DEBUG: Using initial tool_calls as-is");
            tool_calls
        };

        // Execute the selected tools
        info!("Tool calls extracted: {:?}", tool_calls);
        let tool_results = self
            .execute_tools(tool_calls.clone(), wallet_context)
            .await?;
        info!("Tool execution results: {:?}", tool_results);

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
