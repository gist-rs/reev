use anyhow::{Context, Result};
use reev_agent::run_server;
use reev_lib::server_utils::kill_existing_reev_agent;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, fs::File, path::PathBuf, time::Duration};
use tracing::{debug, info};

// Include the common CLI parsing module.
mod common;

use crate::common::helpers::{sync_benchmarks_to_database, ExampleConfig};

/// A minimal representation of the benchmark file for deserialization.
#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    prompt: String,
}

/// A standalone example to make a direct API call for the '001-sol-transfer' scenario.
///
/// This example does the following:
/// 1. Spawns the `reev-agent` server in a background task.
/// 2. Waits for the server to become healthy.
/// 3. Parses the `--agent` command-line argument to select an agent.
/// 4. Loads the `001-sol-transfer.yml` benchmark file.
/// 5. Creates a mock `context_prompt` with placeholder public keys.
/// 6. Sends a POST request to the `reev-agent` with the benchmark `id` and `prompt`.
/// 7. Prints the agent's JSON response to the console.
///
/// # How to Run
///
/// **Deterministic Agent (Default):**
/// ```sh
/// cargo run -p reev-agent --example 001-sol-transfer
/// ```
///
/// **Gemini Agent:**
/// ```sh
/// cargo run -p reev-agent --example 001-sol-transfer -- --agent glm-4.6
/// ```
///
/// **Local Agent:**
/// ```sh
/// cargo run -p reev-agent --example 001-sol-transfer -- --agent local
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and load environment variables from .env file.
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    // 3. Parse the `--agent` command-line argument. Defaults to "deterministic".
    let agent_name = common::get_agent_name();

    info!(
        "--- Running SOL Transfer Example with Agent: '{}' ---",
        agent_name
    );

    // Clean up any existing reev-agent processes before starting
    kill_existing_reev_agent(9090)
        .await
        .context("Failed to cleanup existing reev-agent processes")?;

    // 1. Spawn the server in a background task.
    tokio::spawn(async {
        if let Err(e) = run_server().await {
            eprintln!("[reev-agent-example] Server failed: {e}");
        }
    });

    // 2. Wait for the server to be healthy before proceeding.
    let config = ExampleConfig::new(&agent_name);
    info!("Waiting for agent server to start...");
    loop {
        match config.client.get(config.health_check_url()).send().await {
            Ok(response) if response.status().is_success() => {
                info!("Agent server is running.");
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }

    // 3. Sync benchmarks to database before running examples
    sync_benchmarks_to_database()
        .await
        .context("Failed to sync benchmarks to database")?;

    // 4. Load the benchmark file.
    let benchmark_path = PathBuf::from("benchmarks/001-sol-transfer.yml");
    let f = File::open(&benchmark_path)
        .with_context(|| format!("Failed to open benchmark file at: {benchmark_path:?}"))?;
    let test_case: TestCase =
        serde_yaml::from_reader(f).context("Failed to parse benchmark YAML")?;
    info!("Loaded prompt for benchmark '{}'", test_case.id);

    // 5. Create a mock context, simulating the runner's environment setup.
    let user_wallet_pubkey = Pubkey::new_unique();
    let recipient_wallet_pubkey = Pubkey::new_unique();

    let mut key_map = HashMap::new();
    key_map.insert("USER_WALLET_PUBKEY", user_wallet_pubkey.to_string());
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY",
        recipient_wallet_pubkey.to_string(),
    );

    let context_yaml =
        serde_yaml::to_string(&json!({ "key_map": key_map })).context("Failed to create YAML")?;
    let context_prompt = format!("---\n\nCURRENT ON-CHAIN CONTEXT:\n{context_yaml}\n\n---");

    // 6. Construct the JSON payload for the agent.
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

    // 7. Send the request to the running reev-agent using common helper.
    info!("Sending request to agent at {}...", config.tx_url());

    let response = config
        .client
        .post(config.tx_url())
        .json(&request_payload)
        .send()
        .await
        .context("Failed to send request to the agent")?;

    // 8. Process and print the response.
    if response.status().is_success() {
        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to deserialize agent response")?;
        info!("✅ Agent responded successfully!");
        debug!("{}", serde_json::to_string_pretty(&response_json).unwrap());
    } else {
        let status = response.status();
        tracing::error!("❌ Agent request failed with status: {}", status);
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Error response: {}", error_text);
        return Err(anyhow::anyhow!("Agent request failed: {status}"));
    }

    // The server is running in a background thread. Exit explicitly.
    std::process::exit(0);
}
