//! 🤖 Enhanced OpenAI Agent with Superior Multi-Turn Capabilities
//!
//! This agent leverages the Rig framework's multi-turn conversation to enable
//! step-by-step reasoning, adaptive execution, and superior decision making
//! that demonstrates AI capabilities beyond deterministic approaches.
//!
//! 🎯 IMPORTANT: For GLM models, this agent uses unified GLM logic to ensure
//! identical context and wallet handling as ZAIAgent. Only the OpenAI-specific
//! request/response handling differs from other implementations.

use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::openai::Client};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{
    enhanced::common::{extract_execution_results, AgentHelper, AgentTools, UnifiedGLMAgent},
    LlmRequest,
};
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

        // 🎯 Check if this is a GLM model that should use unified logic
        if model_name.starts_with("glm-") {
            return Self::run_glm_with_unified_logic(model_name, payload, key_map).await;
        }

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

        // 🛠️ Instantiate tools using common helper with flow mode detection
        let flow_mode = allowed_tools.is_some(); // Flow mode when allowed_tools is Some
        let tools = AgentTools::new_with_flow_mode(key_map.clone(), flow_mode);

        // 🧠 Build enhanced multi-turn agent with conditional tool filtering
        let agent = if let Some(allowed_tools) = allowed_tools {
            // Flow mode: only add tools that are explicitly allowed
            info!(
                "[OpenAIAgent] Flow mode: Only allowing {} tools: {:?}",
                allowed_tools.len(),
                allowed_tools
            );
            // Helper function to check if a tool is allowed
            let is_tool_allowed =
                |tool_name: &str| -> bool { allowed_tools.contains(&tool_name.to_string()) };

            let mut builder = client
                .completion_model(&actual_model_name)
                .completions_api()
                .into_agent_builder()
                .preamble(&enhanced_prompt);

            // Add each tool only if it's allowed
            if is_tool_allowed("sol_transfer") {
                builder = builder.tool(tools.sol_tool);
            }
            if is_tool_allowed("spl_transfer") {
                builder = builder.tool(tools.spl_tool);
            }
            if is_tool_allowed("jupiter_swap") {
                // Use flow-aware tool in flow mode for proper swap_details structure
                if flow_mode {
                    if let Some(ref flow_tool) = tools.jupiter_swap_flow_tool {
                        builder = builder.tool(flow_tool.clone());
                        info!("[OpenAIAgent] Using JupiterSwapFlowTool in flow mode");
                    } else {
                        builder = builder.tool(tools.jupiter_swap_tool.clone());
                        info!("[OpenAIAgent] Falling back to JupiterSwapTool (flow tool not available)");
                    }
                } else {
                    builder = builder.tool(tools.jupiter_swap_tool.clone());
                    info!("[OpenAIAgent] Using JupiterSwapTool in normal mode");
                }
            }
            if is_tool_allowed("jupiter_lend_earn_deposit") {
                builder = builder.tool(tools.jupiter_lend_earn_deposit_tool);
            }
            if is_tool_allowed("jupiter_lend_earn_withdraw") {
                builder = builder.tool(tools.jupiter_lend_earn_withdraw_tool);
            }
            if is_tool_allowed("jupiter_lend_earn_mint") {
                builder = builder.tool(tools.jupiter_lend_earn_mint_tool);
            }
            if is_tool_allowed("jupiter_lend_earn_redeem") {
                builder = builder.tool(tools.jupiter_lend_earn_redeem_tool);
            }
            if is_tool_allowed("get_jupiter_lend_earn_tokens") {
                builder = builder.tool(tools.lend_earn_tokens_tool);
            }
            // Re-enable balance tool for consistency with ZAI agent
            if is_tool_allowed("get_account_balance") {
                builder = builder.tool(tools.balance_tool);
            }
            if is_tool_allowed("get_jupiter_earn_position") {
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
                .tool(tools.jupiter_swap_tool.clone())
                .tool(tools.jupiter_lend_earn_deposit_tool)
                .tool(tools.jupiter_lend_earn_withdraw_tool)
                .tool(tools.jupiter_lend_earn_mint_tool)
                .tool(tools.jupiter_lend_earn_redeem_tool)
                // jupiter_earn_tool only available for position/earnings benchmarks (114-*.yml)
                .tool(tools.balance_tool)
                // .tool(tools.jupiter_earn_tool) - REMOVED: Should only be available for position/earnings benchmarks
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

    /// 🧠 Run GLM models using unified logic for identical context and wallet handling
    ///
    /// This method ensures that GLM models routed through OpenAIAgent use the same
    /// context building, wallet creation, and prompt mapping as ZAIAgent. Only the
    /// OpenAI-specific request/response handling differs.
    async fn run_glm_with_unified_logic(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[OpenAIAgent] Running GLM model with unified logic: {model_name}");

        // 🎯 Use unified GLM logic for shared components
        let unified_data = UnifiedGLMAgent::run(model_name, payload, key_map).await?;

        info!("[OpenAIAgent] === OPENAI-SPECIFIC GLM REQUEST HANDLING ===");
        info!(
            "[OpenAIAgent] Conversation Depth: {}",
            unified_data.conversation_depth
        );

        // 🔑 Initialize OpenAI client for GLM models
        let (client, actual_model_name) = if let Ok(zai_api_key) = std::env::var("ZAI_API_KEY") {
            let zai_api_url = std::env::var("ZAI_API_URL")
                .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());

            info!(
                "[OpenAIAgent] Using GLM via OpenAI client with ZAI endpoint: {}",
                zai_api_url
            );

            info!(
                "[OpenAIAgent] 🔍 GLM CLIENT DEBUG: Built client with URL: {}",
                zai_api_url
            );
            info!(
                "[OpenAIAgent] 🔍 GLM CLIENT DEBUG: Model name: {}",
                model_name
            );
            info!(
                "[OpenAIAgent] 🔍 GLM CLIENT DEBUG: API Key (first 10): {}...",
                &zai_api_key[..10.min(zai_api_key.len())]
            );

            let client = Client::builder(&zai_api_key)
                .base_url(&zai_api_url)
                .build()?;
            (client, model_name.to_string())
        } else {
            return Err(anyhow::anyhow!(
                "GLM model '{model_name}' requires ZAI_API_KEY environment variable"
            ));
        };

        // 🛠️ Build agent using unified tools and context
        let mut agent_builder = client
            .completion_model(&actual_model_name)
            .completions_api()
            .into_agent_builder()
            .preamble(&unified_data.enhanced_prompt)
            .tool(unified_data.tools.sol_tool)
            .tool(unified_data.tools.spl_tool);

        // Add appropriate Jupiter swap tool based on flow mode
        if let Some(ref flow_tool) = unified_data.tools.jupiter_swap_flow_tool {
            info!("[OpenAIAgent] Using JupiterSwapFlowTool in flow mode");
            agent_builder = agent_builder.tool(flow_tool.clone());
        } else {
            info!("[OpenAIAgent] Using JupiterSwapTool in normal mode");
            agent_builder = agent_builder.tool(unified_data.tools.jupiter_swap_tool.clone());
        }

        let agent = agent_builder
            .tool(unified_data.tools.jupiter_lend_earn_deposit_tool)
            .tool(unified_data.tools.jupiter_lend_earn_withdraw_tool)
            .tool(unified_data.tools.jupiter_lend_earn_mint_tool)
            .tool(unified_data.tools.jupiter_lend_earn_redeem_tool)
            // jupiter_earn_tool only available for position/earnings benchmarks (114-*.yml)
            .tool(unified_data.tools.balance_tool)
            .tool(unified_data.tools.lend_earn_tokens_tool)
            .build();

        info!("[OpenAIAgent] === OPENAI GLM EXECUTION START ===");
        info!(
            "[OpenAIAgent] Final request being sent to agent:\n{}",
            unified_data.enhanced_user_request
        );

        // Execute request using OpenAI's multi-turn agent

        let response = agent
            .prompt(&unified_data.enhanced_user_request)
            .multi_turn(unified_data.conversation_depth as usize)
            .await?;

        let response_str = response.to_string();
        info!("[OpenAIAgent] OpenAI GLM execution completed");

        // 🎯 Extract tool calls from OpenTelemetry traces
        let tool_calls = AgentHelper::extract_tool_calls_from_otel();

        // 🎯 Use unified response formatting
        UnifiedGLMAgent::format_response(&response_str, "OpenAIAgent-GM", Some(tool_calls)).await
    }
}
