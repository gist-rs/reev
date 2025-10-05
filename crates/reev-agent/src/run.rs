use anyhow::Result;
use rig::{
    completion::Prompt,
    prelude::*,
    providers::{gemini, openai::Client},
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

use crate::{
    prompt::SYSTEM_PREAMBLE,
    tools::{JupiterEarnTool, JupiterLendDepositTool, JupiterLendWithdrawTool, JupiterSwapTool},
    LlmRequest,
};

use crate::protocols::{SolTransferTool, SplTransferTool};

/// A minimal struct for deserializing the `key_map` from the `context_prompt` YAML.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AgentContext {
    key_map: HashMap<String, String>,
}

/// Dispatches the request to the appropriate agent based on the model name.
/// It first parses the on-chain context to provide it to the tools that need it.
pub async fn run_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    // If mock is enabled, use deterministic agent instead
    if payload.mock {
        info!("[run_agent] Mock mode enabled, routing to deterministic agent");
        let response = crate::run_deterministic_agent(payload).await?;
        return Ok(serde_json::to_string(&response.0)?);
    }
    // Parse the context_prompt to extract the key_map, which is needed by the JupiterSwapTool
    // to correctly identify mock mints.
    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();

    // If parsing fails, default to an empty map. This allows the agent to function
    // even with a malformed or missing context, though tools needing it may fail.
    let context: AgentContext = serde_yaml::from_str(yaml_str).unwrap_or(AgentContext {
        key_map: HashMap::new(),
    });
    let key_map = context.key_map;

    if model_name.starts_with("gemini") {
        info!("[reev-agent] Using Gemini agent for model: {model_name}");
        run_gemini_agent(model_name, payload, key_map).await
    } else {
        info!("[reev-agent] Using OpenAI compat agent for model: {model_name}");
        run_openai_compatible_agent(model_name, payload, key_map).await
    }
}

/// Runs the AI agent logic using a Google Gemini model.
async fn run_gemini_agent(
    model_name: &str,
    payload: LlmRequest,
    key_map: HashMap<String, String>,
) -> Result<String> {
    let client = gemini::Client::from_env();

    let gen_cfg = gemini::completion::gemini_api_types::GenerationConfig {
        temperature: Some(0.0),
        ..Default::default()
    };
    let cfg =
        gemini::completion::gemini_api_types::AdditionalParameters::default().with_config(gen_cfg);

    // Instantiate the JupiterSwapTool with the context-aware key_map.
    let jupiter_swap_tool = JupiterSwapTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_deposit_tool = JupiterLendDepositTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_withdraw_tool = JupiterLendWithdrawTool {
        key_map: key_map.clone(),
    };
    let jupiter_positions_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };
    let jupiter_earnings_tool = JupiterEarnTool { key_map };

    let agent = client
        .agent(model_name)
        .preamble(SYSTEM_PREAMBLE)
        .additional_params(serde_json::to_value(cfg)?)
        .tool(SolTransferTool)
        .tool(SplTransferTool)
        .tool(jupiter_swap_tool)
        .tool(jupiter_lend_deposit_tool)
        .tool(jupiter_lend_withdraw_tool)
        .tool(jupiter_positions_tool)
        .tool(jupiter_earnings_tool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).await?;
    Ok(response.to_string())
}

/// Runs the AI agent logic using a local lmstudio model locally.
async fn run_openai_compatible_agent(
    model_name: &str,
    payload: LlmRequest,
    key_map: HashMap<String, String>,
) -> Result<String> {
    let client = Client::builder("")
        .base_url("http://localhost:1234/v1")
        .build()?;

    // Instantiate the JupiterSwapTool with the context-aware key_map.
    let jupiter_swap_tool = JupiterSwapTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_deposit_tool = JupiterLendDepositTool {
        key_map: key_map.clone(),
    };
    let jupiter_lend_withdraw_tool = JupiterLendWithdrawTool {
        key_map: key_map.clone(),
    };
    let jupiter_positions_tool = JupiterEarnTool {
        key_map: key_map.clone(),
    };
    let jupiter_earnings_tool = JupiterEarnTool { key_map };

    let agent = client
        .completion_model(model_name)
        .completions_api()
        .into_agent_builder()
        .preamble(SYSTEM_PREAMBLE)
        .tool(SolTransferTool)
        .tool(SplTransferTool)
        .tool(jupiter_swap_tool)
        .tool(jupiter_lend_deposit_tool)
        .tool(jupiter_lend_withdraw_tool)
        .tool(jupiter_positions_tool)
        .tool(jupiter_earnings_tool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).await?;
    Ok(response.to_string())
}
