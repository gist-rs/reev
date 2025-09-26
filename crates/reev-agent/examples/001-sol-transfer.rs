use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, fs::File, path::PathBuf};
use tracing::info;

/// A minimal representation of the benchmark file for deserialization.
#[derive(Debug, Deserialize)]
struct TestCase {
    prompt: String,
}

/// The main function to run the example.
///
/// This example does the following:
/// 1. Loads the `001-sol-transfer.yml` benchmark file from the `benchmarks` directory.
/// 2. Creates a mock `context_prompt` with hardcoded public keys, simulating what the `reev-runner` would provide.
/// 3. Constructs the JSON payload that the `reev-agent` expects.
/// 4. Sends a POST request to a running `reev-agent` instance.
/// 5. Prints the agent's JSON response to the console.
///
/// # Pre-requisites
/// - A `reev-agent` instance must be running (`cargo run -p reev-agent`).
/// - A `.env` file with `GOOGLE_API_KEY` must be present in the workspace root.
///
/// # How to run
/// ```sh
/// cargo run -p reev-agent --example 001-sol-transfer
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and load environment variables from .env file.
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    info!("--- Running SOL Transfer Example ---");

    // 1. Load the benchmark file.
    // Assumes the example is run from the workspace root.
    let benchmark_path = PathBuf::from("benchmarks/001-sol-transfer.yml");
    let f = File::open(&benchmark_path)
        .with_context(|| format!("Failed to open benchmark file at: {benchmark_path:?}"))?;
    let test_case: TestCase =
        serde_yaml::from_reader(f).context("Failed to parse benchmark YAML")?;
    info!("Loaded prompt: '{}'", test_case.prompt);

    // 2. Create a mock context, simulating the runner's environment setup.
    // In a real run, these pubkeys would be dynamically generated.
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY",
        "USER_WALLET_PUBKEY_XXXXXXXXXXXXXXXXXXXX",
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY",
        "RECIPIENT_WALLET_PUBKEY_XXXXXXXXXXXXXX",
    );

    let context_yaml =
        serde_yaml::to_string(&json!({ "key_map": key_map })).context("Failed to create YAML")?;
    let context_prompt = format!("---\n\nCURRENT ON-CHAIN CONTEXT:\n{context_yaml}\n\n---");
    info!("Constructed mock context prompt.");

    // 3. Construct the JSON payload for the agent.
    let request_payload = json!({
        "prompt": test_case.prompt,
        "context_prompt": context_prompt,
    });
    info!(
        "Request payload:\n{}",
        serde_json::to_string_pretty(&request_payload)?
    );

    // 4. Send the request to the running reev-agent.
    let client = reqwest::Client::new();
    let agent_url = "http://127.0.0.1:9090/gen/tx";
    info!("Sending request to agent at {}...", agent_url);

    let response = client
        .post(agent_url)
        .json(&request_payload)
        .send()
        .await
        .context("Failed to send request to the agent")?;

    // 5. Process and print the response.
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
