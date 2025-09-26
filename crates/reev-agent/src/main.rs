use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use reev_lib::agent::{RawAccountMeta, RawInstruction};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

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
async fn generate_transaction(
    Query(params): Query<MockParams>,
    Json(payload): Json<LlmRequest>,
) -> Json<LlmResponse> {
    if params.mock {
        return mock_generate_transaction(payload).await;
    }

    // For now, default to mock as well.
    mock_generate_transaction(payload).await
}

/// Axum handler for the `POST /gen/tx` endpoint.
///
/// This function simulates the LLM's behavior by returning a hardcoded,
/// valid instruction for a native SOL transfer of 0.1 SOL.
async fn mock_generate_transaction(payload: LlmRequest) -> Json<LlmResponse> {
    info!(
        "[reev-agent] Received request for prompt: \"{}\"",
        payload.prompt
    );

    // Parse the context_prompt YAML to safely extract the key_map.
    let yaml_str = payload
        .context_prompt
        .trim_start_matches("---\n\nCURRENT ON-CHAIN CONTEXT:\n")
        .trim_end_matches("\n\n\n---")
        .trim();
    let context: AgentContext =
        serde_yaml::from_str(yaml_str).expect("Failed to parse context_prompt YAML");
    let key_map = context.key_map;

    // Based on the prompt, decide whether to generate a correct instruction in code
    // or return an incorrect one.
    let raw_instruction = if payload.prompt.contains("0.1 SOL") {
        info!("[reev-agent] Detected 'sol-transfer' prompt. Generating instruction with code.");

        // 1. Parse pubkeys
        let from_pubkey = key_map
            .get("USER_WALLET_PUBKEY")
            .expect("USER_WALLET_PUBKEY not found in key_map");
        let to_pubkey = key_map
            .get("RECIPIENT_WALLET_PUBKEY")
            .expect("RECIPIENT_WALLET_PUBKEY not found in key_map");
        let from = Pubkey::from_str(from_pubkey).expect("Failed to parse from_pubkey");
        let to = Pubkey::from_str(to_pubkey).expect("Failed to parse to_pubkey");
        let lamports = 100_000_000; // 0.1 SOL

        // 2. Generate instruction using solana_sdk
        let instruction = system_instruction::transfer(&from, &to, lamports);
        info!("[reev-agent] Generated instruction: {instruction:?}");

        // 3. Convert back to RawInstruction for the response
        RawInstruction {
            program_id: instruction.program_id.to_string(),
            accounts: instruction
                .accounts
                .iter()
                .map(|acc| RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: bs58::encode(instruction.data).into_string(),
        }
    } else if payload.prompt.contains("15 USDC") {
        info!(
            "[reev-agent] Detected 'spl-token-transfer' prompt. Generating instruction with code."
        );

        // 1. Parse pubkeys from context
        let source_ata_str = key_map
            .get("USER_USDC_ATA")
            .expect("USER_USDC_ATA not found in key_map");
        let dest_ata_str = key_map
            .get("RECIPIENT_USDC_ATA")
            .expect("RECIPIENT_USDC_ATA not found in key_map");
        let authority_str = key_map
            .get("USER_WALLET_PUBKEY")
            .expect("USER_WALLET_PUBKEY not found in key_map");

        let source_pubkey =
            Pubkey::from_str(source_ata_str).expect("Failed to parse source ATA pubkey");
        let destination_pubkey =
            Pubkey::from_str(dest_ata_str).expect("Failed to parse destination ATA pubkey");
        let authority_pubkey =
            Pubkey::from_str(authority_str).expect("Failed to parse authority pubkey");

        let amount = 15_000_000; // 15 USDC with 6 decimals

        // 2. Generate instruction using spl_token sdk
        let instruction = spl_token::instruction::transfer(
            &spl_token::id(),
            &source_pubkey,
            &destination_pubkey,
            &authority_pubkey,
            &[&authority_pubkey],
            amount,
        )
        .expect("Failed to create SPL transfer instruction");
        info!("[reev-agent] Generated instruction: {instruction:?}");

        // 3. Convert back to RawInstruction for the response
        RawInstruction {
            program_id: instruction.program_id.to_string(),
            accounts: instruction
                .accounts
                .iter()
                .map(|acc| RawAccountMeta {
                    pubkey: acc.pubkey.to_string(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data: bs58::encode(instruction.data).into_string(),
        }
    } else {
        info!("[reev-agent] Prompt did not match. Sending intentionally invalid instruction.");
        // Return an invalid instruction for any other case to test failures.
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
    info!("[reev-agent] Mock LLM server listening on http://127.0.0.1:9090");
    info!("[reev-agent] POST /gen/tx is ready to accept requests.");

    axum::serve(listener, app).await?;

    Ok(())
}
