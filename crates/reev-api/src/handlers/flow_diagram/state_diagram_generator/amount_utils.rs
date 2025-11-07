//! Amount Utilities Module
//!
//! This module provides utility functions for handling token amounts and conversions.

/// Convert lamports to formatted SOL string
pub fn lamports_to_sol(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    // Format to avoid floating point issues, show max 4 decimal places
    if sol == sol.trunc() {
        format!("{sol:.0} SOL")
    } else if sol * 10.0 == (sol * 10.0).trunc() {
        format!("{sol:.1} SOL")
    } else if sol * 100.0 == (sol * 100.0).trunc() {
        format!("{sol:.2} SOL")
    } else if sol * 1000.0 == (sol * 1000.0).trunc() {
        format!("{sol:.3} SOL")
    } else {
        format!("{sol:.4} SOL")
    }
}

/// Extract amount from tool_args JSON string
pub fn extract_amount_from_tool_args(tool_args: &str) -> Option<String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(tool_args) {
        if let Some(amount) = parsed.get("amount").and_then(|v| v.as_u64()) {
            return Some(lamports_to_sol(amount));
        }
    }
    None
}
