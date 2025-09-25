use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, system_instruction};
use std::collections::HashMap;
use std::str::FromStr;

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

    // Based on the prompt, decide whether to generate a correct instruction in code
    // or return an incorrect one.
    let raw_instruction = if payload.prompt == "Please send 0.1 SOL from my wallet (USER_WALLET_PUBKEY) to the recipient (RECIPIENT_WALLET_PUBKEY)." {
        println!("[reev-agent] Detected '001-sol-transfer' prompt. Generating instruction with code.");

        // 1. Parse pubkeys
        let from = Pubkey::from_str(from_pubkey).expect("Failed to parse from_pubkey");
        let to = Pubkey::from_str(to_pubkey).expect("Failed to parse to_pubkey");
        let lamports = 100_000_000; // 0.1 SOL

        // 2. Generate instruction using solana_sdk
        let instruction = system_instruction::transfer(&from, &to, lamports);
        println!("[reev-agent] Generated instruction: {:?}", instruction);

        // 3. Convert back to RawInstruction for the response
        RawInstruction {
            program_id: instruction.program_id.to_string(),
            accounts: instruction.accounts.iter().map(|acc| RawAccountMeta {
                pubkey: acc.pubkey.to_string(),
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            }).collect(),
            data: bs58::encode(instruction.data).into_string(),
        }
    } else {
        println!("[reev-agent] Prompt did not match. Sending intentionally invalid instruction.");
        // Return an invalid instruction for any other case to test failures.
        RawInstruction {
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
            data: "invalid".to_string(),
        }
    };

    println!("[reev-agent] Responding with instruction.");

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
