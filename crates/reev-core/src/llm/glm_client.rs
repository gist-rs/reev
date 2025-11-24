//! GLM Client Implementation for reev-core
//!
//! This module implements the LlmClient trait using the GLM-4.6-coding model
//! via the ZAI provider, leveraging existing implementation in reev-agent.

use crate::llm::prompt_templates::FlowPromptTemplate;
use crate::planner::LlmClient;
use anyhow::{anyhow, Result};
use reev_agent::enhanced::UnifiedGLMAgent;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};

/// GLM Client implementation for reev-core planner
pub struct GLMClient {
    model_name: String,
    api_key: String,
}

impl GLMClient {
    /// Create a new GLM client
    pub fn new(model_name: &str, api_key: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Initialize with environment variables
    pub fn from_env() -> Result<Self> {
        let model_name =
            std::env::var("GLM_MODEL").unwrap_or_else(|_| "glm-4.6-coding".to_string());
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow!("ZAI_API_KEY environment variable not set"))?;

        Ok(Self::new(&model_name, &api_key))
    }
}

#[async_trait::async_trait]
impl LlmClient for GLMClient {
    #[instrument(skip(self))]
    async fn generate_flow(&self, prompt: &str) -> Result<String> {
        info!("Generating flow using GLM-4.6-coding model");

        // Build structured prompt for YML flow generation
        let flow_prompt = FlowPromptTemplate::build_flow_prompt(prompt);

        debug!("Calling GLM with structured prompt");

        // Create request payload for UnifiedGLMAgent
        let payload = reev_agent::LlmRequest {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            prompt: flow_prompt.clone(),
            context_prompt: "You are a helpful DeFi assistant that generates structured YML flows."
                .to_string(),
            model_name: self.model_name.clone(),
            mock: false,
            initial_state: None,
            allowed_tools: None, // No tools needed for flow generation
            account_states: None,
            key_map: None, // Will be set in the next step
        };

        // Set up key_map for authentication
        let mut key_map = HashMap::new();
        key_map.insert("ZAI_API_KEY".to_string(), self.api_key.clone());

        // Use the existing UnifiedGLMAgent implementation
        let result = UnifiedGLMAgent::run(&self.model_name, payload, key_map)
            .await
            .map_err(|e| {
                error!("Failed to generate flow with GLM: {}", e);
                anyhow!("LLM generation failed: {e}")
            })?;

        // Extract the execution result from the UnifiedGLMAgent response
        let flow_yml = result.execution_result.summary;

        debug!("Received GLM response: {}", flow_yml);

        Ok(flow_yml)
    }
}

/// Initialize GLM client with environment configuration
pub fn init_glm_client() -> Result<Box<dyn crate::planner::LlmClient>> {
    let client = GLMClient::from_env()?;
    Ok(Box::new(client))
}
