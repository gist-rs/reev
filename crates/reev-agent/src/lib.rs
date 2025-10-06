use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

pub mod flow;
pub mod protocols;
pub mod run;
pub mod tools;

mod agents;
pub mod common;
mod prompt;

#[derive(Debug, Deserialize)]
pub struct LlmRequest {
    pub id: String,
    pub prompt: String,
    pub context_prompt: String,
    #[serde(default = "default_model")]
    pub model_name: String,
    #[serde(default)]
    pub mock: bool,
}

fn default_model() -> String {
    "default-model".to_string()
}

/// The `text` field of the response, containing the JSON string of the instruction(s).
#[derive(Debug, Serialize)]
struct LlmResult {
    text: String,
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
    // Allow mock to be set via query param or request body
    let mock_enabled = params.mock || payload.mock;

    let result = if mock_enabled {
        info!("[reev-agent] Routing to Deterministic Agent (mock=true).");
        run_deterministic_agent(payload).await
    } else {
        info!("[reev-agent] Routing to AI Agent.");
        run_ai_agent(payload).await
    };

    match result {
        Ok(json_response) => (StatusCode::OK, json_response).into_response(),
        Err(e) => {
            let error_msg = format!("Internal agent error: {e}");
            error!(
                "[reev-agent] Agent returned an error: {}. Sending 500 response.",
                error_msg
            );
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

    let response_str = run::run_agent(&model_name, payload).await.map_err(|e| {
        error!("[reev-agent] AI Agent failed. Detailed Error: {e:?}");
        e
    })?;

    info!("[reev-agent] Raw response from AI agent tool call: {response_str}");

    // Use regex to find a JSON block. This is more robust for models that
    // wrap their output in conversational text or markdown.
    let re = Regex::new(r"(?s)```(?:json)?\s*(\{[\s\S]*?\}|\[[\s\S]*?\])\s*```").unwrap();
    let extracted_json = if let Some(caps) = re.captures(&response_str) {
        caps.get(1).map_or("", |m| m.as_str()).to_string()
    } else {
        // If no markdown block is found, assume the whole response is the JSON string.
        response_str
    };

    let cleaned_response = extracted_json.trim().to_string();

    // Validate the response is valid JSON, but pass the string through.
    let _: serde_json::Value = serde_json::from_str(&cleaned_response)
        .context("Failed to validate AI agent response as parseable JSON")?;

    let response = LlmResponse {
        result: LlmResult {
            text: cleaned_response,
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

    // The coding agents return one or more instructions. We serialize the result
    // into a JSON string to match the format expected by the runner.
    let instructions_json = match payload.id.as_str() {
        "001-SOL-TRANSFER" => {
            let ixs = agents::coding::d_001_sol_transfer::handle_sol_transfer(&key_map).await?;
            serde_json::to_string(&ixs)?
        }
        "002-SPL-TRANSFER" => {
            let ixs = agents::coding::d_002_spl_transfer::handle_spl_transfer(&key_map).await?;
            serde_json::to_string(&ixs)?
        }
        "100-JUP-SWAP-SOL-USDC" => {
            let ixs =
                agents::coding::d_100_jup_swap_sol_usdc::handle_jup_swap_sol_usdc(&key_map).await?;
            serde_json::to_string(&ixs)?
        }
        "110-JUP-LEND-DEPOSIT-SOL" => {
            let ixs =
                agents::coding::d_110_jup_lend_deposit_sol::handle_jup_lend_deposit_sol(&key_map)
                    .await?;
            serde_json::to_string(&ixs)?
        }
        "111-JUP-LEND-DEPOSIT-USDC" => {
            let ixs =
                agents::coding::d_111_jup_lend_deposit_usdc::handle_jup_lend_deposit_usdc(&key_map)
                    .await?;
            serde_json::to_string(&ixs)?
        }
        "112-JUP-LEND-WITHDRAW-SOL" => {
            let ixs =
                agents::coding::d_112_jup_lend_withdraw_sol::handle_jup_lend_withdraw_sol(&key_map)
                    .await?;
            serde_json::to_string(&ixs)?
        }
        "113-JUP-LEND-WITHDRAW-USDC" => {
            let ixs = agents::coding::d_113_jup_lend_withdraw_usdc::handle_jup_lend_withdraw_usdc(
                &key_map,
            )
            .await?;
            serde_json::to_string(&ixs)?
        }
        "114-JUP-POSITIONS-AND-EARNINGS" => {
            info!(
                "[reev-agent] 🦀 Received request for benchmark id: \"{}\" - Deterministic Jupiter Positions and Earnings Flow",
                payload.id
            );
            let response =
                agents::coding::d_114_jup_positions_and_earnings::handle_jup_positions_and_earnings(
                    &key_map,
                )
                .await?;
            let response_json = serde_json::to_string(&response)?;
            info!(
                "[reev-agent] Successfully created deterministic response with {} total positions and ${:.2} in earnings",
                response["step_1_result"]["total_positions"],
                response["summary"]["total_earnings_usd"].as_f64().unwrap_or(0.0)
            );
            response_json
        }
        // Handle flow benchmarks (IDs starting with "200-")
        flow_id if flow_id.starts_with("200-") => {
            info!(
                "[reev-agent] 🦀 Received flow benchmark request: \"{}\" - Creating deterministic flow response",
                payload.id
            );
            // For flow benchmarks, create a mock multi-step response
            let flow_response = serde_json::json!({
                "flow_completed": true,
                "total_steps": 2,
                "completed_steps": 2,
                "status": "success",
                "steps": [
                    {
                        "step": 1,
                        "description": "Swap SOL to USDC using Jupiter",
                        "status": "success",
                        "instructions": [
                            {
                                "program_id": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
                                "accounts": [
                                    {"pubkey": &key_map.get("USER_WALLET_PUBKEY").unwrap_or(&"unknown".to_string()), "is_signer": true, "is_writable": true},
                                    {"pubkey": "So11111111111111111111111111111111111111112", "is_signer": false, "is_writable": false},
                                    {"pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "is_signer": false, "is_writable": false}
                                ],
                                "data": "swap_500000000"
                            }
                        ]
                    },
                    {
                        "step": 2,
                        "description": "Deposit USDC into Jupiter lending",
                        "status": "success",
                        "instructions": [
                            {
                                "program_id": "jup3YeL8QhtSx1e253b2FDvsMNC87fDrgQZivbrndc9",
                                "accounts": [
                                    {"pubkey": &key_map.get("USER_WALLET_PUBKEY").unwrap_or(&"unknown".to_string()), "is_signer": true, "is_writable": true},
                                    {"pubkey": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "is_signer": false, "is_writable": false}
                                ],
                                "data": "deposit_500000000"
                            }
                        ]
                    }
                ],
                "summary": {
                    "total_instructions": 2,
                    "estimated_gas": 5000000,
                    "estimated_time_seconds": 15
                }
            });
            serde_json::to_string(&flow_response)?
        }
        _ => anyhow::bail!("Coding agent does not support this id: '{}'", payload.id),
    };

    info!(
        "[reev-agent] Responding with instructions: {}",
        instructions_json
    );
    let response = LlmResponse {
        result: LlmResult {
            text: instructions_json,
        },
    };

    Ok(Json(response))
}

/// The main entry point for the mock agent server.
pub async fn run_server() -> anyhow::Result<()> {
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
