use anyhow::Result;
use rig::{completion::Prompt, prelude::*, providers::gemini};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

use crate::{
    enhanced::enhanced_context::EnhancedContextAgent,
    flow::{create_flow_infrastructure, GlobalFlowTracker},
    prompt::SYSTEM_PREAMBLE,
    tools::{
        AccountBalanceTool, JupiterEarnTool, JupiterLendEarnDepositTool, JupiterLendEarnMintTool,
        JupiterLendEarnRedeemTool, JupiterLendEarnWithdrawTool, JupiterSwapTool,
        LendEarnTokensTool, PositionInfoTool, SolTransferTool, SplTransferTool,
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
        let (context_text, _depth_score, _has_positions) =
            EnhancedContextAgent::build_context(&payload, &key_map);
        let enhanced_prompt = format!("{SYSTEM_PREAMBLE}\n\n---\n{context_text}\n---");

        // 🤖 MULTI-TURN CONVERSATION: Enable step-by-step reasoning
        let user_request = format!(
            "{}\n\nUSER REQUEST: {}",
            payload.context_prompt, payload.prompt
        );

        // 🛠️ Create flow tracking infrastructure
        let _flow_tracker = create_flow_infrastructure();

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
        let jupiter_positions_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };
        let jupiter_earnings_tool = JupiterEarnTool {
            key_map: key_map.clone(),
        };

        // 🔍 DISCOVERY TOOLS: Enable prerequisite validation when context is insufficient
        let balance_tool = AccountBalanceTool {
            key_map: key_map.clone(),
        };
        let position_tool = PositionInfoTool {
            key_map: key_map.clone(),
        };
        let lend_earn_tokens_tool = LendEarnTokensTool::new(key_map.clone());

        // 🧠 Build enhanced multi-turn agent
        let agent = client
            .agent(model_name)
            .preamble(&enhanced_prompt)
            .additional_params(serde_json::to_value(cfg)?)
            .tool(sol_tool)
            .tool(spl_tool)
            .tool(jupiter_swap_tool)
            .tool(jupiter_lend_earn_deposit_tool)
            .tool(jupiter_lend_earn_withdraw_tool)
            .tool(jupiter_lend_earn_mint_tool)
            .tool(jupiter_lend_earn_redeem_tool)
            .tool(jupiter_positions_tool)
            .tool(jupiter_earnings_tool)
            .tool(balance_tool)
            .tool(position_tool)
            .tool(lend_earn_tokens_tool)
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

        // 🎯 EXTRACT FLOW DATA FROM GLOBAL TRACKER
        let flow_data = GlobalFlowTracker::get_flow_data();

        // 🎯 FORMAT COMPREHENSIVE RESPONSE WITH FLOWS
        let mut comprehensive_response = json!({
            "transactions": [],
            "summary": instruction.to_string(),
            "signatures": []
        });

        // Add flow data if available
        if let Some(flows) = flow_data {
            comprehensive_response["flows"] = json!(flows);
            info!(
                "[GeminiAgent] Flow data captured: {} tool calls",
                flows.total_tool_calls
            );
        }

        Ok(serde_json::to_string(&comprehensive_response)?)
    }
}
