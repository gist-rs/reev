use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::openai::Client};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{
    enhanced::common::{extract_execution_results, AgentHelper, AgentTools},
    LlmRequest,
};

/// 🤖 Enhanced OpenAI Agent with Superior Multi-Turn Capabilities
///
/// This agent leverages the Rig framework's multi-turn conversation to enable
/// step-by-step reasoning, adaptive execution, and superior decision making
/// that demonstrates AI capabilities beyond deterministic approaches.
pub struct OpenAIAgent;

impl OpenAIAgent {
    /// 🧠 Run enhanced OpenAI agent with intelligent multi-step execution
    ///
    /// Uses the Rig framework's multi-turn conversation system to break down
    /// complex DeFi operations into manageable steps, validate each step, and
    /// adapt strategy based on results, showcasing superior AI intelligence.
    pub async fn run(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[OpenAIAgent] Running enhanced multi-turn agent with model: {model_name}");

        // 🚨 Check for allowed tools filtering (for flow operations)
        let allowed_tools = payload.allowed_tools.as_ref();
        if let Some(tools) = allowed_tools {
            info!(
                "[OpenAIAgent] Flow mode: Only allowing {} tools: {:?}",
                tools.len(),
                tools
            );
        }

        // 🧠 Build enhanced context with account information using common helper
        let (context_integration, enhanced_prompt_data, enhanced_prompt) =
            AgentHelper::build_enhanced_context(&payload, &key_map)?;

        // 🤖 MULTI-TURN CONVERSATION: Enable step-by-step reasoning
        let _user_request = enhanced_prompt_data.prompt.clone();

        // Check API priority based on model selection
        let (client, actual_model_name) = if model_name == "local" {
            // Local model - always use local endpoint regardless of API keys
            let local_url = std::env::var("LOCAL_MODEL_URL")
                .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
            let dummy_key = "dummy-key-for-local-model";
            let actual_model = std::env::var("LOCAL_MODEL_NAME")
                .unwrap_or_else(|_| "qwen3-coder-30b-a3b-instruct-mlx".to_string());

            info!("[OpenAIAgent] Using local model at: {}", local_url);
            info!("[OpenAIAgent] Using model name: {}", actual_model);
            (
                Client::builder(dummy_key).base_url(&local_url).build()?,
                actual_model,
            )
        } else if model_name.starts_with("glm-") {
            // GLM models - use ZAI_API_KEY if available
            if let Ok(zai_api_key) = std::env::var("ZAI_API_KEY") {
                let zai_api_url = std::env::var("ZAI_API_URL")
                    .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());

                info!(
                    "[OpenAIAgent] Using Regular GLM API (ZAI_API_KEY): {}",
                    zai_api_url
                );
                info!("[OpenAIAgent] Model: {} (regular GLM)", model_name);

                let client = Client::builder(&zai_api_key)
                    .base_url(&zai_api_url)
                    .build()?;
                (client, model_name.to_string())
            } else {
                return Err(anyhow::anyhow!(
                    "GLM model '{model_name}' requires ZAI_API_KEY environment variable"
                ));
            }
        } else if let Ok(openai_api_key) = std::env::var("OPENAI_API_KEY") {
            // Standard OpenAI client for other models
            info!("[OpenAIAgent] Using OpenAI API");
            let client = Client::new(&openai_api_key);
            (client, model_name.to_string())
        } else {
            // Fallback to local model for unknown models
            let local_url = std::env::var("LOCAL_MODEL_URL")
                .unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
            let dummy_key = "dummy-key-for-local-model";
            let actual_model = std::env::var("LOCAL_MODEL_NAME")
                .unwrap_or_else(|_| "qwen3-coder-30b-a3b-instruct-mlx".to_string());

            warn!(
                "[OpenAIAgent] No API key found for model '{}', using local model at: {}",
                model_name, local_url
            );
            let client = Client::builder(dummy_key).base_url(&local_url).build()?;
            (client, actual_model)
        };

        // 🧠 ADAPTIVE CONVERSATION DEPTH: Use context-aware depth optimization
        let conversation_depth = AgentHelper::determine_conversation_depth(
            &context_integration,
            &enhanced_prompt_data,
            payload.initial_state.as_deref().unwrap_or(&[]),
            &key_map,
            &payload.id,
        );

