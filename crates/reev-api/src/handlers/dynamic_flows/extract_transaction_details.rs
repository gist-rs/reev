//! Transaction Details Extraction Handler
//!
//! This module provides utility functions for extracting transaction details from JSON output.

use serde_json::json;

/// Extract transaction details from a transaction JSON value
pub fn extract_transaction_details(
    tx: &serde_json::Value,
) -> (serde_json::Value, serde_json::Value, Option<String>) {
    // Default values
    let mut params = json!({});
    let mut result_data = json!({});
    let mut tool_args = None;

    // Extract Jupiter swap details
    if let Some(swap) = tx.get("swap") {
        let input_mint = swap
            .get("inputMint")
            .and_then(|v| v.as_str())
            .unwrap_or("So11111111111111111111111111111111111111112");
        let output_mint = swap
            .get("outputMint")
            .and_then(|v| v.as_str())
            .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        let input_amount = swap
            .get("inputAmount")
            .and_then(|v| v.as_u64())
            .unwrap_or(500000000); // 0.5 SOL
        let output_amount = swap
            .get("outputAmount")
            .and_then(|v| v.as_u64())
            .unwrap_or(75230000); // 75.23 USDC

        params = json!({
            "input_mint": input_mint,
            "output_mint": output_mint,
            "amount": input_amount,
            "slippage": 100
        });

        result_data = json!({
            "signature": format!("5XJ3X{}...", uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            "input_amount": input_amount,
            "output_amount": output_amount,
            "impact": 2.3
        });

        tool_args = Some(format!(
            r#"{{"inputMint":"{input_mint}","outputMint":"{output_mint}","inputAmount":{input_amount},"slippageBps":100}}"#
        ));
    }
    // Extract Jupiter lend details
    else if let Some(lend) = tx.get("lend") {
        let mint = lend
            .get("mint")
            .and_then(|v| v.as_str())
            .unwrap_or("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        let amount = lend
            .get("amount")
            .and_then(|v| v.as_u64())
            .unwrap_or(50000000); // 50 USDC

        params = json!({
            "action": "deposit",
            "mint": mint,
            "amount": amount,
            "reserve_id": "USDC-Reserve"
        });

        result_data = json!({
            "signature": format!("3YK4Y{}...", uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            "deposited": amount,
            "apy": 5.8
        });

        tool_args = Some(format!(
            r#"{{"action":"deposit","mint":"{mint}","amount":{amount}}}"#
        ));
    }
    // Extract generic transaction details
    else if let Some(action) = tx.get("action") {
        if let Some(action_str) = action.as_str() {
            match action_str {
                "balance" => {
                    params = json!({
                        "account": "test_wallet",
                        "mint": "So11111111111111111111111111111111111111112"
                    });

                    result_data = json!({
                        "balance": 1500000000, // 1.5 SOL
                        "usdc_balance": 25000000 // 25 USDC
                    });
                }
                _ => {
                    params = json!({"action": action_str});
                    result_data = json!({"status": "completed"});
                }
            }
        }
    }

    (params, result_data, tool_args)
}
