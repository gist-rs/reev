//! Extract Tool Details Module
//!
//! This module provides utility functions for extracting tool details from parsed tool calls.

use crate::handlers::flow_diagram::session_parser::ParsedToolCall;

use super::{extract_amount_from_tool_args, lamports_to_sol};

/// Extract tool details (from, to, amount) from a parsed tool call
pub fn extract_tool_details(tool_call: &ParsedToolCall) -> Option<(String, String, String)> {
    if let serde_json::Value::Object(map) = &tool_call.params {
        // Handle sol_transfer specific field names (user_pubkey, recipient_pubkey)
        // Fallback to generic field names for other tools
        let from = map
            .get("user_pubkey")
            .and_then(|v| v.as_str())
            .or_else(|| map.get("from").and_then(|v| v.as_str()))
            .or_else(|| map.get("source").and_then(|v| v.as_str()))
            .map(|s| {
                // Show full from address without truncation
                s.to_string()
            });

        // Try to extract actual recipient from result data first, fallback to params
        let to = if let Some(result_data) = &tool_call.result_data {
            if let Some(results_str) = result_data.get("results").and_then(|v| v.as_str()) {
                if let Ok(results_array) = serde_json::from_str::<serde_json::Value>(results_str) {
                    if let Some(accounts) = results_array
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|inst| inst.get("accounts"))
                        .and_then(|acc| acc.as_array())
                    {
                        // For SOL transfer, second account is typically the recipient
                        if accounts.len() >= 2 {
                            if let Some(recipient) = accounts
                                .get(1)
                                .and_then(|acc| acc.get("pubkey"))
                                .and_then(|pubkey| pubkey.as_str())
                            {
                                Some(recipient.to_string())
                            } else {
                                // Fallback to params
                                map.get("recipient_pubkey")
                                    .and_then(|v| v.as_str())
                                    .or_else(|| map.get("to").and_then(|v| v.as_str()))
                                    .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                                    .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                                    .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                                    .map(|s| s.to_string())
                            }
                        } else {
                            // Fallback to params
                            map.get("recipient_pubkey")
                                .and_then(|v| v.as_str())
                                .or_else(|| map.get("to").and_then(|v| v.as_str()))
                                .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                                .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                                .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                                .map(|s| s.to_string())
                        }
                    } else {
                        // Fallback to params
                        map.get("recipient_pubkey")
                            .and_then(|v| v.as_str())
                            .or_else(|| map.get("to").and_then(|v| v.as_str()))
                            .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                            .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                            .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                            .map(|s| s.to_string())
                    }
                } else {
                    // Fallback to params
                    map.get("recipient_pubkey")
                        .and_then(|v| v.as_str())
                        .or_else(|| map.get("to").and_then(|v| v.as_str()))
                        .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                        .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                        .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                        .map(|s| s.to_string())
                }
            } else {
                // Fallback to params
                map.get("recipient_pubkey")
                    .and_then(|v| v.as_str())
                    .or_else(|| map.get("to").and_then(|v| v.as_str()))
                    .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                    .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                    .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                    .map(|s| s.to_string())
            }
        } else {
            // Fallback to params
            map.get("recipient_pubkey")
                .and_then(|v| v.as_str())
                .or_else(|| map.get("to").and_then(|v| v.as_str()))
                .or_else(|| map.get("recipient").and_then(|v| v.as_str()))
                .or_else(|| map.get("pubkey").and_then(|v| v.as_str()))
                .or_else(|| map.get("destination").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
        };

        // Try to extract amount from tool_args JSON first (contains actual lamports)
        let amount = if let Some(tool_args_str) = &tool_call.tool_args {
            extract_amount_from_tool_args(tool_args_str)
        } else if let Some(amount) = map.get("amount").and_then(|v| v.as_u64()) {
            Some(lamports_to_sol(amount))
        } else {
            Some("transfer".to_string())
        };

        if let (Some(from), Some(to), Some(amount)) = (from, to, amount) {
            return Some((from, to, amount));
        }
    }
    None
}
