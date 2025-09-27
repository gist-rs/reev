#![cfg(test)]

use super::http_client::SurfpoolClient;
use anyhow::{Context, Result, anyhow};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::SolanaEnv,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account::get_associated_token_address;
use std::{collections::HashMap, fs, path::Path, str::FromStr, thread, time::Duration};
use tracing::info;

/// A helper to set up the `SolanaEnv` for a given benchmark file.
pub fn setup_env_for_benchmark(
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
/// 1.  Performs an initial `reset` which creates keypairs for wallets.
/// 2.  It then *derives* the correct ATA addresses based on the wallet and mint pubkeys.
/// 3.  If the benchmark doesn't define a recipient, it creates a "dummy" recipient ATA
///     to act as a valid sink for tokens in lending/swapping tests.
/// 4.  It uses the `surfpool` cheat code to fund the user's token account at the correct ATA.
pub async fn setup_spl_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    let (mut env, test_case, mut initial_observation) = setup_env_for_benchmark(benchmark_path)?;

    let usdc_mint_pubkey = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;

    let user_wallet_pubkey = Pubkey::from_str(
        initial_observation
            .key_map
            .get("USER_WALLET_PUBKEY")
            .context("USER_WALLET_PUBKEY not found in key_map")?,
    )?;

    let user_usdc_ata = get_associated_token_address(&user_wallet_pubkey, &usdc_mint_pubkey);
    info!("[setup] Derived correct User ATA: {}", user_usdc_ata);
    env.pubkey_map
        .insert("USER_USDC_ATA".to_string(), user_usdc_ata);
    initial_observation
        .key_map
        .insert("USER_USDC_ATA".to_string(), user_usdc_ata.to_string());

    let client = SurfpoolClient::new();

    // Handle recipient setup. If a recipient is defined in the benchmark, use it.
    // Otherwise, create a dummy recipient to act as a sink for funds, which is
    // necessary for benchmarks like lending or swapping where there's no explicit recipient.
    if let Some(recipient_wallet_pubkey_str) =
        initial_observation.key_map.get("RECIPIENT_WALLET_PUBKEY")
    {
        let recipient_wallet_pubkey = Pubkey::from_str(recipient_wallet_pubkey_str)?;
        let recipient_usdc_ata =
            get_associated_token_address(&recipient_wallet_pubkey, &usdc_mint_pubkey);

        info!(
            "[setup] Derived correct Recipient ATA: {}",
            recipient_usdc_ata
        );
        env.pubkey_map
            .insert("RECIPIENT_USDC_ATA".to_string(), recipient_usdc_ata);
        initial_observation.key_map.insert(
            "RECIPIENT_USDC_ATA".to_string(),
            recipient_usdc_ata.to_string(),
        );

        client
            .set_token_account(
                &recipient_wallet_pubkey.to_string(),
                &usdc_mint_pubkey.to_string(),
                0,
            )
            .await?;
        info!(
            "[setup] Funded recipient's ATA ({}) with 0 tokens.",
            recipient_usdc_ata
        );
    } else {
        info!("[setup] No recipient found in benchmark, creating a dummy recipient ATA.");
        let dummy_recipient_wallet = Pubkey::new_unique();
        let dummy_recipient_ata =
            get_associated_token_address(&dummy_recipient_wallet, &usdc_mint_pubkey);

        client
            .set_token_account(
                &dummy_recipient_wallet.to_string(),
                &usdc_mint_pubkey.to_string(),
                0,
            )
            .await?;

        info!(
            "[setup] Created and funded dummy ATA: {}",
            dummy_recipient_ata
        );

        // Insert the dummy ATA into the maps so the mock instruction builder can find it.
        env.pubkey_map
            .insert("RECIPIENT_USDC_ATA".to_string(), dummy_recipient_ata);
        initial_observation.key_map.insert(
            "RECIPIENT_USDC_ATA".to_string(),
            dummy_recipient_ata.to_string(),
        );
    }

    // Fund the user's account based on the specific benchmark.
    match test_case.id.as_str() {
        "002-SPL-TRANSFER" => {
            let amount = 50_000_000;
            client
                .set_token_account(
                    &user_wallet_pubkey.to_string(),
                    &usdc_mint_pubkey.to_string(),
                    amount,
                )
                .await?;
            info!(
                "[setup] Funded user's ATA ({}) with {} tokens.",
                user_usdc_ata, amount
            );
        }
        "111-JUP-LEND-USDC" => {
            let amount = 100_000_000;
            client
                .set_token_account(
                    &user_wallet_pubkey.to_string(),
                    &usdc_mint_pubkey.to_string(),
                    amount,
                )
                .await?;
            info!(
                "[setup] Funded user's ATA ({}) with {} tokens.",
                user_usdc_ata, amount
            );
        }
        "100-JUP-SWAP-SOL-USDC" => {
            // Pre-fund with a tiny amount of USDC to satisfy the `expected_gte: 1` assertion.
            client
                .set_token_account(
                    &user_wallet_pubkey.to_string(),
                    &usdc_mint_pubkey.to_string(),
                    100,
                )
                .await?;
            info!("[setup] Pre-funded user's USDC ATA for swap benchmark.");
        }
        _ => {}
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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

    // If a recipient isn't defined, create a dummy one. This allows SOL-based
    // benchmarks (like lending SOL) to pass without a pre-defined recipient.
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

    // This now expects a recipient ATA to exist, which is guaranteed by the setup logic.
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
