use anyhow::{Context, Result};
use reev_agent::run_server;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, fs::File, path::PathBuf, time::Duration};
use tracing::info;

/// A minimal representation of the benchmark file for deserialization.
#[derive(Debug, Deserialize)]
struct TestCase {
    prompt: String,
}

/// The main function to run the example.
///
/// This example does the following:
/// 1. Spawns the `reev-agent` server in a background task.
/// 2. Waits for the server to become healthy.
/// 3. Loads the `002-spl-transfer.yml` benchmark file from the `benchmarks` directory.
/// 4. Creates a mock `context_prompt` with hardcoded public keys.
/// 5. Constructs the JSON payload that the `reev-agent` expects.
/// 6. Sends a POST request to the now-running `reev-agent` instance.
/// 7. Prints the agent's JSON response to the console.
///
/// # Pre-requisites
/// - A `.env` file with `GOOGLE_API_KEY` must be present in the workspace root.
///
/// # How to run
/// ```sh
/// cargo run -p reev-agent --example 002-spl-transfer
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and load environment variables from .env file.
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    info!("--- Running SPL Transfer Example ---");

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
    // In a real run, these pubkeys would be dynamically generated.
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
        "prompt": test_case.prompt,
        "context_prompt": context_prompt,
    });
    info!(
        "Request payload:\n{}",
        serde_json::to_string_pretty(&request_payload)?
    );

    // 6. Send the request to the running reev-agent.
    let agent_url = "http://127.0.0.1:9090/gen/tx";
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
