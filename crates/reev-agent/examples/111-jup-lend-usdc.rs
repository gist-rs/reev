use anyhow::{Context, Result};
use jupiter_lend::run_server;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::{fs::File, path::PathBuf, time::Duration};
use tracing::info;

/// The mainnet USDC mint address.
const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

/// A minimal representation of the benchmark file for deserialization.
#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    prompt: String,
}

/// A standalone example to make a direct API call for the '111-jup-lend-usdc' scenario.
///
/// This example does the following:
/// 1. Spawns the `jupiter-lend` server in a background task.
/// 2. Waits for the server to become healthy.
/// 3. Loads the `111-jup-lend-usdc.yml` benchmark file.
/// 4. Creates a mock `user_public_key`.
/// 5. Sends a POST request to the `jupiter-lend` server to build the transaction.
/// 6. Prints the server's JSON response to the console.
///
/// # How to Run
///
/// ```sh
/// cargo run -p reev-agent --example 111-jup-lend-usdc
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and load environment variables from .env file.
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    info!("--- Running Jupiter Lend USDC Example ---");

    // 1. Spawn the server in a background task.
    tokio::spawn(async {
        if let Err(e) = run_server().await {
            eprintln!("[reev-agent-example] Jupiter Lend server failed: {e}");
        }
    });

    // 2. Wait for the server to be healthy before proceeding.
    let client = reqwest::Client::new();
    let health_url = "http://127.0.0.1:3000/health";
    info!("Waiting for Jupiter Lend server to start...");
    loop {
        match client.get(health_url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("Jupiter Lend server is running.");
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }

    // 3. Load the benchmark file.
    let benchmark_path = PathBuf::from("benchmarks/111-jup-lend-usdc.yml");
    let f = File::open(&benchmark_path)
        .with_context(|| format!("Failed to open benchmark file at: {benchmark_path:?}"))?;
    let test_case: TestCase =
        serde_yaml::from_reader(f).context("Failed to parse benchmark YAML")?;
    info!(
        "Loaded prompt for benchmark '{}': {}",
        test_case.id, test_case.prompt
    );

    // 4. Create a mock user public key.
    let user_wallet_pubkey = Pubkey::new_unique();
    info!("Using mock user wallet: {}", user_wallet_pubkey);

    // 5. Construct the JSON payload for the jupiter-lend server.
    // The prompt is "Lend 100 USDC..." which is 100,000,000 in the smallest unit (6 decimals).
    let request_payload = json!({
        "userPublicKey": user_wallet_pubkey.to_string(),
        "inputMint": USDC_MINT.to_string(),
        "amount": 100_000_000u64,
    });
    info!(
        "Request payload:\n{}",
        serde_json::to_string_pretty(&request_payload)?
    );

    // 6. Send the request to the running jupiter-lend server.
    let lend_server_url = "http://127.0.0.1:3000/build-lend-transaction";
    info!(
        "Sending request to Jupiter Lend server at {}...",
        lend_server_url
    );

    let response = client
        .post(lend_server_url)
        .json(&request_payload)
        .send()
        .await
        .context("Failed to send request to the lend server")?;

    // 7. Process and print the response.
    if response.status().is_success() {
        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to deserialize lend server response")?;
        info!("✅ Jupiter Lend server responded successfully!");
        println!("{}", serde_json::to_string_pretty(&response_json).unwrap());
    } else {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        anyhow::bail!("❌ Lend server request failed with status {status}: {error_body}");
    }

    Ok(())
}
