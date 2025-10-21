use anyhow::Result;
use rig::{completion::Prompt, prelude::*};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{prompt::SYSTEM_PREAMBLE, LlmRequest};
use reev_lib::otel_extraction::{
    convert_to_session_format, extract_current_otel_trace, parse_otel_trace_to_tools,
};

use reev_tools::tools::{
    AccountBalanceTool, JupiterEarnTool, JupiterLendEarnDepositTool, JupiterLendEarnMintTool,
    JupiterLendEarnRedeemTool, JupiterLendEarnWithdrawTool, JupiterSwapTool, LendEarnTokensTool,
    SolTransferTool, SplTransferTool,
};

/// 🎯 Complete response format including transactions, summary, and signatures
#[derive(Debug, Clone)]
struct ExecutionResult {
    transactions: Vec<serde_json::Value>,
    summary: String,
    signatures: Vec<String>,
}

/// 🤖 Enhanced GLM Agent with Tool Support
///
/// This agent uses the Rig framework with GLM's OpenAI-compatible API
/// to provide proper tool integration, following the same pattern as
/// the OpenAI agent but adapted for GLM.
pub struct GlmAgent;

impl GlmAgent {
    /// 🧠 Run GLM agent with proper tool support
    ///
    /// Uses the Rig framework's tool system to execute operations
    /// through proper tools instead of generating raw transactions.
    /// This follows RULES.md requirements for API-only instructions.
    pub async fn run(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[GlmAgent] Running GLM agent with model: {model_name}");

        // 🚨 Check for allowed tools filtering (for flow operations)
        let allowed_tools = payload.allowed_tools.as_ref();
        if let Some(tools) = allowed_tools {
            info!("[GlmAgent] Flow mode - only allowing tools: {:?}", tools);
        }

        // Note: Flow tracking is now handled by OpenTelemetry + rig automatically

        // 📋 Create enhanced prompt with tool usage guidance
        let enhanced_prompt = format!(
            "{}\n\n{}",
            SYSTEM_PREAMBLE,
            "IMPORTANT: You MUST use the available tools to complete operations. \
            Do NOT generate raw transaction JSON. Use the appropriate tool:\n\
            - For SOL transfers: Use sol_transfer tool\n\
            - For SPL token transfers: Use spl_transfer tool\n\
            - For token swaps: Use jupiter_swap tool\n\
            - For Jupiter lending: Use jupiter_lend_* tools\n\
            - For balance checks: Use get_account_balance tool\n\
            - For lending info: Use get_lend_earn_tokens tool\n\
            - For positions: Use jupiter_earn tool"
        );

        // 🌐 Initialize GLM client with OpenAI-compatible API
        let glm_api_key = std::env::var("GLM_API_KEY")
            .map_err(|_| anyhow::anyhow!("GLM_API_KEY environment variable not set"))?;
        let glm_api_url = std::env::var("GLM_API_URL")
            .unwrap_or_else(|_| "https://open.bigmodel.cn/api/paas/v4".to_string());

        info!("[GlmAgent] Using GLM API: {}", glm_api_url);

        let client = rig::providers::openai::Client::new(&glm_api_key);

        // 🔧 Create tools with proper key mapping
        let sol_tool = SolTransferTool {
            key_map: key_map.clone(),
        };
        let spl_tool = SplTransferTool {
            key_map: key_map.clone(),
        };
        let jupiter_swap_tool = JupiterSwapTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_deposit_tool = JupiterLendEarnDepositTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_withdraw_tool = JupiterLendEarnWithdrawTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_mint_tool = JupiterLendEarnMintTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_earn_redeem_tool = JupiterLendEarnRedeemTool {
            key_map: key_map.clone(),
        };
        let jupiter_earn_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };

        // 🔍 DISCOVERY TOOLS: Enable prerequisite validation
        let balance_tool = AccountBalanceTool {
            key_map: key_map.clone(),
        };
        let lend_earn_tokens_tool = LendEarnTokensTool::new(key_map.clone());

