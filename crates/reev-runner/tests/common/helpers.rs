#![cfg(test)]

use anyhow::{Context, Result, anyhow};
use jup_sdk::{
    Jupiter,
    models::{DepositParams, SwapParams},
    surfpool::SurfpoolClient,
};
use reev_lib::{
    actions::spl_transfer, agent::AgentObservation, benchmark::TestCase, env::GymEnv,
    solana_env::environment::SolanaEnv,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account;
use spl_token;
use std::{collections::HashMap, fs, path::Path, str::FromStr, thread, time::Duration};
use tracing::info;

const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";

/// A standard helper to set up the `SolanaEnv` for a given benchmark file.
///
/// This function is the single entry point for setting up any benchmark. It
/// reads the test case from the file and calls `env.reset()`. The `SolanaEnv`'s
/// `reset` implementation contains all the centralized logic, ensuring a
/// consistent environment for all tests.
pub async fn setup_env_for_benchmark(
    benchmark_path: &Path,
) -> Result<(SolanaEnv, TestCase, AgentObservation)> {
    // HACK: Add a small delay and health check to mitigate race conditions
    // where the surfpool validator is not yet ready when the test starts.
    thread::sleep(Duration::from_millis(500));
    let rpc_client = RpcClient::new(LOCAL_RPC_URL.to_string());
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
    let initial_observation = env
        .reset(None, Some(serde_json::to_value(&test_case)?))
        .await?;

    Ok((env, test_case, initial_observation))
}

/// Creates a "mock perfect" instruction based on the benchmark ID for simple transfers.
pub fn mock_perfect_instruction(
    test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Instruction> {
    match test_case.id.as_str() {
        "001-SOL-TRANSFER" => create_sol_transfer_instruction(key_map, 100_000_000), // 0.1 SOL
        "002-SPL-TRANSFER" | "003-SPL-TRANSFER-FAIL" => {
            create_spl_transfer_instruction(key_map, 15_000_000) // 15 USDC
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
    let destination_pubkey_str = key_map.get("RECIPIENT_USDC_ATA").ok_or_else(|| {
        anyhow!("Pubkey placeholder 'RECIPIENT_USDC_ATA' not found in key_map after setup")
    })?;

    let source_pubkey = Pubkey::from_str(source_pubkey_str)?;
    let destination_pubkey = Pubkey::from_str(destination_pubkey_str)?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey_str)?;

    spl_transfer::create_instruction(
        &source_pubkey,
        &destination_pubkey,
        &authority_pubkey,
        amount,
    )
}

/// Prepares the on-chain environment for a Jupiter swap using the `jup-sdk`.
///
/// This function encapsulates all the off-chain work required for a successful
/// swap, delegating the complexity to the SDK. It fetches instructions, identifies
/// all required accounts (including those in ALTs), and pre-loads them into the
/// local `surfpool` fork.
pub async fn prepare_jupiter_swap(
    env: &SolanaEnv,
    _test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Vec<Instruction>> {
    let user_pubkey_str = key_map
        .get("USER_WALLET_PUBKEY")
        .context("USER_WALLET_PUBKEY not found")?;
    let user_pubkey = Pubkey::from_str(user_pubkey_str)?;

    // Parameters are specific to the `100-JUP-SWAP-SOL-USDC` benchmark.
    let input_mint = spl_token::native_mint::ID;
    let output_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
    let amount = 100_000_000; // 0.1 SOL
    let slippage_bps = 500; // 5%

    // 1. Initialize the Jupiter SDK client for surfpool.
    // We clone the RpcClient from the environment to use it here.
    let rpc_client = RpcClient::new(env.rpc_client.url());
    let jupiter_client = Jupiter::surfpool_with_rpc(rpc_client).with_user_pubkey(user_pubkey);

    let swap_params = SwapParams {
        input_mint,
        output_mint,
        amount,
        slippage_bps,
    };

    // 2. Use the SDK to fetch all transaction components.
    info!("[TestSetup] Getting Jupiter transaction components via SDK...");
    let (instructions, alt_accounts) = jupiter_client
        .swap(swap_params)
        .prepare_transaction_components()
        .await?;

    // 3. Use the SDK's utilities to preload all necessary accounts.
    info!("[TestSetup] Pre-loading all required accounts via SDK...");
    let surfpool_client = SurfpoolClient::new(&env.rpc_client.url());
    jup_sdk::surfpool::preload_accounts(
        &env.rpc_client,
        &surfpool_client,
        &user_pubkey,
        &instructions,
        &alt_accounts,
    )
    .await?;

    info!("[TestSetup] Environment preparation for Jupiter swap complete.");
    Ok(instructions)
}

/// Prepares the environment and fetches instructions for a Jupiter lend deposit.
///
/// This function is the "smart test" equivalent for lending benchmarks. It calls
/// the Jupiter SDK to get the real deposit instructions and preloads all required
/// accounts into the `surfpool` test environment, ensuring the test mimics a
/// perfect agent's actions.
pub async fn prepare_jupiter_lend_deposit(
    env: &SolanaEnv,
    _test_case: &TestCase,
    key_map: &HashMap<String, String>,
) -> Result<Vec<Instruction>> {
    info!("[Test Helper] Preparing for Jupiter lend deposit...");
    let user_pubkey = Pubkey::from_str(key_map.get("USER_WALLET_PUBKEY").unwrap())?;
    let amount = 100_000_000; // 0.1 SOL

    // --- 1. Instructions to wrap SOL ---
    // The Jupiter program expects the user to have an initialized WSOL token account.
    // We must create it and fund it before calling the lend instruction.
    let wsol_ata = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &spl_token::native_mint::ID,
    );

    let mut wrap_instructions = vec![
        // Create ATA. This is idempotent, so it's safe to call even if it exists.
        spl_associated_token_account::instruction::create_associated_token_account(
            &user_pubkey,
            &user_pubkey,
            &spl_token::native_mint::ID,
            &spl_token::ID,
        ),
        // Transfer SOL to WSOL ATA to wrap it.
        system_instruction::transfer(&user_pubkey, &wsol_ata, amount),
        // Sync the ATA to have the correct balance for the Jupiter program.
        spl_token::instruction::sync_native(&spl_token::ID, &wsol_ata)?,
    ];

    // --- 2. Jupiter Lend Instruction ---
    let rpc_client = RpcClient::new(env.rpc_client.url());
    let jupiter_client = Jupiter::surfpool_with_rpc(rpc_client).with_user_pubkey(user_pubkey);
    let deposit_params = DepositParams {
        asset_mint: spl_token::native_mint::ID, // Lend APIs use WSOL mint to represent native SOL
        amount,
    };

    info!("[Test Helper] Getting Jupiter transaction components via SDK...");
    let (mut jupiter_instructions, alt_accounts) = jupiter_client
        .deposit(deposit_params)
        .prepare_transaction_components()
        .await
        .context("Failed to get Jupiter lend deposit components from jup-sdk")?;

    // --- 3. Preload all accounts for Jupiter instructions ---
    info!("[Test Helper] Starting account pre-loading process via SDK...");
    let surfpool_client = SurfpoolClient::new(&env.rpc_client.url());
    jup_sdk::surfpool::preload_accounts(
        &env.rpc_client,
        &surfpool_client,
        &user_pubkey,
        &jupiter_instructions,
        &alt_accounts,
    )
    .await
    .context("jup-sdk failed to preload accounts")?;

    // --- 4. Combine all instructions ---
    let mut all_instructions = Vec::new();
    all_instructions.append(&mut wrap_instructions);
    all_instructions.append(&mut jupiter_instructions);

    Ok(all_instructions)
}
