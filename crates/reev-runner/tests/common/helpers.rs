#![cfg(test)]

use anyhow::{Result, anyhow};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::SolanaEnv,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use std::{collections::HashMap, fs, path::Path, str::FromStr, thread, time::Duration};
use tracing::info;

/// A standard helper to set up the `SolanaEnv` for a given benchmark file.
///
/// This function is now the single entry point for setting up any benchmark. It
/// reads the test case from the file and calls `env.reset()`. The `SolanaEnv`'s
/// `reset` implementation now contains all the centralized logic (including complex
/// SPL setup), ensuring a consistent environment for all tests.
pub async fn setup_env_for_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    // HACK: Add a small delay and health check to mitigate race conditions
    // where the surfpool validator is not yet ready when the test starts.
    thread::sleep(Duration::from_millis(500));
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    for i in 0..20 {
        if rpc_client.get_health().is_ok() {
            break;
        }
        if i == 19 {
            return Err(anyhow!("Timed out waiting for validator to be healthy."));
        }
        thread::sleep(Duration::from_millis(500));
    }

    let f = fs::File::open(benchmark_path)?;
    let test_case: TestCase = serde_yaml::from_reader(f)?;

    let mut env = SolanaEnv::new()?;
    // `env.reset` now contains the centralized setup logic from `reev-lib`.
    let initial_observation = env
        .reset(None, Some(serde_json::to_value(&test_case)?))
        .await?;

    Ok((env, test_case, initial_observation))
}

/// Creates a "mock perfect" instruction based on the benchmark ID.
/// This represents the ideal action an agent should take.
pub fn mock_perfect_instruction(
    test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    match test_case.id.as_str() {
        "001-SOL-TRANSFER" => create_sol_transfer_instruction(key_map, 100_000_000), // 0.1 SOL
        "002-SPL-TRANSFER" | "003-SPL-TRANSFER-FAIL" => {
            create_spl_transfer_instruction(key_map, 15_000_000) // 15 USDC
        }
        "100-JUP-SWAP-SOL-USDC" => create_sol_transfer_instruction(key_map, 100_000_000), // 0.1 SOL
        "110-JUP-LEND-SOL" => create_sol_transfer_instruction(key_map, 1_000_000_000),    // 1 SOL
        "111-JUP-LEND-USDC" => create_spl_transfer_instruction(key_map, 100_000_000), // 100 USDC
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
    // If a recipient isn't defined, create a dummy one.
    let to_pubkey_str = key_map
        .get("RECIPIENT_WALLET_PUBKEY")
        .cloned()
        .unwrap_or_else(|| Pubkey::new_unique().to_string());

    let from_pubkey = Pubkey::from_str(from_pubkey_str)?;
    let to_pubkey = Pubkey::from_str(&to_pubkey_str)?;

    info!("[mock] Creating SOL Transfer IX:");
    info!("[mock]   From: {}", from_pubkey);
    info!("[mock]   To: {}", to_pubkey);
    info!("[mock]   Amount (lamports): {}", lamports);

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

    // The destination ATA is now guaranteed to exist in the key_map because the
    // centralized setup logic in `SolanaEnv::reset` creates a dummy one if needed.
    let destination_pubkey_str = key_map.get("RECIPIENT_USDC_ATA").ok_or_else(|| {
        anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map after setup")
    })?;

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    info!("[mock] Creating SPL Transfer IX:");
    info!("[mock]   Source ATA: {}", source_pubkey);
    info!("[mock]   Destination ATA: {}", destination_pubkey);
    info!("[mock]   Authority: {}", authority_pubkey);
    info!("[mock]   Amount: {}", amount);

    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}
