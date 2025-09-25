use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the structure of the incoming request from the `LlmAgent`.
#[derive(Debug, Deserialize)]
struct LlmRequest {
    prompt: String,
    context_prompt: String,
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

/// A simple parser to extract the `key_map` from the YAML-like context string.
fn extract_key_map_from_context(context: &str) -> HashMap<String, String> {
    let mut key_map = HashMap::new();
    let mut in_key_map_section = false;

    for line in context.lines() {
        if line.trim() == "key_map:" {
            in_key_map_section = true;
            continue;
        }

        if in_key_map_section {
            // Stop if we hit a non-indented line.
            if !line.starts_with("  ") && !line.trim().is_empty() {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                key_map.insert(key, value);
            }
        }
    }
    key_map
}

/// Axum handler for the `POST /gen/tx` endpoint.
///
/// This function simulates the LLM's behavior by returning a hardcoded,
/// valid instruction for a native SOL transfer of 0.1 SOL.
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Axum handler for the `POST /gen/tx` endpoint.
///
/// This function simulates the LLM's behavior by returning a hardcoded,
/// valid instruction for a native SOL transfer of 0.1 SOL.
async fn generate_transaction(Json(payload): Json<LlmRequest>) -> Json<LlmResponse> {
    println!(
        "[reev-agent] Received request for prompt: \"{}\"",
        payload.prompt
    );

    // Extract the actual pubkeys from the context_prompt.
    let key_map = extract_key_map_from_context(&payload.context_prompt);

    // The runner's `LlmAgent` expects real pubkeys in the response, not placeholders.
    let from_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .expect("USER_WALLET_PUBKEY not found in key_map");
    let to_pubkey = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .expect("RECIPIENT_WALLET_PUBKEY not found in key_map");

    // Based on the prompt, decide whether to send a correct or incorrect instruction.
    // This allows us to test both the pass and fail scoring paths.
    let data = if payload.prompt.starts_with("E2E TEST") {
        println!("[reev-agent] Detected E2E pass benchmark. Using correct instruction data.");
        // Correct data for a 0.1 SOL transfer.
        "2Z4dY1Wp2j".to_string()
    } else {
        println!("[reev-agent] Detected original benchmark. Using incorrect instruction data.");
        // Intentionally incorrect data to test the failure case.
        "3Bv62F7i".to_string()
    };

    // Construct the mock RawInstruction for a 0.1 SOL transfer.
    let raw_instruction = RawInstruction {
        program_id: "11111111111111111111111111111111".to_string(),
        accounts: vec![
            RawAccountMeta {
                pubkey: from_pubkey.to_string(),
                is_signer: true,
                is_writable: true,
            },
            RawAccountMeta {
                pubkey: to_pubkey.to_string(),
                is_signer: false,
                is_writable: true,
            },
        ],
        data,
    };

    println!("[reev-agent] Responding with hardcoded SOL transfer instruction.");

    // Wrap the instruction in the nested JSON structure the LlmAgent expects.
    let response = LlmResponse {
        result: LlmResult {
            text: raw_instruction,
        },
    };

    Json(response)
}

/// The main entry point for the mock agent server.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up the Axum router with a single endpoint.
    let app = Router::new()
        .route("/gen/tx", post(generate_transaction))
        .route("/health", get(health_check));

    // Start the server.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090").await?;
    println!("[reev-agent] Mock LLM server listening on http://127.0.0.1:9090");
    println!("[reev-agent] POST /gen/tx is ready to accept requests.");

    axum::serve(listener, app).await?;

    Ok(())
}
