pub mod http_client;

use self::http_client::SurfpoolClient;

use anyhow::{Result, anyhow};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::SolanaEnv,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use std::{collections::HashMap, fs, path::Path, str::FromStr};
use tracing::info;

/// A helper to set up the `SolanaEnv` for a given benchmark file.
pub fn setup_env_for_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    let f = fs::File::open(benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    let mut env = SolanaEnv::new()?;
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let initial_observation = env.reset(None, Some(options))?;

    Ok((env, test_case, initial_observation))
}

/// A specialized helper for SPL token benchmarks that require setting up token accounts.
///
/// This function performs two `reset` calls:
/// 1. The first reset gets the dynamically generated wallet keys from the environment.
/// 2. It then uses an RPC client to call the `surfpool` cheat code, setting the token balance.
/// 3. The second reset ensures the environment re-syncs with the newly created on-chain state.
pub async fn setup_spl_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    // First reset to get the key map.
    let (mut env, test_case, initial_observation) = setup_env_for_benchmark(benchmark_path)?;
    let key_map = initial_observation.key_map;

    // Use the key map to set up the token account via RPC.
    let client = SurfpoolClient::new();
    let user_wallet = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("USER_WALLET_PUBKEY not found in key_map"))?;
    let usdc_mint = key_map
        .get("USDC_MINT_PUBKEY")
        .ok_or_else(|| anyhow!("USDC_MINT_PUBKEY not found in key_map"))?;

    info!(
        user_wallet = %user_wallet,
        usdc_mint = %usdc_mint,
        "Setting up USDC token account for user via RPC."
    );

    // Fund the user's USDC account with 100 USDC (assuming 6 decimals).
    client
        .set_token_account(user_wallet, usdc_mint, 100_000_000)
        .await?;
    info!("✅ User USDC account funded.");

    // Second reset to capture the state *after* the RPC call.
    let initial_state_json = serde_json::to_value(&test_case.initial_state)?;
    let options = serde_json::json!({ "initial_state": initial_state_json });
    let final_observation = env.reset(None, Some(options))?;
    info!("✅ Environment reset with updated token account state.");

    Ok((env, test_case, final_observation))
}

/// Creates a "mock perfect" instruction based on the benchmark ID.
pub fn mock_perfect_instruction(
    test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    match test_case.id.as_str() {
        "001-SOL-TRANSFER" => create_sol_transfer_instruction(key_map, 100_000_000), // 0.1 SOL
        "002-SPL-TRANSFER" | "003-SPL-TRANSFER-FAIL" => {
            create_spl_transfer_instruction(key_map, 15_000_000) // 15 USDC
        }
        "100-JUP-SWAP-SOL-USDC" => {
            create_sol_transfer_instruction(key_map, 100_000_000) // 0.1 SOL
        }
        "110-JUP-LEND-SOL" => {
            create_sol_transfer_instruction(key_map, 1_000_000_000) // 1 SOL
        }
        "111-JUP-LEND-USDC" => {
            create_spl_transfer_instruction(key_map, 100_000_000) // 100 USDC
        }
        _ => Err(anyhow!(
            "No mock instruction builder found for benchmark ID: {}",
            test_case.id
        )),
    }
}

/// Helper to create a SOL transfer instruction.
fn create_sol_transfer_instruction(
    key_map: &HashMap<String, String>,
    lamports: u64,
) -> Result<Instruction> {
    let from_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;
    let to_pubkey_str = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .cloned()
        .unwrap_or_else(|| Pubkey::new_unique().to_string());

    let from_pubkey = Pubkey::from_str(from_pubkey_str)?;
    let to_pubkey = Pubkey::from_str(&to_pubkey_str)?;

    Ok(system_instruction::transfer(
        &from_pubkey,
        &to_pubkey,
        lamports,
    ))
}

/// Helper to create an SPL transfer instruction.
fn create_spl_transfer_instruction(
    key_map: &HashMap<String, String>,
    amount: u64,
) -> Result<Instruction> {
    let source_pubkey_str = key_map
        .get("USER_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_USDC_ATA' not found in key_map"))?;
    let authority_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;
    let destination_pubkey_str = key_map
        .get("RECIPIENT_USDC_ATA")
        .cloned()
        .unwrap_or_else(|| Pubkey::new_unique().to_string());

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(&destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}
