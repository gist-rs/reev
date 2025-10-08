use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

use crate::{
    enhanced::{gemini::GeminiAgent, openai::OpenAIAgent},
    LlmRequest,
};

/// A minimal struct for deserializing the `key_map` from the `context_prompt` YAML.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AgentContext {
    key_map: HashMap<String, String>,
}

/// Main dispatcher for AI agents with enhanced context capabilities
/// Routes requests to appropriate agent based on model type and provides enhanced context.
pub async fn run_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    info!("[run_agent] Dispatching to enhanced agent with model: {model_name}");

    // Check if mock is enabled and route to deterministic agent
    if payload.mock {
        info!("[run_agent] Mock mode enabled, routing to deterministic agent");
        let response = crate::run_deterministic_agent(payload).await?;

        // Extract the text field from LlmResponse
        let response_text = response
            .result
            .as_ref()
            .map(|r| r.text.clone())
            .unwrap_or_else(String::new);
        info!(
            "[run_agent] Deterministic agent response: {}",
            response_text
        );
        return Ok(response_text);
    }

    // Parse the context to extract key_map for tools
    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();

    let context: AgentContext = serde_yaml::from_str(yaml_str).unwrap_or(AgentContext {
        key_map: HashMap::new(),
    });
    let key_map = context.key_map;

    // Route to appropriate enhanced agent based on model type
    if model_name.starts_with("gemini") {
        info!("[run_agent] Using Gemini enhanced agent");
        GeminiAgent::run(model_name, payload, key_map).await
    } else if model_name == "local" {
        // Real local model - route to OpenAI agent which supports local LLM servers
        info!("[run_agent] Using real local model via OpenAI agent");
        OpenAIAgent::run(model_name, payload, key_map).await
    } else {
        // Route to deterministic agent
        info!("[run_agent] Legacy local-model detected, routing to deterministic agent");
        let response = crate::run_deterministic_agent(payload).await?;
        // Extract the text field from LlmResponse
        let response_text = response
            .result
            .as_ref()
            .map(|r| r.text.clone())
            .unwrap_or_else(String::new);
        info!(
            "[run_agent] Deterministic agent response: {}",
            response_text
        );
        Ok(response_text)
    }
}
