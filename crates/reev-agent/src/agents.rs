use anyhow::Result;
use rig::{
    completion::Prompt,
    prelude::*,
    providers::{gemini, openai::Client},
};
use tracing::info;

use crate::{
    tools::{JupiterSwapTool, SolTransferTool, SplTransferTool},
    LlmRequest,
};

const SYSTEM_PREAMBLE: &str = "You are a helpful Solana assistant. Your goal is to generate a single, valid Solana transaction instruction in JSON format.
- Analyze the user's request and on-chain context.
- You MUST call a tool, and you MUST only call it ONCE.
- Select the correct tool (`sol_transfer`, `spl_transfer`, or `jupiter_swap`) and provide its parameters.
- The tool will return a JSON object.
- Your final output MUST be ONLY the raw JSON from the tool, starting with `{` and ending with `}`. Do not include `json` block quotes or any other text.";

/// Dispatches the request to the appropriate agent based on the model name.
pub async fn run_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    if model_name.starts_with("gemini") {
        info!("[reev-agent] Using Gemini agent for model: {model_name}");
        run_gemini_agent(model_name, payload).await
    } else {
        info!("[reev-agent] Using OpenAI compat agent for model: {model_name}");
        run_openai_compatible_agent(model_name, payload).await
    }
}

/// Runs the AI agent logic using a Google Gemini model.
async fn run_gemini_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    let client = gemini::Client::from_env();

    let gen_cfg = gemini::completion::gemini_api_types::GenerationConfig {
        temperature: Some(0.0),
        ..Default::default()
    };
    let cfg =
        gemini::completion::gemini_api_types::AdditionalParameters::default().with_config(gen_cfg);

    let agent = client
        .agent(model_name)
        .preamble(SYSTEM_PREAMBLE)
        .additional_params(serde_json::to_value(cfg)?)
        .tool(SolTransferTool)
        .tool(SplTransferTool)
        .tool(JupiterSwapTool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).await?;
    Ok(response.to_string())
}

/// Runs the AI agent logic using a local lmstudio model locally.
async fn run_openai_compatible_agent(model_name: &str, payload: LlmRequest) -> Result<String> {
    let client = Client::builder("")
        .base_url("http://localhost:1234/v1")
        .build()?;

    let agent = client
        .completion_model(model_name)
        .completions_api()
        .into_agent_builder()
        .preamble(SYSTEM_PREAMBLE)
        .tool(SolTransferTool)
        .tool(SplTransferTool)
        .tool(JupiterSwapTool)
        .build();

    let full_prompt = format!(
        "{}\n\nUSER REQUEST: {}",
        payload.context_prompt, payload.prompt
    );

    let response = agent.prompt(&full_prompt).await?;
    Ok(response.to_string())
}
