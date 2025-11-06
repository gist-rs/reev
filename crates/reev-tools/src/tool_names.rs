//! Tool name constants for the reev ecosystem
//!
//! This module provides a centralized location for tool names
//! that can be imported by other crates to avoid hardcoding.
//! These are re-exported from reev-constants to maintain backward compatibility.

// Re-export all constants from reev-constants
pub use reev_constants::*;

// Legacy re-exports for backward compatibility
pub use reev_constants::ACCOUNT_BALANCE;
pub use reev_constants::JUPITER_EARN;
pub use reev_constants::JUPITER_LEND;
pub use reev_constants::JUPITER_POSITIONS;
pub use reev_constants::JUPITER_WITHDRAW;
pub use reev_constants::LEND_EARN_TOKENS;

// Re-export utility functions
pub use reev_constants::all_tool_names;
pub use reev_constants::is_valid_tool_name;
pub use reev_constants::normalize_tool_name;

/// Map program IDs to tool names for fallback parsing
pub fn tool_name_from_program_id(program_id: &str) -> String {
    match program_id {
        "11111111111111111111111111111111" => SOL_TRANSFER.to_string(), // System Program
        _ => {
            // For other programs, return a formatted name
            // This is used as fallback when proper tool tracking isn't available
            if program_id.len() >= 8 {
                format!("program_{}", &program_id[..8])
            } else {
                "unknown_program".to_string()
            }
        }
    }
}
