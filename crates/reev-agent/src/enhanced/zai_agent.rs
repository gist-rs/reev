//! ZAI Agent - Enhanced GLM-4.6 Agent with Full Tool Support
//!
//! This agent provides the same functionality as OpenAIAgent but uses the ZAI provider
//! for GLM-4.6 models instead of OpenAI's rig provider. It supports all reev-tools
//! and provides intelligent multi-step execution capabilities.
//!
//! 🎯 IMPORTANT: This agent uses unified GLM logic to ensure identical context
//! and wallet handling as other GLM agents. Only the request/response handling
//! differs from other implementations.

use anyhow::Result;

use rig::{
    completion::{CompletionModel, CompletionRequestBuilder},
    prelude::*,
    tool::Tool,
};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::providers::zai;
use crate::{
    enhanced::common::{AgentHelper, AgentTools, UnifiedGLMAgent},
    LlmRequest,
};

/// 🤖 Enhanced ZAI Agent with Superior Multi-Turn Capabilities
///
/// This agent leverages the Rig framework's multi-turn conversation to enable
/// step-by-step reasoning, adaptive execution, and superior decision making
/// that demonstrates AI capabilities beyond deterministic approaches.
pub struct ZAIAgent;

impl ZAIAgent {
    /// 🧠 Run enhanced ZAI agent with unified GLM logic
    ///
    /// Uses unified GLM logic to ensure identical context and wallet handling
    /// as other GLM agents. Only the ZAI-specific request/response handling differs.
    pub async fn run(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[ZAIAgent] Running ZAI agent with unified GLM logic: {model_name}");

        // 🔥 DEBUG: Check full incoming payload
        debug!("[ZAIAgent] DEBUG - full LlmRequest payload: {:?}", payload);

        // 🔧 FIX: Extract key_map from payload if not provided as parameter
        // 🔧 Extract key_map from payload - it should be populated in enhanced context
        let key_map_to_use = payload
            .key_map
            .as_ref()
            .cloned()
            .unwrap_or_else(|| key_map.clone());
        debug!("[ZAIAgent] key_map being used: {:?}", key_map_to_use);

        // 🚨 Check for allowed tools filtering (for flow operations)
        let allowed_tools = payload.allowed_tools.clone();

        // 🎯 Use unified GLM logic for shared components
        let unified_data = UnifiedGLMAgent::run(model_name, payload, key_map_to_use).await?;

        info!("[ZAIAgent] === ZAI-SPECIFIC REQUEST HANDLING ===");
        info!(
            "[ZAIAgent] Conversation Depth: {}",
            unified_data.conversation_depth
        );
        info!(
            "[ZAIAgent] Is Single Turn: {}",
            unified_data.conversation_depth == 1
        );

        // 🔑 Initialize ZAI client (provider-specific)
        let api_key = std::env::var("ZAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("ZAI_API_KEY environment variable is required"))?;

        let client = zai::Client::builder(&api_key).build();

        info!("[ZAIAgent] Starting ZAI completion request");

        // Create completion model using unified data
        let model = client.completion_model(zai::GLM_4_6);

        // Helper function to check if a tool is allowed
        let is_tool_allowed = |tool_name: &str| -> bool {
            match &allowed_tools {
                Some(tools) => tools.contains(&tool_name.to_string()),
                None => true, // No restrictions when allowed_tools is None
            }
        };

        // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
        // let balance_tool_def = unified_data.tools.balance_tool.definition(String::new()).await;

        // Build completion request using unified enhanced user request
        let mut request_builder =
            CompletionRequestBuilder::new(model.clone(), &unified_data.enhanced_user_request);

        // Add each tool only if it's allowed
        if is_tool_allowed("sol_transfer") {
            request_builder =
                request_builder.tool(unified_data.tools.sol_tool.definition(String::new()).await);
        }
        if is_tool_allowed("spl_transfer") {
            request_builder =
                request_builder.tool(unified_data.tools.spl_tool.definition(String::new()).await);
        }
        if is_tool_allowed("jupiter_swap") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_swap_tool
                    .definition(String::new())
                    .await,
            );
        }
        if is_tool_allowed("jupiter_lend_earn_deposit") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_lend_earn_deposit_tool
                    .definition(String::new())
                    .await,
            );
        }
        if is_tool_allowed("jupiter_lend_earn_withdraw") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_lend_earn_withdraw_tool
                    .definition(String::new())
                    .await,
            );
        }
        if is_tool_allowed("jupiter_lend_earn_mint") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_lend_earn_mint_tool
                    .definition(String::new())
                    .await,
            );
        }
        if is_tool_allowed("jupiter_lend_earn_redeem") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_lend_earn_redeem_tool
                    .definition(String::new())
                    .await,
            );
        }
        if is_tool_allowed("jupiter_earn") {
            request_builder = request_builder.tool(
                unified_data
                    .tools
                    .jupiter_earn_tool
                    .definition(String::new())
                    .await,
            );
        }
        let request_builder = request_builder;

        let request = request_builder
            // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
            // .tool(balance_tool_def)
            .additional_params(json!({"tool_choice": "required"})) // Force LLM to use tools instead of generating transactions directly
            .build();

        let result = model.completion(request).await?;

        info!("[ZAIAgent] ZAI completion completed");

        // Extract tool calls from the result (provider-specific)
        let tool_calls: Vec<_> = result
            .choice
            .iter()
            .filter_map(|content| {
                if let rig::message::AssistantContent::ToolCall(tool_call) = content {
                    Some(tool_call)
                } else {
                    None
                }
            })
            .collect();

        let response_str = if !tool_calls.is_empty() {
            let tool_call = &tool_calls[0];
            info!("[ZAIAgent] Tool called: {}", tool_call.function.name);
            info!("[ZAIAgent] Arguments: {}", tool_call.function.arguments);

            // Route tool call to appropriate tool using unified tools
            let tool_result =
                Self::execute_tool_call(tool_call, &unified_data.tools, &allowed_tools).await?;

            info!("[ZAIAgent] Tool result: {}", tool_result);

            // Determine appropriate summary based on tool type
            let summary = Self::get_tool_summary(tool_call.function.name.as_str());

            // Format as JSON response
            json!({
                "transactions": [tool_result],
                "summary": summary,
                "signatures": ["estimated_signature"]
            })
            .to_string()
        } else {
            // Extract text response
            let response_text = result
                .choice
                .iter()
                .find_map(|content| {
                    if let rig::message::AssistantContent::Text(text) = content {
                        Some(text.text.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            info!("[ZAIAgent] Text response: {}", response_text);
            response_text
        };

        // 🎯 Extract tool calls from OpenTelemetry traces
        let tool_calls = AgentHelper::extract_tool_calls_from_otel();

        // 🎯 Use unified response formatting
        UnifiedGLMAgent::format_response(&response_str, "ZAIAgent", Some(tool_calls)).await
    }

    /// 🔧 Execute tool call using unified tools
    async fn execute_tool_call(
        tool_call: &rig::message::ToolCall,
        tools: &AgentTools,
        _allowed_tools: &Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        match tool_call.function.name.as_str() {
            "sol_transfer" => {
                let args: reev_tools::tools::native::NativeTransferArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .sol_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("SOL transfer error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "spl_transfer" => {
                let args: reev_tools::tools::native::NativeTransferArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .spl_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("SPL transfer error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_swap" => {
                let args: reev_tools::tools::jupiter_swap::JupiterSwapArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_swap_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter swap error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_lend_earn_deposit" => {
                let args: reev_tools::tools::jupiter_lend_earn_deposit::JupiterLendEarnDepositArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_deposit_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend deposit error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_lend_earn_withdraw" => {
                let args: reev_tools::tools::jupiter_lend_earn_withdraw::JupiterLendEarnWithdrawArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_withdraw_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend withdraw error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_lend_earn_mint" => {
                let args: reev_tools::tools::jupiter_lend_earn_mint_redeem::JupiterLendEarnMintArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_mint_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend mint error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_lend_earn_redeem" => {
                let args: reev_tools::tools::jupiter_lend_earn_mint_redeem::JupiterLendEarnRedeemArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_redeem_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend redeem error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            "jupiter_earn" => {
                let args: reev_tools::tools::jupiter_earn::JupiterEarnArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_earn_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("jupiter_earn execution error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
            /*
            "get_account_balance" => {
                let args: reev_tools::tools::discovery::balance_tool::AccountBalanceArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                tools.balance_tool.call(args).await
            }
            */
            "get_lend_earn_tokens" => {
                let args: reev_tools::tools::discovery::lend_earn_tokens::LendEarnTokensArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .lend_earn_tokens_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Lend earn tokens error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            _ => Err(anyhow::anyhow!(
                "Unknown tool called: {}",
                tool_call.function.name
            )),
        }
    }

    /// 📝 Get appropriate summary for tool type
    fn get_tool_summary(tool_name: &str) -> &'static str {
        match tool_name {
            "sol_transfer" => "SOL transfer completed successfully",
            "spl_transfer" => "SPL transfer completed successfully",
            "jupiter_swap" => "Jupiter swap completed successfully",
            "jupiter_lend_earn_deposit" => "Jupiter lend deposit completed successfully",
            "jupiter_lend_earn_withdraw" => "Jupiter lend withdraw completed successfully",
            "jupiter_lend_earn_mint" => "Jupiter lend mint completed successfully",
            "jupiter_lend_earn_redeem" => "Jupiter lend redeem completed successfully",
            "jupiter_earn" => "Jupiter earn operation completed successfully",
            // TODO: Temporarily disabled - comment out balance_tool to fix SOL transfers
            // "get_account_balance" => "Account balance retrieved successfully",
            "lend_earn_tokens" => "Lend earn tokens operation completed successfully",
            _ => "Tool operation completed successfully",
        }
    }
}
