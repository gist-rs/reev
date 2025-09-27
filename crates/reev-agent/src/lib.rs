use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use reev_lib::agent::RawInstruction;
use serde::{Deserialize, Serialize};
use serde_json::json;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info};

pub mod jupiter;
pub mod tools;

mod agents;
mod prompt;

/// Represents the structure of the incoming request from the `LlmAgent`.
#[derive(Debug, Deserialize)]
pub struct LlmRequest {
    pub id: String,
    pub prompt: String,
    pub context_prompt: String,
    #[serde(default = "default_model")]
    pub model_name: String,
}

fn default_model() -> String {
    "qwen3-coder-30b-a3b-instruct-mlx".to_string()
}

/// The `text` field of the response, containing the raw instruction.
#[derive(Debug, Serialize)]
struct LlmResult {
    text: RawInstruction,
}

/// The top-level response structure, mirroring what the real LLM service would send.
#[derive(Debug, Serialize)]
struct LlmResponse {
    result: LlmResult,
}

/// Structs for deserializing the `context_prompt` YAML.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AgentContext {
    key_map: HashMap<String, String>,
}

/// Parameters for enabling mock transaction generation.
#[derive(Debug, Deserialize)]
struct MockParams {
    #[serde(default)]
    mock: bool,
}

/// Axum handler for the `GET /health` endpoint.
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Axum handler for the `POST /gen/tx` endpoint.
///
/// This function routes the request to either the deterministic agent or the AI agent
/// based on the `mock` query parameter.
async fn generate_transaction(
    Query(params): Query<MockParams>,
    Json(payload): Json<LlmRequest>,
) -> Response {
    let result = if params.mock {
        info!("[reev-agent] Routing to Deterministic Agent (mock=true).");
        run_deterministic_agent(payload).await
    } else {
        info!("[reev-agent] Routing to AI Agent.");
        run_ai_agent(payload).await
    };

    match result {
        Ok(json_response) => (StatusCode::OK, json_response).into_response(),
        Err(_e) => {
            // The error from `rig` can cause a stack overflow when formatted.
            // The detailed error is logged inside `run_ai_agent`.
            // This handler just returns a generic, safe response to prevent a crash.
            let error_msg = "Internal agent error. See agent logs for details.".to_string();
            info!("[reev-agent] Agent returned an error. Sending 500 response.");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": error_msg })),
            )
                .into_response()
        }
    }
}

/// Executes the AI agent logic using the dynamically selected model.
async fn run_ai_agent(payload: LlmRequest) -> Result<Json<LlmResponse>> {
    let model_name = payload.model_name.clone();

    let response_str = agents::run_agent(&model_name, payload).await.map_err(|e| {
        error!("[reev-agent] Agent failed. Detailed Error: {e:?}");
        e
    })?;

    info!("[reev-agent] Raw response from agent tool call: {response_str}");

    // Clean the response: trim whitespace and remove markdown code blocks.
    let cleaned_response = response_str
        .trim()
        .strip_prefix("```json")
        .unwrap_or(&response_str)
        .strip_suffix("```")
        .unwrap_or(&response_str)
        .trim();

    let raw_instruction: RawInstruction = serde_json::from_str(cleaned_response)
        .context("Failed to deserialize RawInstruction from AI agent tool response")?;

    let response = LlmResponse {
        result: LlmResult {
            text: raw_instruction,
        },
    };

    Ok(Json(response))
}

