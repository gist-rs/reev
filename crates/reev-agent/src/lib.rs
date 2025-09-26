use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reev_lib::agent::{RawAccountMeta, RawInstruction};

use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info};

pub mod tools;

mod agents;

/// Represents the structure of the incoming request from the `LlmAgent`.
#[derive(Debug, Deserialize)]
pub struct LlmRequest {
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

    let raw_instruction: RawInstruction = serde_json::from_str(&response_str)
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
        "[reev-agent] Received request for prompt: \"{}\"",
        payload.prompt
    );

    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();
    let context: AgentContext =
        serde_yaml::from_str(yaml_str).context("Failed to parse context_prompt YAML")?;
    let key_map = context.key_map;

    let raw_instruction = if payload.prompt.contains("0.1 SOL") {
        info!("[reev-agent] Detected 'sol-transfer' prompt. Generating instruction with code.");
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
    } else if payload.prompt.contains("15 USDC") {
        info!(
            "[reev-agent] Detected 'spl-token-transfer' prompt. Generating instruction with code."
        );
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
    } else {
        info!("[reev-agent] Prompt did not match. Sending intentionally invalid instruction.");
        let from_pubkey = key_map
            .get("USER_WALLET_PUBKEY")
            .cloned()
            .unwrap_or_else(|| "USER_WALLET_PUBKEY_NOT_FOUND".to_string());
        let to_pubkey = key_map
            .get("RECIPIENT_WALLET_PUBKEY")
            .cloned()
            .unwrap_or_else(|| "RECIPIENT_WALLET_PUBKEY_NOT_FOUND".to_string());
        RawInstruction {
            program_id: "11111111111111111111111111111111".to_string(),
            accounts: vec![
                RawAccountMeta {
                    pubkey: from_pubkey,
                    is_signer: true,
                    is_writable: true,
                },
                RawAccountMeta {
                    pubkey: to_pubkey,
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: "invalid".to_string(),
        }
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
