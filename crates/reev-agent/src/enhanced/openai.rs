use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::openai::Client};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::{
    enhanced::enhanced_context::EnhancedContextAgent,
    prompt::SYSTEM_PREAMBLE,
    tools::{
        JupiterEarnTool, JupiterLendDepositTool, JupiterLendWithdrawTool, JupiterMintTool,
        JupiterRedeemTool, JupiterSwapTool, SolTransferTool, SplTransferTool,
    },
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

        // 🧠 Build enhanced context for superior AI reasoning
        let enhanced_context = EnhancedContextAgent::build_context(&payload, &key_map);
        let enhanced_prompt = format!("{SYSTEM_PREAMBLE}\n\n---\n{enhanced_context}\n---");

        // 🤖 MULTI-TURN CONVERSATION: Enable step-by-step reasoning
        let user_request = format!(
            "{}\n\nUSER REQUEST: {}",
            payload.context_prompt, payload.prompt
        );

        // 🐛 DEBUG: Log the full prompt being sent to the model
        info!(
            "[OpenAIAgent] DEBUG - Full prompt being sent to model:\n---\n{}\n---",
            user_request
        );

        // 🚀 Initialize OpenAI client
        let client = Client::builder("")
            .base_url("http://localhost:1234/v1")
            .build()?;

        // 🛠️ Instantiate tools with context-aware key_map
        let sol_tool = SolTransferTool {
            key_map: key_map.clone(),
        };
        let spl_tool = SplTransferTool {
            key_map: key_map.clone(),
        };
        let jupiter_swap_tool = JupiterSwapTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_deposit_tool = JupiterLendDepositTool {
            key_map: key_map.clone(),
        };
        let jupiter_lend_withdraw_tool = JupiterLendWithdrawTool {
            key_map: key_map.clone(),
        };
        let jupiter_mint_tool = JupiterMintTool {
            key_map: key_map.clone(),
        };
        let jupiter_redeem_tool = JupiterRedeemTool {
            key_map: key_map.clone(),
        };
        let jupiter_positions_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };
        let jupiter_earnings_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };

        // 🧠 Build enhanced multi-turn agent
        let agent = client
            .completion_model(model_name)
            .completions_api()
            .into_agent_builder()
            .preamble(&enhanced_prompt)
            .tool(sol_tool)
            .tool(spl_tool)
            .tool(jupiter_swap_tool)
            .tool(jupiter_lend_deposit_tool)
            .tool(jupiter_lend_withdraw_tool)
            .tool(jupiter_mint_tool)
            .tool(jupiter_redeem_tool)
            .tool(jupiter_positions_tool)
            .tool(jupiter_earnings_tool)
            .build();

        // 🧠 MULTI-TURN AGENT: Enable intelligent step-by-step reasoning
        let response = agent.prompt(&user_request).multi_turn(5).await?;

        let response_str = response.to_string();
        info!(
            "[OpenAIAgent] Raw response from enhanced multi-turn agent: {}",
            response_str
        );

        // 🐛 DEBUG: Try to parse the response and log any errors
        match serde_json::from_str::<serde_json::Value>(&response_str) {
            Ok(json_value) => {
                info!(
                    "[OpenAIAgent] Successfully parsed response as JSON: {}",
                    json_value
                );

                // Check if the response contains tool calls
                if let Some(tool_calls) = json_value.get("tool_calls") {
                    info!("[OpenAIAgent] Found tool calls in response: {}", tool_calls);
                } else {
                    warn!("[OpenAIAgent] No tool_calls field found in JSON response");
                }

                // 🧠 Process response and extract instructions
                let instruction = if let Some(instruction_field) = json_value.get("instruction") {
                    instruction_field
                } else {
                    &json_value
                };

                Ok(serde_json::to_string(instruction)?)
            }
            Err(parse_error) => {
                error!(
                    "[OpenAIAgent] Failed to parse response as JSON: {}",
                    parse_error
                );
                error!("[OpenAIAgent] Response was not JSON - this suggests the model didn't call tools");
                error!("[OpenAIAgent] Actual response: {}", response_str);

                // Return an error that will be caught by the caller
                Err(anyhow::anyhow!(
                    "Model returned natural language instead of JSON instructions. Response: {response_str}"
                ))
            }
        }
    }
}
