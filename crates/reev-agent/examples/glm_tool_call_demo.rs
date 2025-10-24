//! GLM Agent Tool Calling Demo
//!
//! This example demonstrates the GLM agent's tool calling capabilities using the ZAI provider.
//! Run with: cargo run --example glm_tool_call_demo --features native

use anyhow::Result;
use reev_agent::enhanced::zai_agent::ZAIAgent;
use reev_agent::LlmRequest;
use std::collections::HashMap;
use std::env;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Check for required environment variables
    let api_key = env::var("ZAI_API_KEY");
    if api_key.is_err() {
        eprintln!("❌ ZAI_API_KEY environment variable not set");
        eprintln!("Please set your ZAI API key:");
        eprintln!("export ZAI_API_KEY=your_api_key_here");
        return Ok(());
    }

    info!("🚀 Starting GLM Tool Calling Demo with ZAI Provider");
    info!(
        "🔑 Using ZAI API key: {}...",
        api_key.unwrap()[..8].to_string()
    );

    // Demo scenarios for SOL transfers (since we have SolTransferTool available)
    let scenarios = vec![
        (
            "Please send 0.1 SOL to the recipient wallet.",
            "Basic SOL transfer",
        ),
        (
            "Transfer 0.05 SOL from USER_WALLET_PUBKEY to RECIPIENT_WALLET_PUBKEY.",
            "SOL transfer with explicit addresses",
        ),
        (
            "Send 0.2 SOL to the recipient and include a small fee.",
            "SOL transfer with fee consideration",
        ),
        (
            "What is the current balance of the wallet?",
            "Balance query without transfers",
        ),
        (
            "Transfer 0.1 SOL and then check the resulting balance.",
            "Multi-step operation",
        ),
    ];

    // Create a sample key_map for demo purposes
    let mut key_map = HashMap::new();
    key_map.insert(
        "USER_WALLET_PUBKEY".to_string(),
        "11111111111111111111111111111111".to_string(),
    );
    key_map.insert(
        "RECIPIENT_WALLET_PUBKEY".to_string(),
        "22222222222222222222222222222222".to_string(),
    );

    for (i, (prompt, description)) in scenarios.into_iter().enumerate() {
        info!("\n📍 Scenario {}: {}", i + 1, description);
        info!("💬 Prompt: {}", prompt);
        info!("⏳ Processing...");

        let request = LlmRequest {
            id: format!("demo-{}-{}", i + 1, chrono::Utc::now().timestamp()),
            session_id: format!("demo-session-{}-{}", i + 1, chrono::Utc::now().timestamp()),
            prompt: prompt.to_string(),
            context_prompt: "You are a helpful assistant with access to Solana blockchain tools. Use tools when appropriate to execute transactions and provide accurate information.".to_string(),
            model_name: "glm-4.6".to_string(),
            mock: false,
            initial_state: None,
            allowed_tools: None,
            account_states: None,
        };

        match ZAIAgent::run("glm-4.6", request, key_map.clone()).await {
            Ok(response) => {
                info!("✅ Response received successfully");

                // Parse and display the response
                match serde_json::from_str::<serde_json::Value>(&response) {
                    Ok(json_response) => {
                        if let Some(summary) = json_response.get("summary") {
                            info!("📝 Summary: {}", summary.as_str().unwrap_or("No summary"));
                        }
                        if let Some(transactions) = json_response.get("transactions") {
                            if let Some(trans_array) = transactions.as_array() {
                                if !trans_array.is_empty() {
                                    info!("🔧 Generated {} transaction(s)", trans_array.len());
                                    for (j, tx) in trans_array.iter().enumerate() {
                                        if let Some(program_id) = tx.get("program_id") {
                                            info!(
                                                "   Transaction {}: Program ID = {}",
                                                j + 1,
                                                program_id.as_str().unwrap_or("Unknown")
                                            );
                                        }
                                    }
                                } else {
                                    info!("📝 No transactions generated");
                                }
                            }
                        }
                        if let Some(signatures) = json_response.get("signatures") {
                            if let Some(sig_array) = signatures.as_array() {
                                info!("🔐 Estimated signatures: {}", sig_array.len());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to parse JSON response: {e}");
                        info!("📄 Raw response: {}", response);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error processing request: {e}");
            }
        }

        // Add a small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("\n🎉 Demo completed! The GLM agent has demonstrated tool calling capabilities.");
    info!("💡 Key features shown:");
    info!("   • Tool detection and execution");
    info!("   • Parameter parsing and validation");
    info!("   • Solana transaction generation");
    info!("   • Proper JSON response formatting");
    info!("   • ZAI provider integration");

    Ok(())
}
