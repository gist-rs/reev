//! Operation types and parameter parsing for YML generator
//!
//! This module defines the different operation types that can be generated
//! and provides utilities for parsing parameters from prompts.

use anyhow::Result;
use regex;

/// Operation types supported by the YML generator
#[derive(Debug, Clone)]
pub enum OperationType {
    Swap(SwapParams),
    Transfer(TransferParams),
    Lend(LendParams),
    SwapThenLend(SwapThenLendParams),
    Unknown,
}

/// Parameters for swap operations
#[derive(Debug, Clone)]
pub struct SwapParams {
    pub amount: f64,
    pub from_token: String,
    pub to_token: String,
}

/// Parameters for transfer operations
#[derive(Debug, Clone)]
pub struct TransferParams {
    pub amount: f64,
    pub recipient: String,
}

/// Parameters for lend operations
#[derive(Debug, Clone)]
pub struct LendParams {
    pub amount: f64,
    pub token: String,
}

/// Parameters for swap then lend operations
#[derive(Debug, Clone)]
pub struct SwapThenLendParams {
    pub amount: f64,
    pub from_token: String,
    pub to_token: String,
}

/// Parse swap parameters from prompt
pub fn parse_swap_params(prompt: &str) -> Result<SwapParams> {
    // Default values
    let mut from = "SOL".to_string();
    let mut to = "USDC".to_string();
    let mut amount = 1.0;

    // Try to extract "from" token
    for token in ["SOL", "USDC", "USDT"] {
        if prompt.contains(&format!("{} ", token.to_lowercase()))
            || prompt.contains(&format!(" {}", token.to_lowercase()))
        {
            from = token.to_string();
            break;
        }
    }

    // Try to extract "to" token
    for token in ["SOL", "USDC", "USDT"] {
        if token != from
            && (prompt.contains(&format!(" {}", token.to_lowercase()))
                || prompt.contains(&format!(" to {}", token.to_lowercase())))
        {
            to = token.to_string();
            break;
        }
    }

    // Try to extract amount
    if let Some(percentage) = extract_percentage(prompt) {
        // Percentage detected
        amount = percentage;
    } else {
        // Look for specific amount
        let amount_regex = regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
        if let Some(captures) = amount_regex.captures(prompt) {
            if let Ok(val) = captures[1].parse::<f64>() {
                amount = val;
            }
        }
    }

    Ok(SwapParams {
        amount,
        from_token: from,
        to_token: to,
    })
}

/// Parse transfer parameters from prompt
pub fn parse_transfer_params(prompt: &str) -> Result<TransferParams> {
    // Default values
    let mut recipient = "unknown".to_string();
    let mut amount = 1.0;

    // Try to extract recipient address
    let pubkey_regex = regex::Regex::new(r"([A-HJ-NP-Za-km-z1-9]{32,44})").unwrap();
    if let Some(captures) = pubkey_regex.captures(prompt) {
        recipient = captures[1].to_string();
    }

    // Try to extract amount
    if let Some(percentage) = extract_percentage(prompt) {
        // Percentage detected
        amount = percentage;
    } else {
        // Look for specific amount
        let amount_regex = regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
        if let Some(captures) = amount_regex.captures(prompt) {
            if let Ok(val) = captures[1].parse::<f64>() {
                amount = val;
            }
        }
    }

    Ok(TransferParams { amount, recipient })
}

/// Parse lend parameters from prompt
pub fn parse_lend_params(prompt: &str) -> Result<LendParams> {
    // Default values
    let mut token = "USDC".to_string();
    let mut amount = 100.0;

    // Try to extract token
    for t in ["SOL", "USDC", "USDT"] {
        if prompt.contains(&t.to_lowercase()) {
            token = t.to_string();
            break;
        }
    }

    // Try to extract amount
    if let Some(percentage) = extract_percentage(prompt) {
        // Percentage detected
        amount = percentage;
    } else {
        let amount_regex = regex::Regex::new(r"(\d+\.?\d*)\s*(sol|usdc|usdt|eth|btc)?").unwrap();
        if let Some(captures) = amount_regex.captures(prompt) {
            if let Ok(val) = captures[1].parse::<f64>() {
                amount = val;
            }
        }
    }

    Ok(LendParams { amount, token })
}

/// Parse swap then lend parameters from prompt
pub fn parse_swap_then_lend_params(prompt: &str) -> Result<SwapThenLendParams> {
    let swap_params = parse_swap_params(prompt)?;
    Ok(SwapThenLendParams {
        amount: swap_params.amount,
        from_token: swap_params.from_token,
        to_token: swap_params.to_token,
    })
}

/// Extract percentage from prompt
pub fn extract_percentage(prompt: &str) -> Option<f64> {
    let regex = regex::Regex::new(r"(\d+\.?\d*)%").unwrap();
    regex
        .captures(prompt)
        .and_then(|captures| captures[1].parse::<f64>().ok())
}

/// Check if prompt contains "all" for SOL operations
pub fn is_sell_all(prompt: &str) -> bool {
    prompt.to_lowercase().contains("all") && prompt.to_lowercase().contains("sol")
}
