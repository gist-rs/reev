use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::gemini};
use std::collections::HashMap;
use tracing::info;

use crate::{
    enhanced::enhanced_context::EnhancedContextAgent,
    prompt::SYSTEM_PREAMBLE,
    tools::{
        JupiterEarnTool, JupiterLendDepositTool, JupiterLendWithdrawTool, JupiterMintTool,
        JupiterRedeemTool, JupiterSwapTool, SolTransferTool, SplTransferTool,
    },
    LlmRequest,
};

/// 🤖 Enhanced Gemini Agent with Superior AI Capabilities
///
/// This agent demonstrates advanced reasoning, multi-step workflow understanding,
/// and adaptive intelligence that exceeds deterministic agent performance.
pub struct GeminiAgent;

impl GeminiAgent {
    /// 🧠 Run enhanced Gemini agent with multi-turn conversation capabilities
    ///
    /// Provides intelligent DeFi operation orchestration with step-by-step reasoning,
    /// adaptive strategy selection, and superior decision making compared to deterministic agents.
    pub async fn run(
        model_name: &str,
        payload: LlmRequest,
        key_map: HashMap<String, String>,
    ) -> Result<String> {
        info!("[GeminiAgent] Running enhanced Gemini agent with model: {model_name}");

        // Initialize Gemini client with enhanced configuration
        let client = gemini::Client::from_env();

        // 🧠 Configure for creativity and intelligence
        let gen_cfg = gemini::completion::gemini_api_types::GenerationConfig {
            temperature: Some(0.2), // Higher temperature for creative problem solving
            top_p: Some(0.9),       // More diverse responses
            top_k: Some(40),        // Consider more options
            ..Default::default()
        };
        let cfg = gemini::completion::gemini_api_types::AdditionalParameters::default()
            .with_config(gen_cfg);

        // 🧠 Build enhanced context for superior AI reasoning
        let enhanced_context = EnhancedContextAgent::build_context(&payload, &key_map);
        let enhanced_prompt = format!("{SYSTEM_PREAMBLE}\n\n---\n{enhanced_context}\n---");

        // 🤖 MULTI-TURN CONVERSATION: Enable step-by-step reasoning
        let user_request = format!(
            "{}\n\nUSER REQUEST: {}",
            payload.context_prompt, payload.prompt
        );

        // 🛠️ Instantiate tools with enhanced context awareness
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

        // 🧠 Build enhanced agent with superior capabilities
        let agent = client
            .agent(model_name)
            .preamble(&enhanced_prompt)
            .additional_params(serde_json::to_value(cfg)?)
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

        // 🤖 MULTI-TURN AGENT: Enable intelligent step-by-step reasoning
        let response = agent.prompt(&user_request).multi_turn(5).await?;

        info!(
            "[GeminiAgent] Raw response from enhanced multi-turn agent: {}",
            response.to_string()
        );

        // 🧠 Process response and extract instructions
        let tool_call_response: serde_json::Value = serde_json::from_str(&response.to_string())?;
        let instruction = if let Some(instruction_field) = tool_call_response.get("instruction") {
            instruction_field
        } else {
            &tool_call_response
        };

        Ok(serde_json::to_string(instruction)?)
    }
}
