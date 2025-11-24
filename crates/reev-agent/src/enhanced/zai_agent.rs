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
        let flow_mode_indicator = payload.allowed_tools.clone();

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

        // 🔧 ROUTING: Use correct API URL based on model
        // glm-4.6 uses ZAI_API_URL, glm-4.6-coding uses GLM_CODING_API_URL

        let base_url = match model_name {
            "glm-4.6-coding" => std::env::var("GLM_CODING_API_URL")
                .unwrap_or_else(|_| "https://api.z.ai/api/coding/paas/v4".to_string()),
            "glm-4.6" => std::env::var("ZAI_API_URL")
                .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string()),
            _ => std::env::var("ZAI_API_URL")
                .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string()),
        };

        let client = zai::Client::builder(&api_key).base_url(&base_url).build();

        // 🔧 MODEL MAPPING: glm-4.6-coding uses glm-4.6 at the coding endpoint
        let actual_model_name = match model_name {
            "glm-4.6-coding" => "glm-4.6",
            "glm-4.6" => "glm-4.6",
            _ => model_name,
        };

        // Create completion model using dynamic model parameter
        let model = client.completion_model(actual_model_name);

        // Verify the model is actually available before proceeding
        client.verify_model(actual_model_name).await
            .map_err(|e| anyhow::anyhow!("ZAI model '{actual_model_name}' validation failed: {e}. Please check if the model is available and your API credentials are correct."))?;

        // Helper function to check if a tool is allowed
        let is_tool_allowed = |tool_name: &str| -> bool {
            match &flow_mode_indicator {
                Some(tools) => tools.contains(&tool_name.to_string()),
                None => {
                    // SECURITY: Restrict jupiter_earn tool in normal mode (only available for position/earnings benchmarks 114-*.yml)
                    tool_name
                        != reev_types::ToolName::GetJupiterLendEarnPosition
                            .to_string()
                            .as_str()
                }
            }
        };

        // Re-enable balance tool for ZAI API to fix 400 errors
        // Balance tool definition handled in enum match below

        // Build completion request using unified enhanced user request
        let mut request_builder =
            CompletionRequestBuilder::new(model.clone(), &unified_data.enhanced_user_request);

        // Use type-safe enum-based tool selection
        for tool_name_str in reev_types::ToolRegistry::all_tools() {
            if is_tool_allowed(&tool_name_str) {
                let tool_enum = tool_name_str
                    .parse::<reev_types::ToolName>()
                    .unwrap_or(reev_types::ToolName::SolTransfer); // fallback
                match tool_enum {
                    reev_types::ToolName::SolTransfer => {
                        request_builder = request_builder
                            .tool(unified_data.tools.sol_tool.definition(String::new()).await);
                    }
                    reev_types::ToolName::SplTransfer => {
                        request_builder = request_builder
                            .tool(unified_data.tools.spl_tool.definition(String::new()).await);
                    }
                    reev_types::ToolName::JupiterSwap => {
                        // Use flow-aware tool in flow mode for proper swap_details structure
                        let flow_mode = flow_mode_indicator.is_some();
                        if flow_mode {
                            if let Some(ref flow_tool) = unified_data.tools.jupiter_swap_flow_tool {
                                request_builder =
                                    request_builder.tool(flow_tool.definition(String::new()).await);
                                info!("[ZAIAgent] Using JupiterSwapFlowTool in flow mode");
                            } else {
                                request_builder = request_builder.tool(
                                    unified_data
                                        .tools
                                        .jupiter_swap_tool
                                        .definition(String::new())
                                        .await,
                                );
                                info!("[ZAIAgent] Falling back to JupiterSwapTool (flow tool not available)");
                            }
                        } else {
                            request_builder = request_builder.tool(
                                unified_data
                                    .tools
                                    .jupiter_swap_tool
                                    .definition(String::new())
                                    .await,
                            );
                            info!("[ZAIAgent] Using JupiterSwapTool in normal mode");
                        }
                    }
                    reev_types::ToolName::JupiterLendEarnDeposit => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .jupiter_lend_earn_deposit_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::JupiterLendEarnWithdraw => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .jupiter_lend_earn_withdraw_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::JupiterLendEarnMint => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .jupiter_lend_earn_mint_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::JupiterLendEarnRedeem => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .jupiter_lend_earn_redeem_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::GetAccountBalance => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .balance_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::GetJupiterLendEarnTokens => {
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .lend_earn_tokens_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::GetJupiterLendEarnPosition => {
                        // jupiter_earn_tool only available for position/earnings benchmarks (114-*.yml)
                        request_builder = request_builder.tool(
                            unified_data
                                .tools
                                .get_jupiter_lend_earn_tool
                                .definition(String::new())
                                .await,
                        );
                    }
                    reev_types::ToolName::JupiterSwapFlow => {
                        if let Some(ref flow_tool) = unified_data.tools.jupiter_swap_flow_tool {
                            request_builder =
                                request_builder.tool(flow_tool.definition(String::new()).await);
                        }
                    }
                    reev_types::ToolName::ExecuteTransaction => {
                        // ExecuteTransaction not implemented for ZAI agent
                        info!("[ZAIAgent] ExecuteTransaction tool not supported in ZAI mode");
                        continue;
                    }
                }
            }
        }

        // For flow generation, we don't want to force tool usage
        // For regular operations, we need tools
        let is_flow_mode = flow_mode_indicator.is_some();

        // Debug request for flow mode
        if is_flow_mode {
            info!("[ZAIAgent] Flow mode detected");
        }

        let request = if is_flow_mode {
            request_builder.build()
        } else {
            request_builder
                .additional_params(json!({"tool_choice": "required"})) // Force LLM to use tools instead of generating transactions directly
                .build()
        };

        // Debug the model completion
        info!("[ZAIAgent] Sending request to model...");
        let result = match model.completion(request).await {
            Ok(result) => {
                info!("[ZAIAgent] ZAI completion successful");
                result
            }
            Err(e) => {
                tracing::error!("[ZAIAgent] ZAI completion failed: {}", e);
                return Err(anyhow::anyhow!("Model completion failed: {}", e));
            }
        };

        // Debug the result structure
        info!("[ZAIAgent] Result structure: {:?}", result);
        info!("[ZAIAgent] Result choice count: {}", result.choice.len());

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
                Self::execute_tool_call(tool_call, &unified_data.tools, &flow_mode_indicator)
                    .await?;

            info!("[ZAIAgent] Tool result: {}", tool_result);

            // Determine appropriate summary based on tool type - use type-safe enum parsing
            let tool_enum = tool_call
                .function
                .name
                .parse::<reev_types::ToolName>()
                .unwrap_or(reev_types::ToolName::SolTransfer); // fallback
            let summary = Self::get_tool_summary(&tool_enum);

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
            info!("[ZAIAgent] Full result object: {:?}", result);
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
        // Use type-safe enum parsing instead of hardcoded strings
        match tool_call.function.name.parse::<reev_types::ToolName>() {
            Ok(reev_types::ToolName::SolTransfer) => {
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
            Ok(reev_types::ToolName::GetJupiterLendEarnPosition) => {
                let args: reev_tools::tools::jupiter_lend_earn::GetJupiterLendEarnPositionArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .get_jupiter_lend_earn_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter earn error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            Ok(reev_types::ToolName::ExecuteTransaction) => {
                Err(anyhow::anyhow!("ExecuteTransaction is not implemented"))
            }
            Ok(reev_types::ToolName::SplTransfer) => {
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
            Ok(reev_types::ToolName::JupiterSwap) => {
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
            Ok(reev_types::ToolName::JupiterSwapFlow) => {
                if let Some(ref flow_tool) = tools.jupiter_swap_flow_tool {
                    let args: reev_tools::tools::flow::jupiter_swap_flow::JupiterSwapFlowArgs =
                        serde_json::from_value(tool_call.function.arguments.clone())?;
                    let result = flow_tool
                        .call(args)
                        .await
                        .map_err(|e| anyhow::anyhow!("Jupiter swap flow error: {e}"))?;
                    serde_json::to_value(result)
                        .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
                } else {
                    // Fallback to regular jupiter_swap if flow tool not available
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
            }
            Ok(reev_types::ToolName::JupiterLendEarnDeposit) => {
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
            Ok(reev_types::ToolName::JupiterLendEarnWithdraw) => {
                let args: reev_tools::tools::jupiter_lend_earn_withdraw::JupiterLendEarnWithdrawArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_withdraw_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend earn withdraw error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            Ok(reev_types::ToolName::JupiterLendEarnMint) => {
                let args: reev_tools::tools::jupiter_lend_earn_mint_redeem::JupiterLendEarnMintArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_mint_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend earn mint error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            Ok(reev_types::ToolName::JupiterLendEarnRedeem) => {
                let args: reev_tools::tools::jupiter_lend_earn_mint_redeem::JupiterLendEarnRedeemArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .jupiter_lend_earn_redeem_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Jupiter lend earn redeem error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            Ok(reev_types::ToolName::GetAccountBalance) => {
                let args: reev_tools::tools::discovery::balance_tool::AccountBalanceArgs =
                    serde_json::from_value(tool_call.function.arguments.clone())?;
                let result = tools
                    .balance_tool
                    .call(args)
                    .await
                    .map_err(|e| anyhow::anyhow!("Account balance error: {e}"))?;
                serde_json::to_value(result)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))
            }
            Ok(reev_types::ToolName::GetJupiterLendEarnTokens) => {
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
            Err(_) => Err(anyhow::anyhow!("Unknown tool: {}", tool_call.function.name)),
        }
    }

    /// 📝 Get appropriate summary for tool type - use type-safe matching
    fn get_tool_summary(tool_enum: &reev_types::ToolName) -> &'static str {
        match tool_enum {
            reev_types::ToolName::SolTransfer => "SOL transfer completed successfully",
            reev_types::ToolName::SplTransfer => "SPL transfer completed successfully",
            reev_types::ToolName::JupiterSwap => "Jupiter swap completed successfully",
            reev_types::ToolName::JupiterSwapFlow => "Jupiter swap flow completed successfully",
            reev_types::ToolName::JupiterLendEarnDeposit => {
                "Jupiter lend deposit completed successfully"
            }
            reev_types::ToolName::JupiterLendEarnWithdraw => {
                "Jupiter lend withdraw completed successfully"
            }
            reev_types::ToolName::JupiterLendEarnMint => "Jupiter lend mint completed successfully",
            reev_types::ToolName::JupiterLendEarnRedeem => {
                "Jupiter lend redeem completed successfully"
            }
            reev_types::ToolName::GetJupiterLendEarnPosition => {
                "Jupiter lend earn operation completed successfully"
            }
            reev_types::ToolName::GetAccountBalance => "Account balance retrieved successfully",
            reev_types::ToolName::GetJupiterLendEarnTokens => {
                "Lend earn tokens operation completed successfully"
            }
            reev_types::ToolName::ExecuteTransaction => {
                "Transaction execution completed successfully"
            }
        }
    }
}
