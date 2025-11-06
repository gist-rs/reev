//! Constants for the reev ecosystem
//!
//! This crate provides centralized constants that can be used across
//! multiple crates without creating circular dependencies.

/// Account balance checking tool name
pub const GET_ACCOUNT_BALANCE: &str = "get_account_balance";

/// Jupiter position info tool name
pub const GET_JUPITER_LEND_EARN_POSITION: &str = "get_jupiter_lend_earn_position";

/// Jupiter lend earn tokens tool name
pub const GET_JUPITER_LEND_EARN_TOKENS: &str = "get_jupiter_lend_earn_tokens";

/// SOL transfer tool name
pub const SOL_TRANSFER: &str = "sol_transfer";

/// SPL transfer tool name
pub const SPL_TRANSFER: &str = "spl_transfer";

/// Jupiter swap tool name
pub const JUPITER_SWAP: &str = "jupiter_swap";

/// Jupiter swap flow tool name
pub const JUPITER_SWAP_FLOW: &str = "jupiter_swap_flow";

/// Jupiter lend earn deposit tool name
pub const JUPITER_LEND_EARN_DEPOSIT: &str = "jupiter_lend_earn_deposit";

/// Jupiter lend earn withdraw tool name
pub const JUPITER_LEND_EARN_WITHDRAW: &str = "jupiter_lend_earn_withdraw";

/// Jupiter lend earn mint tool name
pub const JUPITER_LEND_EARN_MINT: &str = "jupiter_lend_earn_mint";

/// Jupiter lend earn redeem tool name
pub const JUPITER_LEND_EARN_REDEEM: &str = "jupiter_lend_earn_redeem";

/// Generic transaction execution tool name
pub const EXECUTE_TRANSACTION: &str = "execute_transaction";

/// Legacy tool name constants for backward compatibility
/// These should be replaced with the new constants above
/// @deprecated Use GET_ACCOUNT_BALANCE instead
pub const ACCOUNT_BALANCE: &str = "get_account_balance";

/// @deprecated Use GET_JUPITER_LEND_EARN_POSITION instead
pub const JUPITER_POSITIONS: &str = "get_jupiter_lend_earn_position";

/// @deprecated Use GET_JUPITER_LEND_EARN_TOKENS instead
pub const LEND_EARN_TOKENS: &str = "get_jupiter_lend_earn_tokens";

/// @deprecated Use JUPITER_LEND_EARN_WITHDRAW instead
pub const JUPITER_WITHDRAW: &str = "jupiter_lend_earn_withdraw";

/// @deprecated Use GET_JUPITER_LEND_EARN_POSITION instead
pub const JUPITER_EARN: &str = "get_jupiter_lend_earn_position";

/// @deprecated Use JUPITER_LEND_EARN_DEPOSIT instead
pub const JUPITER_LEND: &str = "jupiter_lend_earn_deposit";

/// Get all valid tool names
pub fn all_tool_names() -> Vec<&'static str> {
    vec![
        GET_ACCOUNT_BALANCE,
        GET_JUPITER_LEND_EARN_POSITION,
        GET_JUPITER_LEND_EARN_TOKENS,
        SOL_TRANSFER,
        SPL_TRANSFER,
        JUPITER_SWAP,
        JUPITER_SWAP_FLOW,
        EXECUTE_TRANSACTION,
        JUPITER_LEND_EARN_DEPOSIT,
        JUPITER_LEND_EARN_WITHDRAW,
        JUPITER_LEND_EARN_MINT,
        JUPITER_LEND_EARN_REDEEM,
    ]
}

/// Check if a tool name is valid
pub fn is_valid_tool_name(tool_name: &str) -> bool {
    all_tool_names().contains(&tool_name)
}

/// Map old tool names to new ones for backward compatibility
pub fn normalize_tool_name(tool_name: &str) -> String {
    match tool_name {
        "account_balance" => GET_ACCOUNT_BALANCE.to_string(),
        "jupiter_positions" => GET_JUPITER_LEND_EARN_POSITION.to_string(),
        "jupiter_earn" => GET_JUPITER_LEND_EARN_POSITION.to_string(),
        "lend_earn_tokens" => GET_JUPITER_LEND_EARN_TOKENS.to_string(),
        "get_position_info" => GET_JUPITER_LEND_EARN_POSITION.to_string(),
        "jupiter_lend" => JUPITER_LEND_EARN_DEPOSIT.to_string(),
        "jupiter_withdraw" => JUPITER_LEND_EARN_WITHDRAW.to_string(),
        _ => tool_name.to_string(),
    }
}