/// Executes the deterministic, code-based agent logic to generate a ground truth instruction.
async fn run_deterministic_agent(payload: LlmRequest) -> Result<Json<LlmResponse>> {
    info!(
        "[reev-agent] 🦀 Received request for benchmark id: \"{}\"",
        payload.id
    );

    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();
    let context: AgentContext =
        serde_yaml::from_str(yaml_str).context("Failed to parse context_prompt YAML")?;
    let key_map = context.key_map;

    let raw_instruction = if payload.id == "001-SOL-TRANSFER" {
        info!("[reev-agent] Matched '001-SOL-TRANSFER' id. Generating instruction with code.");
        let from_pubkey = key_map
            .get("USER_WALLET_PUBKEY")
            .context("USER_WALLET_PUBKEY not found in key_map")?;
        let to_pubkey = key_map
            .get("RECIPIENT_WALLET_PUBKEY")
            .context("RECIPIENT_WALLET_PUBKEY not found in key_map")?;
        let from = Pubkey::from_str(from_pubkey).context("Failed to parse from_pubkey")?;
        let to = Pubkey::from_str(to_pubkey).context("Failed to parse to_pubkey")?;
        let lamports = 100_000_000;
        let instruction = system_instruction::transfer(&from, &to, lamports);
        info!("[reev-agent] Generated instruction: {instruction:?}");
        instruction.into()
    } else if payload.id == "002-SPL-TRANSFER" {
        info!("[reev-agent] Matched '002-SPL-TRANSFER' id. Generating instruction with code.");
        let source_ata_str = key_map
            .get("USER_USDC_ATA")
            .context("USER_USDC_ATA not found in key_map")?;
        let dest_ata_str = key_map
            .get("RECIPIENT_USDC_ATA")
            .context("RECIPIENT_USDC_ATA not found in key_map")?;
        let authority_str = key_map
            .get("USER_WALLET_PUBKEY")
            .context("USER_WALLET_PUBKEY not found in key_map")?;
        let source_pubkey =
            Pubkey::from_str(source_ata_str).context("Failed to parse source ATA pubkey")?;
        let destination_pubkey =
            Pubkey::from_str(dest_ata_str).context("Failed to parse destination ATA pubkey")?;
        let authority_pubkey =
            Pubkey::from_str(authority_str).context("Failed to parse authority pubkey")?;
        let amount = 15_000_000;
        let instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &source_pubkey,
            &destination_pubkey,
            &authority_pubkey,
            &[&authority_pubkey],
            amount,
        )
        .context("Failed to create SPL transfer instruction")?;
        info!("[reev-agent] Generated instruction: {instruction:?}");
        instruction.into()
    } else if payload.id == "100-JUP-SWAP-SOL-USDC" {
        info!("[reev-agent] Matched '100-JUP-SWAP-SOL-USDC' id. Generating instruction with code.");
        let user_pubkey_str = key_map
            .get("USER_WALLET_PUBKEY")
            .context("USER_WALLET_PUBKEY not found in key_map")?;
        let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

        // For the deterministic case, the prompt hardcodes the swap details.
        // We provide the known mock mint from the context, and `handle_jupiter_swap` will do the replacement.
        let input_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        let output_mint_str = key_map
            .get("MOCK_USDC_MINT")
            .context("MOCK_USDC_MINT not found in key_map")?;
        let output_mint = Pubkey::from_str(output_mint_str)?;
        let amount = 100_000_000; // 0.1 SOL specified in the prompt
        let slippage_bps = 50; // Default slippage

        jupiter::swap::handle_jupiter_swap(
            user_pubkey,
            input_mint,
            output_mint,
            amount,
            slippage_bps,
            &key_map,
        )
        .await?
    } else {
        anyhow::bail!(
            "Deterministic agent does not support this id: '{}'",
            payload.id
        );
    };

    info!("[reev-agent] Responding with instruction.");
    let response = LlmResponse {
        result: LlmResult {
            text: raw_instruction,
        },
    };

    Ok(Json(response))
}

/// The main entry point for the mock agent server.
pub async fn run_server() -> anyhow::Result<()> {
    // Load environment variables from a .env file, if present.
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/gen/tx", post(generate_transaction))
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090").await?;
    info!("[reev-agent] Mock LLM server listening on http://127.0.0.1:9090");
    info!("[reev-agent] POST /gen/tx is ready to accept requests.");

    axum::serve(listener, app).await?;

    Ok(())
}