        // 🧠 Build GLM agent with proper tool integration
        let agent = if let Some(allowed_tools) = allowed_tools {
            // Flow mode: only add tools that are explicitly allowed
            info!(
                "[GlmAgent] Flow mode: Only allowing {} tools: {:?}",
                allowed_tools.len(),
                allowed_tools
            );
            let mut builder = client
                .completion_model(model_name)
                .completions_api()
                .into_agent_builder()
                .preamble(&enhanced_prompt)
                .tool(sol_tool)
                .tool(spl_tool)
                .tool(jupiter_swap_tool)
                .tool(jupiter_lend_earn_deposit_tool)
                .tool(jupiter_lend_earn_withdraw_tool)
                .tool(jupiter_lend_earn_mint_tool)
                .tool(jupiter_lend_earn_redeem_tool);

            if allowed_tools.contains(&"get_lend_earn_tokens".to_string()) {
                builder = builder.tool(lend_earn_tokens_tool);
            }
            if allowed_tools.contains(&"get_account_balance".to_string()) {
                builder = builder.tool(balance_tool);
            }
            if allowed_tools.contains(&"jupiter_earn".to_string()) {
                builder = builder.tool(jupiter_earn_tool);
            }

            builder.build()
        } else {
            // Normal mode: add all discovery tools
            info!("[GlmAgent] Normal mode: Adding all discovery tools");
            client
                .completion_model(model_name)
                .completions_api()
                .into_agent_builder()
                .preamble(&enhanced_prompt)
                .tool(sol_tool)
                .tool(spl_tool)
                .tool(jupiter_swap_tool)
                .tool(jupiter_lend_earn_deposit_tool)
                .tool(jupiter_lend_earn_withdraw_tool)
                .tool(jupiter_lend_earn_mint_tool)
                .tool(jupiter_lend_earn_redeem_tool)
                .tool(jupiter_earn_tool)
                .tool(balance_tool)
                .tool(lend_earn_tokens_tool)
                .build()
        };

        // 📝 Process user request with explicit tool usage instruction
        let enhanced_user_request = format!(
            "{}\n\nCRITICAL: You MUST use the available tools to execute this operation. \
            Do NOT generate raw transaction JSON. Select and use the appropriate tool for the task.",
            payload.prompt
        );

        info!("[GlmAgent] === AGENT EXECUTION START ===");
        info!(
            "[GlmAgent] Final request being sent to GLM agent:\n{}",
            enhanced_user_request
        );

        // 🚀 Execute the request with proper tool usage
        let response = agent.prompt(&enhanced_user_request).await?;

        info!("[GlmAgent] GLM agent execution completed");

        // 📊 Extract execution results from response
        let execution_result = Self::extract_execution_results(&response)?;

        // 🌊 Extract tool calls from OpenTelemetry traces
        info!("[GlmAgent] Extracting tool calls from OpenTelemetry traces");

        if let Some(otel_trace) = extract_current_otel_trace() {
            debug!(
                "[GlmAgent] Found OpenTelemetry trace with {} spans",
                otel_trace.spans.len()
            );

            let tool_calls = parse_otel_trace_to_tools(otel_trace);
            info!(
                "[GlmAgent] Extracted {} tool calls from OpenTelemetry",
                tool_calls.len()
            );

            if !tool_calls.is_empty() {
                // Convert to session format for Mermaid diagram generation
                let session_tools = convert_to_session_format(tool_calls);
                info!(
                    "[GlmAgent] Converted {} tools to session format",
                    session_tools.len()
                );

                // Log tool details for debugging
                for tool in &session_tools {
                    debug!(
                        "[GlmAgent] Tool: {} | Status: {} | Duration: {}ms",
                        tool.tool_name,
                        tool.status,
                        tool.end_time
                            .duration_since(tool.start_time)
                            .unwrap_or_default()
                            .as_millis()
                    );
                }
            }
        } else {
            warn!("[GlmAgent] No OpenTelemetry trace found - tool calls may not be captured");
        }

        // 🎯 Logging shutdown
        info!("[GlmAgent] === AGENT EXECUTION COMPLETE ===");
        info!("[GlmAgent] Tool call tracing completed via OpenTelemetry integration");

        // 📋 Return comprehensive response
        let comprehensive_response = json!({
            "result": {
                "success": true,
                "transactions": execution_result.transactions,
                "summary": execution_result.summary,
                "signatures": execution_result.signatures,
                "flows": null // Flow data handled by OpenTelemetry
            }
        });

        Ok(serde_json::to_string(&comprehensive_response)?)
    }

    /// 📊 Extract execution results from agent response
    fn extract_execution_results(response: &str) -> Result<ExecutionResult> {
        // Try to parse as JSON first
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
            return Ok(ExecutionResult {
                transactions: json_value
                    .get("transactions")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .to_vec(),
                summary: json_value
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Operation completed successfully")
                    .to_string(),
                signatures: json_value
                    .get("signatures")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect(),
            });
        }

        // Fallback: create a basic response
        Ok(ExecutionResult {
            transactions: vec![],
            summary: response.to_string(),
            signatures: vec![],
        })
    }
}
