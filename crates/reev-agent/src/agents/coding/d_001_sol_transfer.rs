use anyhow::{Context, Result};
use reev_lib::agent::RawInstruction;
use reev_protocols::native::handle_sol_transfer as protocol_handle_sol_transfer;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr, time::Instant};
use tracing::{error, info};

// Import enhanced OTEL logging macros
use reev_flow::{log_tool_call, log_tool_completion};

pub(crate) async fn handle_sol_transfer(
    key_map: &HashMap<String, String>,
) -> Result<Vec<RawInstruction>> {
    let start_time = Instant::now();
    info!("[reev-agent] Matched '001-sol-transfer' id. Calling centralized SOL transfer handler.");

    let from_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;
    let to_pubkey = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .context("RECIPIENT_WALLET_PUBKEY not found in key_map")?;
    let from = Pubkey::from_str(from_pubkey).context("Failed to parse from_pubkey")?;
    let to = Pubkey::from_str(to_pubkey).context("Failed to parse to_pubkey")?;
    let lamports = 100_000_000; // 0.1 SOL

    // 🎯 Add enhanced logging for deterministic agents
    let args = serde_json::json!({
        "user_pubkey": from_pubkey,
        "recipient_pubkey": to_pubkey,
        "lamports": lamports
    });
    log_tool_call!("deterministic_sol_transfer", &args);

    // Execute tool logic with inline error handling
    let result = async {
        // Call the protocol handler
        protocol_handle_sol_transfer(from, to, lamports, key_map).await
    }
    .await;

    match result {
        Ok(instructions) => {
            let execution_time = start_time.elapsed().as_millis() as u64;

            // 🎯 Add enhanced logging at SUCCESS
            let result_data = serde_json::json!({
                "instruction_count": instructions.len(),
                "lamports": lamports,
                "from_pubkey": from_pubkey,
                "to_pubkey": to_pubkey
            });
            log_tool_completion!(
                "deterministic_sol_transfer",
                execution_time,
                &result_data,
                true
            );

            info!(
                "[reev-agent] Successfully received {} instructions. Responding to runner.",
                instructions.len()
            );
            Ok(instructions)
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            let error_data = serde_json::json!({
                "error": e.to_string(),
                "lamports": lamports,
                "from_pubkey": from_pubkey,
                "to_pubkey": to_pubkey
            });

            // 🎯 Add enhanced logging at ERROR
            log_tool_completion!(
                "deterministic_sol_transfer",
                execution_time,
                &error_data,
                false
            );

            error!(
                "[deterministic_sol_transfer] Tool execution failed in {}ms: {}",
                execution_time, e
            );
            Err(e)
        }
    }
}
