use anyhow::{Context, Result};
use reev_agent::run_server;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, fs::File, path::PathBuf, time::Duration};
use tracing::info;

mod common;

/// A minimal representation of the benchmark file for deserialization.
#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    prompt: String,
}

/// The main function to run the example.
///
/// This example demonstrates a direct API call to the `reev-agent` for a specific scenario.
///
/// # How to run
///
/// **Deterministic Agent (Default):**
/// ```sh
/// cargo run -p reev-agent --example 002-spl-transfer
/// ```
///
/// **Gemini Agent:**
/// ```sh
/// cargo run -p reev-agent --example 002-spl-transfer -- --agent gemini-2.5-pro
/// ```
///
/// **Local Agent:**
/// ```sh
/// cargo run -p reev-agent --example 002-spl-transfer -- --agent local
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and load environment variables from .env file.
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let agent_name = common::get_agent_name();

    info!(
        "--- Running SPL Transfer Example with Agent: {} ---",
        agent_name
    );

    // 1. Spawn the server in a background task.
    tokio::spawn(async {
        if let Err(e) = run_server().await {
            eprintln!("[reev-agent-example] Server failed: {e}");
        }
    });

    // 2. Wait for the server to be healthy before proceeding.
    let client = reqwest::Client::new();
    let health_url = "http://127.0.0.1:9090/health";
    info!("Waiting for agent server to start...");
    loop {
        match client.get(health_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("Agent server is running.");
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }

    // 3. Load the benchmark file.
    let benchmark_path = PathBuf::from("benchmarks/002-spl-transfer.yml");
    let f = File::open(&benchmark_path)
        .with_context(|| format!("Failed to open benchmark file at: {benchmark_path:?}"))?;
    let test_case: TestCase =
        serde_yaml::from_reader(f).context("Failed to parse benchmark YAML")?;
    info!("Loaded prompt: '{}'", test_case.prompt);

    // 4. Create a mock context, simulating the runner's environment setup.
    let user_wallet_pubkey = Pubkey::new_unique();
    let recipient_wallet_pubkey = Pubkey::new_unique();
    let mock_usdc_mint = Pubkey::new_unique();
    let user_usdc_ata = Pubkey::new_unique();
    let recipient_usdc_ata = Pubkey::new_unique();

    let mut key_map = HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY", user_wallet_pubkey.to_string());
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY",
        recipient_wallet_pubkey.to_string(),
    );
    key_map.insert("MOCK_USDC_MINT", mock_usdc_mint.to_string());
    key_map.insert("USER_USDC_ATA", user_usdc_ata.to_string());
    key_map.insert("RECIPIENT_USDC_ATA", recipient_usdc_ata.to_string());

    let context_yaml =
        serde_yaml::to_string(&json!({ "key_map": key_map })).context("Failed to create YAML")?;
    let context_prompt = format!("---\n\nCURRENT ON-CHAIN CONTEXT:\n{context_yaml}\n\n---");
    info!("Constructed mock context prompt.");

    // 5. Construct the JSON payload for the agent.
    let request_payload = json!({
        "id": test_case.id,
        "prompt": test_case.prompt,
        "context_prompt": context_prompt,
        "model_name": agent_name,
    });
    info!(
        "Request payload:\n{}",
        serde_json::to_string_pretty(&request_payload)?
    );

    // 6. Send the request to the running reev-agent.
    let agent_url = if agent_name == "deterministic" {
        "http://127.0.0.1:9090/gen/tx?mock=true"
    } else {
        "http://127.0.0.1:9090/gen/tx"
    };

    info!("Sending request to agent at {}...", agent_url);

    let response = client
        .post(agent_url)
        .json(&request_payload)
        .send()
        .await
        .context("Failed to send request to the agent")?;

    // 7. Process and print the response.
    if response.status().is_success() {
        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to deserialize agent response")?;
        info!("✅ Agent responded successfully!");
        println!("{}", serde_json::to_string_pretty(&response_json).unwrap());
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("❌ Agent request failed with status {status}: {error_body}");
    }

    Ok(())
}
