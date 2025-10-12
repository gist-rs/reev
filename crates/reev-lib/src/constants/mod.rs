//! Constants module for centralized configuration values

pub mod addresses;
pub mod amounts;
pub mod env;

// Re-export commonly used constants for convenience
pub use addresses::{
    network::{reev_agent_url, surfpool_url, LOCALHOST, REEV_AGENT_PORT, SURFPOOL_PORT},
    programs::{
        jupiter_program, system_program, token_program, A_TOKEN_PROGRAM, JUPITER_PROGRAM,
        SYSTEM_PROGRAM, TOKEN_PROGRAM,
    },
    tokens::{jusdc_mint, sol_mint, usdc_mint, JUSDC_MINT, SOL_MINT, USDC_MINT},
};

pub use amounts::{
    defaults::{
        SOL_SWAP_AMOUNT, SOL_SWAP_AMOUNT_MEDIUM, USDC_LEND_AMOUNT, USDC_LEND_AMOUNT_LARGE,
        USDC_MINT_AMOUNT,
    },
    scoring::{MAX_SCORE, MIN_PASSING_SCORE, SCORE_TOLERANCE},
    slippage::{EIGHT_PERCENT, FIVE_PERCENT, TEN_PERCENT},
    solana::{MIN_BALANCE, RENT_EXEMPTION},
    tokens::{sol, usdc},
};

pub use env::{
    agents::{default_agent, enable_mock, mock_param},
    database::{database_path, enable_connection_pooling, max_connections},
    llm::{google_api_key, local_server_url, max_turns, temperature},
    load_config,
    logging::{enable_debug, rust_log_filter},
    network::{reev_agent_host, reev_agent_port, surfpool_host, surfpool_port},
    solana::{commitment_level, devnet_rpc_url, mainnet_rpc_url, preflight_commitment},
    timeouts::{benchmark_timeout_seconds, health_check_seconds, http_request_seconds},
    validate_required_env,
};
