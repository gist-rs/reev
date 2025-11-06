//! Tool name constants for the reev ecosystem
//!
//! This module provides a centralized location for tool names
//! that can be imported by other crates to avoid hardcoding.

/// Native SOL transfer tool name
pub const SOL_TRANSFER: &str = "sol_transfer";

/// SPL token transfer tool name
pub const SPL_TRANSFER: &str = "spl_transfer";

/// Jupiter swap tool name
pub const JUPITER_SWAP: &str = "jupiter_swap";

/// Jupiter earn tool name
pub const JUPITER_EARN: &str = "get_jupiter_earn_position";

/// Jupiter lend earn deposit tool name
pub const JUPITER_LEND_EARN_DEPOSIT: &str = "jupiter_lend_earn_deposit";

/// Jupiter lend earn mint/redeem tool name
pub const JUPITER_LEND_EARN_MINT_REDEEM: &str = "jupiter_lend_earn_mint_redeem";

/// Jupiter lend earn withdraw tool name
pub const JUPITER_LEND_EARN_WITHDRAW: &str = "jupiter_lend_earn_withdraw";

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
