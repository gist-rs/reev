pub mod http_client;

use self::http_client::SurfpoolClient;
use anyhow::{Context, Result, anyhow};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::SolanaEnv,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account::get_associated_token_address;
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
    let options = serde_json::json!({
        "id": test_case.id,
        "initial_state": initial_state_json
    });
    let initial_observation = env.reset(None, Some(options))?;

    Ok((env, test_case, initial_observation))
}

/// A specialized helper for SPL token benchmarks.
///
/// This function is the key to correctly setting up SPL benchmarks. It:
/// 1.  Performs an initial `reset` which creates keypairs for wallets but assigns
///     *temporary, incorrect* pubkeys for the ATA placeholders.
/// 2.  It then *derives* the correct ATA addresses based on the wallet and mint pubkeys.
/// 3.  It **replaces** the incorrect ATA pubkeys in the environment's `pubkey_map`
///     with the correct, derived ones.
/// 4.  It uses the `surfpool` cheat code to fund the user's token account at the correct ATA.
/// 5.  Crucially, it **does not** perform a second `reset`, as this would invalidate
///     the keypairs generated in the first step. The `initial_observation` is returned
///     with the now-correct `key_map`.
pub async fn setup_spl_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    let (mut env, test_case, mut initial_observation) = setup_env_for_benchmark(benchmark_path)?;

    // Define the USDC mint address, as this is a known constant for these benchmarks.
    let usdc_mint_pubkey = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    // Get the wallet pubkeys from the key_map created by the initial reset.
    let user_wallet_pubkey = Pubkey::from_str(
        initial_observation
            .key_map
            .get("USER_WALLET_PUBKEY")
            .context("USER_WALLET_PUBKEY not found in key_map")?,
    )?;
    let recipient_wallet_pubkey = Pubkey::from_str(
        initial_observation
            .key_map
            .get("RECIPIENT_WALLET_PUBKEY")
            .context("RECIPIENT_WALLET_PUBKEY not found in key_map")?,
    )?;

    // Derive the *correct* ATA addresses.
    let user_usdc_ata = get_associated_token_address(&user_wallet_pubkey, &usdc_mint_pubkey);
    let recipient_usdc_ata =
        get_associated_token_address(&recipient_wallet_pubkey, &usdc_mint_pubkey);

    info!("[setup] Derived correct ATAs:");
    info!("[setup]   User ATA: {}", user_usdc_ata);
    info!("[setup]   Recipient ATA: {}", recipient_usdc_ata);

    // IMPORTANT: Overwrite the incorrect, randomly-generated pubkeys in the maps
    // with the correctly derived ATA addresses.
    env.pubkey_map
        .insert("USER_USDC_ATA".to_string(), user_usdc_ata);
    initial_observation
        .key_map
        .insert("USER_USDC_ATA".to_string(), user_usdc_ata.to_string());

    env.pubkey_map
        .insert("RECIPIENT_USDC_ATA".to_string(), recipient_usdc_ata);
    initial_observation.key_map.insert(
        "RECIPIENT_USDC_ATA".to_string(),
        recipient_usdc_ata.to_string(),
    );

    // Fund the accounts using the surfpool cheat code.
    let client = SurfpoolClient::new();
    let user_initial_amount = 50_000_000; // 50 USDC, as defined in the benchmark.
    client
        .set_token_account(
            &user_wallet_pubkey.to_string(),
            &usdc_mint_pubkey.to_string(),
            user_initial_amount,
        )
        .await?;
    info!(
        "[setup] Funded user's ATA ({}) with {} tokens via RPC.",
        user_usdc_ata, user_initial_amount
    );

    let recipient_initial_amount = 0;
    client
        .set_token_account(
            &recipient_wallet_pubkey.to_string(),
            &usdc_mint_pubkey.to_string(),
            recipient_initial_amount,
        )
        .await?;
    info!(
        "[setup] Funded recipient's ATA ({}) with {} tokens via RPC.",
        recipient_usdc_ata, recipient_initial_amount
    );

    // Give the validator a moment to catch up with the RPC-injected state.
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Return the environment and the modified initial_observation with the correct key_map.
    // DO NOT reset the environment again.
    Ok((env, test_case, initial_observation))
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
        "110-JUP-LEND-SOL" => create_sol_transfer_instruction(key_map, 1_000_000_000), // 1 SOL
        "111-JUP-LEND-USDC" => create_spl_transfer_instruction(key_map, 100_000_000),  // 100 USDC
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
    // This function now correctly looks up the ATA addresses from the key_map,
    // which have been pre-populated with the correct derived addresses by `setup_spl_benchmark`.
    let source_pubkey_str = key_map
        .get("USER_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_USDC_ATA' not found in key_map"))?;
    let authority_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'USER_WALLET_PUBKEY' not found in key_map"))?;
    let destination_pubkey_str = key_map
        .get("RECIPIENT_USDC_ATA")
        .ok_or_else(|| anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map"))?;

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
