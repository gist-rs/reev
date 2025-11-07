//! Summarize Result Data Module
//!
//! This module provides utility functions for summarizing result data for display in diagrams.

use super::lamports_to_token_amount;

/// Summarize result data for display in diagram
pub fn summarize_result_data(result_data: &serde_json::Value) -> Option<String> {
    match result_data {
        serde_json::Value::Object(map) => {
            // Handle Jupiter swap results
            if let (Some(input_amount), Some(output_amount), Some(signature)) = (
                map.get("input_amount").and_then(|v| v.as_u64()),
                map.get("output_amount").and_then(|v| v.as_u64()),
                map.get("signature").and_then(|v| v.as_str()),
            ) {
                let input_formatted = lamports_to_token_amount(
                    input_amount,
                    "So11111111111111111111111111111111111111112",
                );
                let output_formatted = lamports_to_token_amount(
                    output_amount,
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                );
                let short_sig = if signature.len() > 12 {
                    format!("{}...", &signature[..12])
                } else {
                    signature.to_string()
                };
                return Some(format!(
                    "{input_formatted} SOL → {output_formatted} USDC ({short_sig})"
                ));
            }

            // Handle Jupiter lend results
            if let (Some(deposited), Some(apy), Some(signature)) = (
                map.get("deposited").and_then(|v| v.as_u64()),
                map.get("apy").and_then(|v| v.as_f64()),
                map.get("signature").and_then(|v| v.as_str()),
            ) {
                let deposited_formatted = lamports_to_token_amount(
                    deposited,
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                );
                let short_sig = if signature.len() > 12 {
                    format!("{}...", &signature[..12])
                } else {
                    signature.to_string()
                };
                return Some(format!(
                    "deposit {deposited_formatted} USDC @ {apy:.1}% APY ({short_sig})"
                ));
            }

            // Handle balance check results
            if let (Some(balance), Some(usdc_balance)) = (
                map.get("balance").and_then(|v| v.as_u64()),
                map.get("usdc_balance").and_then(|v| v.as_u64()),
            ) {
                let sol_formatted = lamports_to_token_amount(
                    balance,
                    "So11111111111111111111111111111111111111112",
                );
                let usdc_formatted = lamports_to_token_amount(
                    usdc_balance,
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                );
                return Some(format!(
                    "Balance: {sol_formatted} SOL, {usdc_formatted} USDC"
                ));
            }

            // Generic result with signature
            if let Some(signature) = map.get("signature").and_then(|v| v.as_str()) {
                let short_sig = if signature.len() > 12 {
                    format!("{}...", &signature[..12])
                } else {
                    signature.to_string()
                };
                return Some(format!("Tx: {short_sig}"));
            }

            None
        }
        _ => None,
    }
}
