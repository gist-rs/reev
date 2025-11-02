use anyhow::{Context, Result};
use reev_lib::mock::MockGenerator;
use serde_json::json;
use std::{collections::HashMap, time::Instant};
use tracing::info;

// Import enhanced OTEL logging macros
use reev_flow::{log_tool_call, log_tool_completion};

/// Handles the deterministic logic for the `114-JUP-POSITIONS-AND-EARNINGS` benchmark.
///
/// This is a multi-step flow benchmark that demonstrates fetching Jupiter positions
/// and then getting earnings data. For the deterministic agent, we return mock
/// responses that simulate the real API calls.
pub(crate) async fn handle_jup_positions_and_earnings(
    key_map: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    let start_time = Instant::now();
    info!("[reev-agent] Matched '114-jup-positions-and-earnings' id. Creating deterministic multi-step flow response.");

    let user_pubkey = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found in key_map")?;

    // 🎯 Add enhanced logging for deterministic agents
    let args = serde_json::json!({
        "user_pubkey": user_pubkey,
        "benchmark_type": "multi_step_flow",
        "steps": ["positions", "earnings"]
    });
    log_tool_call!("deterministic_jup_positions_earnings", &args);

    // Create mock data generator
    let mut mock_gen = MockGenerator::with_seed(42); // Use fixed seed for deterministic results

    // Step 1: Generate mock Jupiter positions response
    let position_summary_1 = mock_gen.jupiter_position_summary(true); // Has balance
    let position_summary_2 = mock_gen.jupiter_position_summary(false); // No balance
    let position_summary_3 = mock_gen.jupiter_position_summary(false); // No balance

    let positions_response = json!({
        "total_positions": 6,
        "positions_with_balance": 1,
        "summary": [
            position_summary_1,
            position_summary_2,
            position_summary_3,
            // Add 3 more empty positions for variety
            mock_gen.jupiter_position_summary(false),
            mock_gen.jupiter_position_summary(false),
            mock_gen.jupiter_position_summary(false)
        ],
        "raw_positions": []
    });

    // Step 2: Generate mock Jupiter earnings response
    let earnings_response = json!({
        "user_pubkey": user_pubkey,
        "position_filter": null,
        "total_positions": 1,
        "positions": [
            mock_gen.jupiter_position_item(user_pubkey, true)
        ],
        "raw_earnings": [
            mock_gen.raw_jupiter_earnings(user_pubkey, true)
        ]
    });

    // Create a comprehensive response that includes both steps
    let response = json!({
        "step_1_result": positions_response,
        "step_2_result": earnings_response,
        "flow_completed": true,
        "summary": {
            "total_positions": 6,
            "positions_with_balance": 1,
            "total_earnings_usd": mock_gen.random_price(1.0, 10.0),
            "total_deposits_usd": mock_gen.random_price(1000.0, 5000.0),
            "total_withdraws_usd": mock_gen.random_price(500.0, 3000.0),
            "current_balance_usd": mock_gen.random_price(500.0, 2000.0),
            "active_positions": ["jlUSDC"],
            "highest_yielding_position": {
                "symbol": "jlUSDC",
                "apy": 8.68,
                "balance_usd": mock_gen.random_price(500.0, 2000.0)
            }
        },
        "next_step": "flow_completed",
        "message": "Successfully retrieved Jupiter positions and earnings. User has 1 active jlUSDC position with earnings."
    });

    let execution_time = start_time.elapsed().as_millis() as u64;

    // 🎯 Add enhanced logging at SUCCESS
    log_tool_completion!(
        "deterministic_jup_positions_earnings",
        execution_time,
        &response,
        true
    );

    info!(
        "[deterministic_jup_positions_earnings] Flow execution completed in {}ms",
        execution_time
    );

    Ok(response)
}
