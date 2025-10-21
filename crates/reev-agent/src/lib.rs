use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reev_lib::constants::{
    usdc, usdc_mint, EIGHT_PERCENT, FIVE_PERCENT, SOL_SWAP_AMOUNT, SOL_SWAP_AMOUNT_MEDIUM,
    USDC_LEND_AMOUNT, USDC_LEND_AMOUNT_LARGE, USDC_MINT_AMOUNT,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use spl_token::native_mint;
use std::{collections::HashMap, str::FromStr};
use tracing::{error, info};

/// Handle simple transfer benchmarks (001-004 series)
async fn handle_simple_transfer_benchmarks(
    benchmark_id: &str,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    match benchmark_id {
        "001-sol-transfer" => {
            let ixs = agents::coding::d_001_sol_transfer::handle_sol_transfer(key_map).await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "002-spl-transfer" => {
            let ixs = agents::coding::d_002_spl_transfer::handle_spl_transfer(key_map).await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "003-spl-transfer-fail" => {
            let ixs =
                agents::coding::d_003_spl_transfer_fail::handle_spl_transfer_fail(key_map).await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "004-partial-score-spl-transfer" => {
            let ixs = agents::coding::d_004_partial_score_spl_transfer::handle_partial_score_spl_transfer(
                key_map,
            )
            .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        _ => anyhow::bail!("Not a simple transfer benchmark: {benchmark_id}"),
    }
}

/// Handle Jupiter swap benchmarks (100 series)
async fn handle_jupiter_swap_benchmarks(
    benchmark_id: &str,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    match benchmark_id {
        "100-jup-swap-sol-usdc" => {
            let ixs =
                agents::coding::d_100_jup_swap_sol_usdc::handle_jup_swap_sol_usdc(key_map).await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        _ => anyhow::bail!("Not a Jupiter swap benchmark: {benchmark_id}"),
    }
}

/// Handle Jupiter lending benchmarks (110-116 series)
async fn handle_jupiter_lending_benchmarks(
    benchmark_id: &str,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    match benchmark_id {
        "110-jup-lend-deposit-sol" => {
            let ixs =
                agents::coding::d_110_jup_lend_deposit_sol::handle_jup_lend_deposit_sol(key_map)
                    .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "111-jup-lend-deposit-usdc" => {
            let ixs =
                agents::coding::d_111_jup_lend_deposit_usdc::handle_jup_lend_deposit_usdc(key_map)
                    .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "112-jup-lend-withdraw-sol" => {
            let ixs =
                agents::coding::d_112_jup_lend_withdraw_sol::handle_jup_lend_withdraw_sol(key_map)
                    .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "113-jup-lend-withdraw-usdc" => {
            let ixs = agents::coding::d_113_jup_lend_withdraw_usdc::handle_jup_lend_withdraw_usdc(
                key_map,
            )
            .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "114-jup-positions-and-earnings" => {
            info!(
                "[reev-agent] Received request for benchmark id: \"{}\" - Deterministic Jupiter Positions and Earnings Flow",
                benchmark_id
            );
            let response =
                agents::coding::d_114_jup_positions_and_earnings::handle_jup_positions_and_earnings(
                    key_map,
                )
                .await?;
            let response_json = serde_json::to_string(&response)?;
            info!(
                "[reev-agent] Successfully created deterministic response with {} total positions and ${:.2} in earnings",
                response["step_1_result"]["total_positions"],
                response["summary"]["total_earnings_usd"].as_f64().unwrap_or(0.0)
            );
            Ok(response_json)
        }
        "115-jup-lend-mint-usdc" => {
            let ixs = agents::coding::d_115_jup_lend_mint_usdc::handle_jupiter_mint(
                &usdc_mint(),
                USDC_MINT_AMOUNT,
                key_map,
            )
            .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        "116-jup-lend-redeem-usdc" => {
            let ixs = agents::coding::d_116_jup_lend_redeem_usdc::handle_jupiter_redeem(
                &usdc_mint(),
                usdc::FORTY,
                key_map,
            )
            .await?;
            Ok(serde_json::to_string(&ixs)?)
        }
        _ => anyhow::bail!("Not a Jupiter lending benchmark: {benchmark_id}"),
    }
}

/// Handle flow benchmarks (200 series and multi-step flows)
async fn handle_flow_benchmarks(
    benchmark_id: &str,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    match benchmark_id {
        "200-jup-swap-then-lend-deposit" => {
            info!(
                "[reev-agent] Matched '200-jup-swap-then-lend-deposit' id. Starting deterministic flow."
            );

            let user_pubkey_str = key_map
                .get("USER_WALLET_PUBKEY")
                .context("USER_WALLET_PUBKEY not found in key_map")?;
            let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

            // Step 1: Swap 0.1 SOL to USDC using Jupiter
            info!("[reev-agent] Step 1: Swapping 0.1 SOL to USDC");
            let input_mint = native_mint::ID;
            let output_mint = usdc_mint();
            let swap_amount = SOL_SWAP_AMOUNT; // 0.1 SOL
            let slippage_bps = FIVE_PERCENT; // 5%

            let swap_instructions = handle_jupiter_swap(
                user_pubkey,
                input_mint,
                output_mint,
                swap_amount,
                slippage_bps,
            )
            .await?;

            info!(
                "[reev-agent] Step 1 completed: {} swap instructions generated",
                swap_instructions.len()
            );

            // Step 2: Deposit received USDC into Jupiter lending
            info!("[reev-agent] Step 2: Depositing USDC into Jupiter lending");

            // For lending, we use the USDC mint and deposit the expected amount from the swap
            // Note: In a real scenario, we'd calculate the exact amount received from the swap
            // For deterministic purposes, we estimate ~0.5 SOL worth of USDC (accounting for slippage)
            let deposit_amount = USDC_LEND_AMOUNT_LARGE; // ~9 USDC (accounting for slippage and fees)
            let usdc_mint = usdc_mint();

            let lend_instructions =
                handle_jupiter_lend_deposit(user_pubkey, usdc_mint, deposit_amount).await?;

            info!(
                "[reev-agent] Step 2 completed: {} lending instructions generated",
                lend_instructions.len()
            );

            // Combine all instructions for the complete flow
            let mut all_instructions = Vec::new();
            all_instructions.extend(swap_instructions);
            all_instructions.extend(lend_instructions);

            // Create flow response
            let flow_response = serde_json::json!({
                "benchmark_id": "200-jup-swap-then-lend-deposit",
                "agent_type": "deterministic",
                "steps": [
                    {
                        "step_id": "1",
                        "description": "Swap 0.1 SOL to USDC using Jupiter",
                        "instructions": all_instructions,
                        "estimated_time_seconds": 10
                    },
                    {
                        "step_id": "2",
                        "description": "Deposit received USDC into Jupiter lending",
                        "instructions": [],
                        "estimated_time_seconds": 15
                    }
                ]
            });
            Ok(serde_json::to_string(&flow_response)?)
        }
        // Handle other flow benchmarks (IDs starting with "200-")
        flow_id if flow_id.starts_with("200-") => {
            // Generic flow handler for other 200-series benchmarks
            let flow_response = serde_json::json!({
                "benchmark_id": benchmark_id,
                "agent_type": "deterministic",
                "steps": [
                    {
                        "step_id": "1",
                        "description": format!("Handling flow benchmark: {}", benchmark_id),
                        "instructions": [],
                        "estimated_time_seconds": 15
                    }
                ]
            });
            Ok(serde_json::to_string(&flow_response)?)
        }
        _ => anyhow::bail!("Not a flow benchmark: {benchmark_id}"),
    }
}

/// Handle flow step benchmarks (multi-step flows)
async fn handle_flow_step_benchmarks(
    benchmark_id: &str,
    key_map: &HashMap<String, String>,
) -> Result<String> {
    match benchmark_id {
        flow_id if flow_id.contains("200-jup-swap-then-lend-deposit-step-1") => {
            info!("[reev-agent] Handling flow step 1: Jupiter SOL to USDC swap");
            let user_pubkey_str = key_map
                .get("USER_WALLET_PUBKEY")
                .context("USER_WALLET_PUBKEY not found in key_map")?;
            let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

            let input_mint = native_mint::ID;
            let output_mint = usdc_mint();
            let amount = SOL_SWAP_AMOUNT_MEDIUM; // 0.5 SOL for step 1
            let slippage_bps = EIGHT_PERCENT; // 8%

            let instructions =
                handle_jupiter_swap(user_pubkey, input_mint, output_mint, amount, slippage_bps)
                    .await?;

            info!(
                "[reev-agent] Step 1: Successfully generated {} Jupiter swap instructions",
                instructions.len()
            );
            Ok(serde_json::to_string(&instructions)?)
        }
        flow_id if flow_id.contains("200-jup-swap-then-lend-deposit-step-2") => {
            info!("[reev-agent] Handling flow step 2: Jupiter USDC lending deposit");
            let user_pubkey_str = key_map
                .get("USER_WALLET_PUBKEY")
                .context("USER_WALLET_PUBKEY not found in key_map")?;
            let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

            let usdc_mint = usdc_mint();
            let deposit_amount = USDC_LEND_AMOUNT; // 10 USDC

            let instructions =
                handle_jupiter_lend_deposit(user_pubkey, usdc_mint, deposit_amount).await?;

            info!(
                "[reev-agent] Step 2: Successfully generated {} Jupiter lending instructions",
                instructions.len()
            );
            Ok(serde_json::to_string(&instructions)?)
        }
        flow_id if flow_id.contains("116-jup-lend-redeem-usdc-step-1") => {
            info!("[reev-agent] Handling flow step 1: Jupiter USDC mint (deposit)");
            let user_pubkey_str = key_map
                .get("USER_WALLET_PUBKEY")
                .context("USER_WALLET_PUBKEY not found in key_map")?;
            let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

            let usdc_mint = usdc_mint();
            let deposit_amount = USDC_MINT_AMOUNT; // 50 USDC for step 1

            let instructions =
                handle_jupiter_lend_deposit(user_pubkey, usdc_mint, deposit_amount).await?;

            info!(
                "[reev-agent] Step 1: Successfully generated {} Jupiter lending mint instructions",
                instructions.len()
            );
            Ok(serde_json::to_string(&instructions)?)
        }
        flow_id if flow_id.contains("116-jup-lend-redeem-usdc-step-2") => {
            info!("[reev-agent] Handling flow step 2: Jupiter jUSDC redeem (withdraw)");
            let asset = usdc_mint();
            let redeem_amount = usdc::FORTY; // 40 USDC worth of jUSDC (conservative amount to ensure success)

            let instructions = agents::coding::d_116_jup_lend_redeem_usdc::handle_jupiter_redeem(
                &asset,
                redeem_amount,
                key_map,
            )
            .await?;

            info!(
                "[reev-agent] Step 2: Successfully generated {} Jupiter redeem instructions",
                instructions.len()
            );
            Ok(serde_json::to_string(&instructions)?)
        }
        _ => anyhow::bail!("Not a flow step benchmark: {benchmark_id}"),
    }
}

pub mod context;
pub mod enhanced;
pub mod flow;
pub mod run;

mod agents;
pub mod common;
mod prompt;

use reev_protocols::jupiter::lend_deposit::handle_jupiter_lend_deposit;
use reev_protocols::jupiter::swap::handle_jupiter_swap;

#[derive(Debug, Deserialize)]
pub struct LlmRequest {
    pub id: String,
    pub prompt: String,
    pub context_prompt: String,
    #[serde(default = "default_model")]
    pub model_name: String,
    #[serde(default)]
    pub mock: bool,
    #[serde(default)]
    pub initial_state: Option<Vec<reev_lib::benchmark::InitialStateItem>>,
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
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
    // Support old format for backward compatibility
    result: Option<LlmResult>,
    // Support new comprehensive format
    transactions: Option<Vec<serde_json::Value>>,
    summary: Option<String>,
    signatures: Option<Vec<String>>,
    // Flow information containing tool calls and execution order
    flows: Option<reev_lib::agent::FlowData>,
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

    // Try to parse the response as our new comprehensive format first
    match serde_json::from_str::<serde_json::Value>(&response_str) {
        Ok(json_value) => {
            // Check if it's our new comprehensive format
            if let (Some(transactions), Some(summary), Some(signatures)) = (
                json_value.get("transactions"),
                json_value.get("summary"),
                json_value.get("signatures"),
            ) {
                info!("[reev-agent] Detected new comprehensive format, passing through directly");
                // Return the new format directly
                // Extract flows if available
                let flows = json_value.get("flows").and_then(|f| {
                    serde_json::from_value::<reev_lib::agent::FlowData>(f.clone()).ok()
                });

                let response = LlmResponse {
                    result: None, // Old format not used
                    transactions: Some(transactions.as_array().unwrap_or(&vec![]).to_vec()),
                    summary: summary.as_str().map(|s| s.to_string()),
                    signatures: Some(
                        signatures
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|s| s.as_str())
                            .map(|s| s.to_string())
                            .collect(),
                    ),
                    flows, // Include flow data if available
                };
                return Ok(Json(response));
            }
        }
        Err(_) => {
            // Not our new format, continue with old logic
        }
    }

    // Use regex to find a JSON block for old format compatibility
    let re = Regex::new(r"(?s)```(?:json)?\s*(\{[\s\S]*?\}|\[[\s\S]*?\])\s*```")
        .context("Failed to compile JSON extraction regex")?;
    let extracted_json = if let Some(caps) = re.captures(&response_str) {
        caps.get(1).map_or("", |m| m.as_str()).to_string()
    } else {
        // If no markdown block is found, assume the whole response is the JSON string.
        response_str
    };

    let cleaned_response = extracted_json.trim().to_string();

    // Validate the response is valid JSON
    let _: serde_json::Value = serde_json::from_str(&cleaned_response)
        .context("Failed to validate AI agent response as parseable JSON")?;

    // Check if it's our new comprehensive format without markdown
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&cleaned_response) {
        if let (Some(transactions), Some(summary), Some(signatures)) = (
            json_value.get("transactions"),
            json_value.get("summary"),
            json_value.get("signatures"),
        ) {
            info!("[reev-agent] Detected clean comprehensive format, passing through directly");
            // Extract flows if available
            let flows = json_value
                .get("flows")
                .and_then(|f| serde_json::from_value::<reev_lib::agent::FlowData>(f.clone()).ok());

            let response = LlmResponse {
                result: None, // Old format not used
                transactions: Some(transactions.as_array().unwrap_or(&vec![]).to_vec()),
                summary: summary.as_str().map(|s| s.to_string()),
                signatures: Some(
                    signatures
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|s| s.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                ),
                flows, // Include flow data if available
            };
            return Ok(Json(response));
        }
    }

    // Fall back to old format for backward compatibility
    info!("[reev-agent] Falling back to old LlmResult format");
    let response = LlmResponse {
        result: Some(LlmResult {
            text: cleaned_response,
        }),
        transactions: None,
        summary: None,
        signatures: None,
        flows: None, // Flow data not available in legacy responses
    };

    Ok(Json(response))
}

/// Executes the deterministic, code-based agent logic to generate a ground truth instruction.
async fn run_deterministic_agent(payload: LlmRequest) -> Result<Json<LlmResponse>> {
    info!(
        "[reev-agent] Received request for benchmark id: \"{}\"",
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
    let instructions_json = match handle_simple_transfer_benchmarks(&payload.id, &key_map).await {
        Ok(result) => result,
        Err(_) => match handle_jupiter_swap_benchmarks(&payload.id, &key_map).await {
            Ok(result) => result,
            Err(_) => match handle_jupiter_lending_benchmarks(&payload.id, &key_map).await {
                Ok(result) => result,
                Err(_) => match handle_flow_step_benchmarks(&payload.id, &key_map).await {
                    Ok(result) => result,
                    Err(_) => match handle_flow_benchmarks(&payload.id, &key_map).await {
                        Ok(result) => result,
                        Err(_) => {
                            anyhow::bail!("Coding agent does not support this id: '{}'", payload.id)
                        }
                    },
                },
            },
        },
    };

    info!(
        "[reev-agent] Responding with instructions: {}",
        instructions_json
    );
    let response = LlmResponse {
        result: Some(LlmResult {
            text: instructions_json.clone(),
        }),
        transactions: None,
        summary: None,
        signatures: None,
        flows: None, // Flow data not available in legacy responses
    };

    Ok(Json(response))
}

/// The main entry point for the mock agent server.
pub async fn run_server() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // Initialize protocol configurations
    initialize_configurations()?;

    let app = Router::new()
        .route("/gen/tx", post(generate_transaction))
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9090").await?;
    info!("[reev-agent] Mock LLM server listening on http://127.0.0.1:9090");
    info!("[reev-agent] POST /gen/tx is ready to accept requests.");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize all protocol configurations
fn initialize_configurations() -> anyhow::Result<()> {
    info!("[reev-agent] Initializing protocol configurations...");

    // Initialize Jupiter configuration
    let jupiter_config = reev_protocols::jupiter::JupiterConfig::from_env();
    jupiter_config.validate()?;
    reev_protocols::jupiter::init_jupiter_config(jupiter_config);
    info!("[reev-agent] Jupiter configuration initialized");

    // Initialize Native configuration
    let native_config = reev_protocols::native::NativeConfig::from_env();
    reev_protocols::native::init_native_config(native_config);
    info!("[reev-agent] Native configuration initialized");

    info!("[reev-agent] All protocol configurations initialized successfully");
    Ok(())
}
