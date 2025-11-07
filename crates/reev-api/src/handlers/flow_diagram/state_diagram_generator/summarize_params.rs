//! Summarize Params Module
//!
//! This module provides utility functions for summarizing parameters for display in diagrams.

use super::{lamports_to_token_amount, mint_to_symbol};

/// Summarize parameters for display in diagram
pub fn summarize_params(params: &serde_json::Value) -> String {
    match params {
        serde_json::Value::Object(map) => {
            // Enhanced Jupiter transaction parsing
            if let (Some(input_mint), Some(output_mint), Some(amount)) = (
                map.get("input_mint").and_then(|v| v.as_str()),
                map.get("output_mint").and_then(|v| v.as_str()),
                map.get("amount").and_then(|v| v.as_u64()),
            ) {
                let input_symbol = mint_to_symbol(input_mint);
                let output_symbol = mint_to_symbol(output_mint);
                let input_amount = lamports_to_token_amount(amount, input_mint);
                return format!("{input_amount} {input_symbol} → {output_symbol}");
            }

            // Jupiter lend/deposit parsing
            if let (Some(action), Some(mint), Some(amount)) = (
                map.get("action").and_then(|v| v.as_str()),
                map.get("mint").and_then(|v| v.as_str()),
                map.get("amount").and_then(|v| v.as_u64()),
            ) {
                let symbol = mint_to_symbol(mint);
                let amount_formatted = lamports_to_token_amount(amount, mint);
                return format!("{action} {amount_formatted} {symbol}");
            }

            // Generic parameter parsing
            let mut summaries = Vec::new();
            for (key, value) in map {
                if key == "pubkey" || key == "user_pubkey" {
                    if let Some(pubkey) = value.as_str() {
                        // Show first 8 chars of pubkey
                        let short_pubkey = if pubkey.len() > 8 {
                            format!("{}...", &pubkey[..8])
                        } else {
                            pubkey.to_string()
                        };
                        summaries.push(format!("{key} = {short_pubkey}"));
                    }
                } else if key == "amount" {
                    if let Some(amount) = value.as_u64() {
                        summaries.push(format!("{key} = {amount}"));
                    }
                } else if key == "input_mint" || key == "output_mint" {
                    if let Some(mint) = value.as_str() {
                        // Show token symbol if recognizable
                        let token_symbol = mint_to_symbol(mint);
                        summaries.push(format!("{} = {}", key.replace("_mint", ""), token_symbol));
                    }
                } else if summaries.len() < 3 {
                    // Limit to 3 most important params
                    summaries.push(format!("{key} = {value}"));
                }
            }

            if summaries.is_empty() {
                "Execute".to_string()
            } else {
                summaries.join(", ")
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..47])
            } else {
                s.clone()
            }
        }
        _ => {
            format!("{params:?}")
        }
    }
}
