use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::{enhanced::openai::OpenAIAgent, enhanced::zai_agent::ZAIAgent, LlmRequest};

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

    // Initialize enhanced otel logger with session ID from payload - MUST HAPPEN FIRST
    // This ensures deterministic agents also get enhanced OTEL logging
    match reev_flow::get_enhanced_otel_logger() {
        Ok(logger) => {
            // Logger already initialized, check if it has the correct session_id
            if logger.session_id() != payload.session_id {
                tracing::warn!(
                    "[run_agent] Logger has different session_id: {} vs expected: {}",
                    logger.session_id(),
                    payload.session_id
                );
            }
        }
        Err(_) => {
            // Logger not initialized, initialize with session_id
            if let Ok(log_file) =
                reev_flow::init_enhanced_otel_logging_with_session(payload.session_id.clone())
            {
                tracing::info!(
                    "[run_agent] Enhanced OpenTelemetry logging initialized for session: {}",
                    log_file
                );
            } else {
                tracing::warn!("[run_agent] Failed to initialize Enhanced OpenTelemetry logging");
            }
        }
    }

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

    // Extract key_map from payload first (primary source)
    let mut key_map = payload.key_map.clone().unwrap_or_default();

    // If no key_map in payload, try to extract from YAML context (fallback)
    if key_map.is_empty() {
        let yaml_str = payload
            .context_prompt
            .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
            .trim_end_matches("\n\n\n---")
            .trim();

        let context: AgentContext = serde_yaml::from_str(yaml_str).unwrap_or(AgentContext {
            key_map: HashMap::new(),
        });
        key_map = context.key_map;
    }

    // Debug: Log key_map being passed to tools
    debug!("[run_agent] Key map for tools: {:?}", key_map);

    // Route to appropriate enhanced agent based on model type
    if model_name.starts_with("glm-") {
        info!("[run_agent] Routing GLM model: {}", model_name);
        if std::env::var("ZAI_API_KEY").is_ok() {
            // Route GLM models to appropriate agent based on type
            match model_name {
                "glm-4.6" => {
                    // glm-4.6 uses OpenAI agent with ZAI_API_URL
                    info!("[run_agent] ROUTING: glm-4.6 -> OpenAI agent (ZAI_API_URL)");
                    OpenAIAgent::run(model_name, payload, key_map).await
                }
                "glm-4.6-coding" => {
                    // glm-4.6-coding uses ZAI-specific client with GLM_CODING_API_URL
                    info!("[run_agent] ROUTING: glm-4.6-coding -> ZAI agent (GLM_CODING_API_URL)");
                    ZAIAgent::run(model_name, payload, key_map).await
                }
                _ => {
                    // Other GLM models default to OpenAI agent
                    info!(
                        "[run_agent] ROUTING: {} -> OpenAI agent (default)",
                        model_name
                    );
                    OpenAIAgent::run(model_name, payload, key_map).await
                }
            }
        } else {
            Err(anyhow::anyhow!(
                "GLM model '{model_name}' requires ZAI_API_KEY environment variable"
            ))
        }
    } else if model_name == "local" {
        // Real local model - route to OpenAI agent which supports local LLM servers
        info!("[run_agent] Using real local model via OpenAI agent");
        OpenAIAgent::run(model_name, payload, key_map).await
    } else if model_name.starts_with("gpt-")
        || model_name.starts_with("claude-")
        || model_name.starts_with("o1-")
    {
        // Other cloud models - route to OpenAI agent
        info!(
            "[run_agent] Using cloud model via OpenAI agent: {}",
            model_name
        );
        OpenAIAgent::run(model_name, payload, key_map).await
    } else {
        // Route to deterministic agent for unknown models
        info!(
            "[run_agent] Unknown model '{}' detected, routing to deterministic agent",
            model_name
        );
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