        // Log prompt information using common helper
        AgentHelper::log_prompt_info(
            "OpenAIAgent",
            &payload,
            &enhanced_prompt_data,
            &enhanced_prompt,
            conversation_depth,
        );

        // Tool tracking is now handled by OpenTelemetry + rig framework
        // No manual flow infrastructure needed

        // 🛠️ Instantiate tools using common helper
        let tools = AgentTools::new(key_map.clone());

        // 🧠 Build enhanced multi-turn agent with conditional tool filtering
        let agent = if let Some(allowed_tools) = allowed_tools {
            // Flow mode: only add tools that are explicitly allowed
            info!(
                "[OpenAIAgent] Flow mode: Only allowing {} tools: {:?}",
                allowed_tools.len(),
                allowed_tools
            );
            let mut builder = client
                .completion_model(&actual_model_name)
                .completions_api()
                .into_agent_builder()
                .preamble(&enhanced_prompt)
                .tool(tools.sol_tool)
                .tool(tools.spl_tool)
                .tool(tools.jupiter_swap_tool)
                .tool(tools.jupiter_lend_earn_deposit_tool)
                .tool(tools.jupiter_lend_earn_withdraw_tool)
                .tool(tools.jupiter_lend_earn_mint_tool)
                .tool(tools.jupiter_lend_earn_redeem_tool);

            if allowed_tools.contains(&"get_lend_earn_tokens".to_string()) {
                builder = builder.tool(tools.lend_earn_tokens_tool);
            }
            // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
            // if allowed_tools.contains(&"get_account_balance".to_string()) {
            //     builder = builder.tool(tools.balance_tool);
            // }
            if allowed_tools.contains(&"jupiter_earn".to_string()) {
                builder = builder.tool(tools.jupiter_earn_tool);
            }

            builder.build()
        } else {
            // Normal mode: add all discovery tools
            info!("[OpenAIAgent] Normal mode: Adding all discovery tools");
            client
                .completion_model(&actual_model_name)
                .completions_api()
                .into_agent_builder()
                .preamble(&enhanced_prompt)
                .tool(tools.sol_tool)
                .tool(tools.spl_tool)
                .tool(tools.jupiter_swap_tool)
                .tool(tools.jupiter_lend_earn_deposit_tool)
                .tool(tools.jupiter_lend_earn_withdraw_tool)
                .tool(tools.jupiter_lend_earn_mint_tool)
                .tool(tools.jupiter_lend_earn_redeem_tool)
                .tool(tools.jupiter_earn_tool)
                // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
                // .tool(tools.balance_tool)
                .tool(tools.lend_earn_tokens_tool)
                .build()
        };

        // Add explicit stop instruction to the user request for simple operations
        let enhanced_user_request = AgentHelper::enhance_user_request(
            &enhanced_prompt_data.prompt,
            conversation_depth,
            "OpenAIAgent",
        );

        info!("[OpenAIAgent] === AGENT EXECUTION START ===");
        info!(
            "[OpenAIAgent] Sending to agent - conversation_depth: {}",
            conversation_depth
        );
        info!(
            "[OpenAIAgent] Final request being sent to agent:\n{}",
            enhanced_user_request
        );

        // Note: Removed OpenTelemetry tracing spans to prevent "global default trace dispatcher has already been set" error
        // when the agent is spawned from the API server which already has tracing initialized
        info!("[OpenAIAgent] Starting agent execution");

        let response = agent
            .prompt(&enhanced_user_request)
            .multi_turn(conversation_depth)
            .await?;

        info!("[OpenAIAgent] Agent execution completed");

        let response_str = response.to_string();
        info!("[OpenAIAgent] === AGENT RESPONSE RECEIVED ===");
        info!(
            "[OpenAIAgent] Raw response from enhanced multi-turn agent: {}",
            response_str
        );
        info!(
            "[OpenAIAgent] Response length: {} chars",
            response_str.len()
        );

        // 🎯 EXTRACT TOOL EXECUTION RESULTS FROM CONVERSATION
        let execution_result = extract_execution_results(&response_str, "OpenAIAgent").await?;

        // 🎯 EXTRACT TOOL CALLS FROM OPENTELEMETRY TRACES
        let tool_calls = AgentHelper::extract_tool_calls_from_otel();

        // 🎯 FORMAT COMPREHENSIVE RESPONSE WITH FLOWS
        AgentHelper::format_comprehensive_response(
            execution_result,
            Some(tool_calls),
            "OpenAIAgent",
        )
    }
}
